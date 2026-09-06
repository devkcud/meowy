use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::{self, Token, TokenKind};

pub(crate) type ParseResult<T> = Result<T, Diagnostic>;

pub(crate) enum TreeNode<'a> {
    Expr(&'a Expr),
    Stmt(&'a Stmt),
}

pub fn parse(source: &str) -> Result<Block, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let mut parser = Parser::new(tokens);
    let block = parser.block(None, false, 0);
    if parser.errors.is_empty() {
        Ok(block)
    } else {
        Err(parser.errors)
    }
}

pub(crate) struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) depth: usize,
    pub(crate) errors: Vec<Diagnostic>,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens
                .into_iter()
                .filter(|token| !matches!(token.kind, TokenKind::Space | TokenKind::Comment))
                .collect(),
            pos: 0,
            depth: 0,
            errors: Vec::new(),
        }
    }

    pub(crate) fn token(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    pub(crate) fn at(&self, text: &str) -> bool {
        self.token().text == text
    }

    pub(crate) fn eof(&self) -> bool {
        self.token().kind == TokenKind::Eof
    }

    pub(crate) fn bump(&mut self) -> Token {
        let token = self.token().clone();
        if !self.eof() {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn take(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(crate) fn need(&mut self, text: &str) -> ParseResult<Token> {
        if self.at(text) {
            Ok(self.bump())
        } else {
            let code = if matches!(text, ")" | "]" | "}" | ">") {
                "E002"
            } else {
                "E004"
            };
            Err(Diagnostic::new(
                code,
                format!("expected `{text}`, found {}", self.description()),
                self.token().span,
            ))
        }
    }

    pub(crate) fn description(&self) -> String {
        if self.eof() {
            "end of file".to_owned()
        } else if self.token().kind == TokenKind::Newline {
            "newline".to_owned()
        } else {
            format!("`{}`", self.token().text)
        }
    }

    pub(crate) fn name(&mut self) -> ParseResult<Token> {
        if self.token().kind == TokenKind::Name {
            Ok(self.bump())
        } else {
            Err(Diagnostic::new(
                "E004",
                "expected a name",
                self.token().span,
            ))
        }
    }

    pub(crate) fn newlines(&mut self) {
        while self.token().kind == TokenKind::Newline {
            self.bump();
        }
    }

    pub(crate) fn separators(&mut self) {
        while self.token().kind == TokenKind::Newline || self.at(";") {
            self.bump();
        }
    }

    pub(crate) fn end(&self) -> usize {
        if self.pos == 0 {
            self.token().span.start
        } else {
            self.tokens[self.pos - 1].span.end
        }
    }

    pub(crate) fn terminator(&self) -> bool {
        self.eof() || self.at(";") || self.at("}") || self.token().kind == TokenKind::Newline
    }

    pub(crate) fn block(&mut self, label: Option<String>, closed: bool, start: usize) -> Block {
        let mut stmts = Vec::new();
        self.separators();
        while !self.eof() && !(closed && self.at("}")) {
            let before = self.pos;
            match self.statement() {
                Ok(stmt) => {
                    stmts.push(stmt);
                    if !self.terminator() {
                        self.errors.push(Diagnostic::new(
                            "E004",
                            "expected a newline or `;` between statements",
                            self.token().span,
                        ));
                        self.recover(closed);
                    }
                }
                Err(error) => {
                    self.errors.push(error);
                    self.recover(closed);
                }
            }
            if self.pos == before && !self.eof() && !(closed && self.at("}")) {
                self.bump();
            }
            self.separators();
        }
        if closed && let Err(error) = self.need("}") {
            self.errors.push(error);
        }
        Block {
            label,
            stmts,
            span: Span::new(start, self.end()),
        }
    }

    pub(crate) fn recover(&mut self, closed: bool) {
        while !self.eof()
            && !self.at(";")
            && self.token().kind != TokenKind::Newline
            && !(closed && self.at("}"))
        {
            self.bump();
        }
    }

    pub(crate) fn statement(&mut self) -> ParseResult<Stmt> {
        if self.depth >= 64 {
            return Err(Diagnostic::unsupported(
                "statement nesting beyond 64 levels",
                self.token().span,
            ));
        }
        self.depth += 1;
        let result = self.statement_inner();
        self.depth -= 1;
        result
    }

    pub(crate) fn statement_inner(&mut self) -> ParseResult<Stmt> {
        let start = self.token().span.start;
        let kind = if self.take("|") {
            let condition = self.expr(0, true, false, true)?;
            self.need("|")?;
            self.newlines();
            let body = self.statement()?;
            StmtKind::Match {
                arms: vec![(Some(condition), Box::new(body))],
            }
        } else if self.take("->") {
            self.emission(None)?
        } else if self.at("'")
            && self
                .tokens
                .get(self.pos + 2)
                .is_some_and(|token| token.text == "->")
        {
            self.bump();
            let label = self.name()?.text;
            self.need("->")?;
            self.emission(Some(label))?
        } else if self.at("<") {
            let alias = self.type_union()?;
            self.need(":")?;
            self.newlines();
            let TypeKind::Name(name) = alias.kind else {
                return Err(Diagnostic::unsupported(
                    "generic or computed type alias names",
                    alias.span,
                ));
            };
            let ty = if self.at("<") {
                self.type_union()?
            } else {
                let value = self.expr(0, false, false, false)?;
                TypeExpr {
                    span: value.span,
                    kind: TypeKind::Computed(Box::new(value)),
                }
            };
            StmtKind::TypeAlias { name, ty }
        } else if let Some((name, ty, mutable)) = self.binding_head()? {
            self.newlines();
            StmtKind::Bind {
                name,
                ty,
                mutable,
                value: self.expr(0, false, false, false)?,
            }
        } else {
            let save = self.pos;
            let forward = if self.token().kind == TokenKind::Name {
                let name = self.bump().text;
                match self.type_union() {
                    Ok(ty) if self.terminator() && matches!(ty.kind, TypeKind::Function { .. }) => {
                        Some(StmtKind::Forward { name, ty })
                    }
                    _ => None,
                }
            } else {
                None
            };
            if let Some(kind) = forward {
                kind
            } else {
                self.pos = save;
                let target = self.expr(0, false, false, false)?;
                if self.take("=") {
                    self.newlines();
                    StmtKind::Assign {
                        target,
                        value: self.expr(0, false, false, false)?,
                    }
                } else {
                    StmtKind::Expr(target)
                }
            }
        };
        Ok(Stmt {
            kind,
            span: Span::new(start, self.end()),
        })
    }

    pub(crate) fn binding_head(&mut self) -> ParseResult<Option<(String, Option<TypeExpr>, bool)>> {
        if self.token().kind != TokenKind::Name {
            return Ok(None);
        }
        let save = self.pos;
        let name = self.bump().text;
        let ty = if self.at("<") {
            match self.type_union() {
                Ok(ty) => Some(ty),
                Err(error) if error.code == "B001" && self.annotation_binding(save + 1) => {
                    return Err(error);
                }
                Err(_) => {
                    self.pos = save;
                    return Ok(None);
                }
            }
        } else {
            None
        };
        if self.at(":") || self.at(":=") {
            let mutable = self.bump().text == ":=";
            Ok(Some((name, ty, mutable)))
        } else {
            self.pos = save;
            Ok(None)
        }
    }

    pub(crate) fn emission(&mut self, label: Option<String>) -> ParseResult<StmtKind> {
        self.newlines();
        let (name, ty, mutable) = match self.binding_head()? {
            Some((name, ty, mutable)) => (Some(name), ty, mutable),
            None => (None, None, false),
        };
        self.newlines();
        let value = self.expr(0, false, false, false)?;
        Ok(StmtKind::Emit {
            label,
            name,
            ty,
            mutable,
            value,
        })
    }

    pub(crate) fn expr(
        &mut self,
        min: u8,
        condition: bool,
        multiline: bool,
        pipe: bool,
    ) -> ParseResult<Expr> {
        if self.depth >= 64 {
            return Err(Diagnostic::unsupported(
                "expression nesting beyond 64 levels",
                self.token().span,
            ));
        }
        self.depth += 1;
        let result = self.expr_inner(min, condition, multiline, pipe);
        self.depth -= 1;
        result
    }

    pub(crate) fn expr_inner(
        &mut self,
        min: u8,
        condition: bool,
        multiline: bool,
        pipe: bool,
    ) -> ParseResult<Expr> {
        if multiline {
            self.newlines();
        }
        let mut left = self.prefix(condition, multiline, pipe)?;
        loop {
            if !bounded_tree(&left) {
                return Err(Diagnostic::unsupported(
                    "syntax tree depth beyond 256 levels",
                    left.span,
                ));
            }
            if multiline {
                self.newlines();
            } else if self.token().kind == TokenKind::Newline {
                let save = self.pos;
                self.newlines();
                if !self.at(".") {
                    self.pos = save;
                    break;
                }
            }
            let start = left.span.start;
            if min <= 100 {
                if self.take("(") {
                    let args = self.arguments(")")?;
                    left = Expr {
                        kind: ExprKind::Call {
                            callee: Box::new(left),
                            args,
                        },
                        span: Span::new(start, self.end()),
                    };
                    continue;
                }
                if self.take("[") {
                    let index = self.expr(0, false, true, false)?;
                    self.need("]")?;
                    left = Expr {
                        kind: ExprKind::Index {
                            value: Box::new(left),
                            index: Box::new(index),
                        },
                        span: Span::new(start, self.end()),
                    };
                    continue;
                }
                if self.take(".") {
                    self.newlines();
                    let kind = if self.take("(") {
                        self.newlines();
                        let callee = self.expr(0, false, true, false)?;
                        let args = if self.take(",") {
                            self.arguments(")")?
                        } else {
                            self.need(")")?;
                            Vec::new()
                        };
                        ExprKind::Dispatch {
                            value: Box::new(left),
                            callee: Box::new(callee),
                            args,
                        }
                    } else if self.take("{") {
                        let block = self.block(None, true, self.tokens[self.pos - 1].span.start);
                        ExprKind::DispatchBlock {
                            value: Box::new(left),
                            block,
                        }
                    } else {
                        ExprKind::Field {
                            value: Box::new(left),
                            name: self.name()?.text,
                        }
                    };
                    left = Expr {
                        kind,
                        span: Span::new(start, self.end()),
                    };
                    continue;
                }
                if self.at("<") {
                    if self.specialized_call() {
                        return Err(Diagnostic::unsupported(
                            "generic call specialization",
                            self.token().span,
                        ));
                    }
                    let save = self.pos;
                    self.bump();
                    self.newlines();
                    if self.take(">") {
                        left = Expr {
                            kind: ExprKind::TypeQuery(Box::new(left)),
                            span: Span::new(start, self.end()),
                        };
                        continue;
                    }
                    self.pos = save;
                    if !condition || min <= 40 {
                        if let Ok(ty) = self.type_union() {
                            if self.at("(") {
                                return Err(Diagnostic::unsupported(
                                    "generic call specialization",
                                    Span::new(start, self.end()),
                                ));
                            }
                            if condition && is_comparison(&left) {
                                return Err(Diagnostic::new(
                                    "E004",
                                    "comparisons cannot be chained",
                                    ty.span,
                                ));
                            }
                            left = Expr {
                                kind: ExprKind::Ascribe {
                                    value: Box::new(left),
                                    ty,
                                    predicate: condition,
                                },
                                span: Span::new(start, self.end()),
                            };
                            continue;
                        }
                        self.pos = save;
                    }
                }
            }
            if pipe && self.at("|") {
                break;
            }
            let op = self.token().text.clone();
            let Some(level) = precedence(&op) else {
                break;
            };
            if level < min {
                break;
            }
            if level == 40 && is_comparison(&left) {
                return Err(Diagnostic::new(
                    "E004",
                    "comparisons cannot be chained",
                    self.token().span,
                ));
            }
            self.bump();
            self.newlines();
            let right = self.expr(level + 1, condition, multiline, pipe)?;
            left = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span: Span::new(start, self.end()),
            };
        }
        Ok(left)
    }

    pub(crate) fn prefix(
        &mut self,
        condition: bool,
        multiline: bool,
        pipe: bool,
    ) -> ParseResult<Expr> {
        let token = self.bump();
        let start = token.span.start;
        let kind = match token.kind {
            TokenKind::Int => ExprKind::Int(token.text),
            TokenKind::Float => ExprKind::Float(token.text),
            TokenKind::String => ExprKind::String(self.string_parts(&token)?),
            TokenKind::Name => ExprKind::Name(token.text),
            _ => match token.text.as_str() {
                "(" => {
                    if let Some((params, label)) = self.function_head()? {
                        self.need("{")?;
                        ExprKind::Function {
                            params,
                            body: self.block(label, true, start),
                        }
                    } else {
                        let value = self.expr(0, condition, true, false)?;
                        self.need(")")?;
                        ExprKind::Group(Box::new(value))
                    }
                }
                "{" => ExprKind::Block(self.block(None, true, start)),
                "'" => {
                    let label = self.name()?.text;
                    if self.take("{") {
                        ExprKind::Block(self.block(Some(label), true, start))
                    } else {
                        ExprKind::Label(label)
                    }
                }
                "[" => ExprKind::List(self.arguments("]")?),
                "@" => {
                    let path = self.bump();
                    if path.kind != TokenKind::String {
                        return Err(Diagnostic::new(
                            "E004",
                            "module import requires a string literal",
                            path.span,
                        ));
                    }
                    let parts = self.string_parts(&path)?;
                    if parts
                        .iter()
                        .any(|part| matches!(part, StringPart::Value(_)))
                    {
                        return Err(Diagnostic::unsupported(
                            "interpolated module paths",
                            path.span,
                        ));
                    }
                    ExprKind::Import(
                        parts
                            .into_iter()
                            .filter_map(|part| {
                                if let StringPart::Text(text) = part {
                                    Some(text)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    )
                }
                "<" => {
                    self.pos -= 1;
                    ExprKind::TypeValue(self.type_union()?)
                }
                "!" if self.at("{") => {
                    return Err(Diagnostic::unsupported(
                        "unchecked blocks",
                        Span::new(start, self.token().span.end),
                    ));
                }
                "!" | "-" | "~" | "*" | "&" | "&!" | ">>" | "<<" => {
                    self.newlines();
                    let value = self.expr(90, condition, multiline, pipe)?;
                    ExprKind::Unary {
                        op: token.text,
                        value: Box::new(value),
                    }
                }
                ")" | "]" | "}" => {
                    return Err(Diagnostic::new(
                        "E002",
                        "unexpected closing delimiter",
                        token.span,
                    ));
                }
                _ => {
                    return Err(Diagnostic::new(
                        "E004",
                        "expected an expression",
                        token.span,
                    ));
                }
            },
        };
        Ok(Expr {
            kind,
            span: Span::new(start, self.end()),
        })
    }

    pub(crate) fn specialized_call(&self) -> bool {
        if self.tokens[self.pos + 1..]
            .iter()
            .find(|token| token.kind != TokenKind::Newline)
            .is_some_and(|token| token.text == ">")
        {
            return false;
        }
        let mut angles = 0usize;
        let mut parens = 0usize;
        let mut brackets = 0usize;
        for (offset, token) in self.tokens[self.pos..].iter().enumerate() {
            if token.kind == TokenKind::Eof {
                return false;
            }
            match token.text.as_str() {
                "(" => parens += 1,
                ")" if parens > 0 => parens -= 1,
                "[" => brackets += 1,
                "]" if brackets > 0 => brackets -= 1,
                "<" if parens == 0 && brackets == 0 => angles += 1,
                ">" | ">>" if parens == 0 && brackets == 0 => {
                    let count = token.text.len();
                    if count > angles {
                        return false;
                    }
                    angles -= count;
                    if angles == 0 {
                        return self
                            .tokens
                            .get(self.pos + offset + 1)
                            .is_some_and(|next| next.text == "(");
                    }
                }
                "," | "." | "&" | "&!" | "*" | "!" => {}
                _ if parens > 0 || brackets > 0 => {}
                _ if matches!(
                    token.kind,
                    TokenKind::Name | TokenKind::Int | TokenKind::Newline
                ) => {}
                _ => return false,
            }
        }
        false
    }

    pub(crate) fn annotation_binding(&self, start: usize) -> bool {
        let mut depth = 0usize;
        for token in &self.tokens[start..] {
            match token.text.as_str() {
                "<" => depth += 1,
                ">" | ">>" if token.text.len() <= depth => depth -= token.text.len(),
                ":" | ":=" if depth == 0 => return true,
                ";" if depth == 0 => return false,
                _ if depth == 0 => return false,
                _ => {}
            }
        }
        false
    }

    pub(crate) fn function_head(&mut self) -> ParseResult<Option<(Vec<Param>, Option<String>)>> {
        let save = self.pos;
        self.newlines();
        let mut params = Vec::new();
        if !self.at(")") {
            loop {
                let name = match self.name() {
                    Ok(name) => name,
                    Err(_) => {
                        self.pos = save;
                        return Ok(None);
                    }
                };
                let ty = match self.type_union() {
                    Ok(ty) => ty,
                    Err(_) => {
                        self.pos = save;
                        return Ok(None);
                    }
                };
                params.push(Param {
                    name: name.text,
                    span: Span::new(name.span.start, ty.span.end),
                    ty,
                });
                self.newlines();
                if !self.take(",") {
                    break;
                }
                self.newlines();
                if self.at(")") {
                    break;
                }
            }
        }
        if !self.take(")") {
            self.pos = save;
            return Ok(None);
        }
        let label = if self.take("'") {
            Some(self.name()?.text)
        } else {
            None
        };
        if self.at("!") {
            return Err(Diagnostic::unsupported(
                "unchecked functions",
                self.token().span,
            ));
        }
        if !self.at("{") {
            self.pos = save;
            return Ok(None);
        }
        Ok(Some((params, label)))
    }

    pub(crate) fn arguments(&mut self, close: &str) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();
        self.newlines();
        if self.take(close) {
            return Ok(args);
        }
        loop {
            args.push(self.expr(0, false, true, false)?);
            if self.take(close) {
                break;
            }
            self.need(",")?;
            self.newlines();
            if self.take(close) {
                break;
            }
        }
        Ok(args)
    }

    pub(crate) fn type_union(&mut self) -> ParseResult<TypeExpr> {
        let mut members = vec![self.type_bracket()?];
        while self.at("<") {
            let save = self.pos;
            match self.type_bracket() {
                Ok(ty) => members.push(ty),
                Err(_) => {
                    self.pos = save;
                    break;
                }
            }
        }
        if self.at("!")
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|token| token.text == "<")
        {
            return Err(Diagnostic::unsupported(
                "type subtraction",
                self.token().span,
            ));
        }
        if members.len() == 1 {
            Ok(members.remove(0))
        } else {
            let span = Span::new(members[0].span.start, members.last().unwrap().span.end);
            Ok(TypeExpr {
                kind: TypeKind::Union(members),
                span,
            })
        }
    }

    pub(crate) fn type_bracket(&mut self) -> ParseResult<TypeExpr> {
        if self.depth >= 64 {
            return Err(Diagnostic::unsupported(
                "type nesting beyond 64 levels",
                self.token().span,
            ));
        }
        self.depth += 1;
        let result = self.type_bracket_inner();
        self.depth -= 1;
        result
    }

    pub(crate) fn type_bracket_inner(&mut self) -> ParseResult<TypeExpr> {
        let start = self.need("<")?.span.start;
        self.newlines();
        if self.at(":") {
            return Err(Diagnostic::unsupported(
                "generic type binders",
                self.token().span,
            ));
        }
        let mut ty = self.type_atom()?;
        self.newlines();
        self.need(">")?;
        ty.span = Span::new(start, self.end());
        Ok(ty)
    }

    pub(crate) fn type_atom(&mut self) -> ParseResult<TypeExpr> {
        if self.depth >= 64 {
            return Err(Diagnostic::unsupported(
                "type nesting beyond 64 levels",
                self.token().span,
            ));
        }
        self.depth += 1;
        let result = self.type_atom_inner();
        self.depth -= 1;
        result
    }

    pub(crate) fn type_atom_inner(&mut self) -> ParseResult<TypeExpr> {
        self.newlines();
        let start = self.token().span.start;
        let kind = if self.take("{") {
            let mut primary = None;
            let mut fields = Vec::new();
            self.separators();
            while !self.at("}") && !self.eof() {
                if self.take("->") {
                    self.newlines();
                    if primary.is_some() {
                        return Err(Diagnostic::new(
                            "E203",
                            "duplicate record primary type",
                            self.token().span,
                        ));
                    }
                    primary = Some(Box::new(self.type_union()?));
                } else {
                    let name = self.name()?.text;
                    self.newlines();
                    let ty = self.type_union()?;
                    let mutable = self.take(":=");
                    fields.push((name, ty, mutable));
                }
                if !self.at("}") && !self.terminator() {
                    return Err(Diagnostic::new(
                        "E004",
                        "expected a separator between record fields",
                        self.token().span,
                    ));
                }
                self.separators();
            }
            self.need("}")?;
            TypeKind::Record { primary, fields }
        } else if self.at("&") || self.at("&!") {
            let mutable = self.bump().text == "&!";
            TypeKind::Reference {
                value: Box::new(self.type_atom()?),
                mutable,
            }
        } else if self.at("*") {
            return Err(Diagnostic::unsupported(
                "raw pointer types",
                self.token().span,
            ));
        } else if self.take("(") {
            let save = self.pos;
            let parsed = self.function_type_params();
            if let Ok(params) = parsed {
                self.newlines();
                if self.take("->") {
                    let result = self.type_atom()?;
                    TypeKind::Function {
                        params,
                        result: Box::new(result),
                    }
                } else {
                    self.pos = save;
                    let value = self.expr(0, false, true, false)?;
                    self.need(")")?;
                    TypeKind::Computed(Box::new(value))
                }
            } else {
                self.pos = save;
                let value = self.expr(0, false, true, false)?;
                self.need(")")?;
                TypeKind::Computed(Box::new(value))
            }
        } else {
            let mut name = self.name()?.text;
            while self.take(".") {
                name.push('.');
                name.push_str(&self.name()?.text);
            }
            if self.at("<") {
                return Err(Diagnostic::unsupported(
                    "generic type arguments",
                    self.token().span,
                ));
            }
            TypeKind::Name(name)
        };
        let mut ty = TypeExpr {
            kind,
            span: Span::new(start, self.end()),
        };
        while self.take("[") {
            self.newlines();
            let size = if self.at("]") {
                None
            } else {
                Some(Box::new(self.expr(0, false, true, false)?))
            };
            self.need("]")?;
            ty = TypeExpr {
                kind: TypeKind::List {
                    element: Box::new(ty),
                    size,
                },
                span: Span::new(start, self.end()),
            };
        }
        Ok(ty)
    }

    pub(crate) fn function_type_params(&mut self) -> ParseResult<Vec<TypeExpr>> {
        let mut params = Vec::new();
        self.newlines();
        if self.take(")") {
            return Ok(params);
        }
        loop {
            params.push(if self.at("<") {
                self.type_union()?
            } else {
                self.type_atom()?
            });
            self.newlines();
            if self.take(")") {
                break;
            }
            self.need(",")?;
            self.newlines();
        }
        Ok(params)
    }

    pub(crate) fn string_parts(&mut self, token: &Token) -> ParseResult<Vec<StringPart>> {
        let text = &token.text;
        let bytes = text.as_bytes();
        let mut pos = 1;
        let mut value = String::new();
        let mut parts = Vec::new();
        while pos + 1 < bytes.len() {
            match bytes[pos] {
                b'\\' => {
                    pos += 1;
                    value.push(match bytes[pos] {
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'0' => '\0',
                        byte => byte as char,
                    });
                    pos += 1;
                }
                b'{' => {
                    if !value.is_empty() {
                        parts.push(StringPart::Text(std::mem::take(&mut value)));
                    }
                    let mut errors = Vec::new();
                    let end = lexer::scan_interpolation(text, pos, 0, &mut errors);
                    if let Some(mut error) = errors.into_iter().next() {
                        error.span.start += token.span.start;
                        error.span.end += token.span.start;
                        return Err(error);
                    }
                    let offset = token.span.start + pos + 1;
                    let tokens = lexer::lex(&text[pos + 1..end - 1]).map_err(|mut errors| {
                        let mut error = errors.remove(0);
                        error.span.start += offset;
                        error.span.end += offset;
                        error
                    })?;
                    let tokens = tokens
                        .into_iter()
                        .map(|mut token| {
                            token.span.start += offset;
                            token.span.end += offset;
                            token
                        })
                        .collect();
                    let mut parser = Parser::new(tokens);
                    parser.depth = self.depth;
                    let expression = parser.expr(0, false, true, false)?;
                    if !parser.errors.is_empty() {
                        return Err(parser.errors.remove(0));
                    }
                    if !parser.eof() {
                        return Err(Diagnostic::new(
                            "E004",
                            "expected one interpolation expression",
                            parser.token().span,
                        ));
                    }
                    parts.push(StringPart::Value(expression));
                    pos = end;
                }
                _ => {
                    let character = text[pos..].chars().next().unwrap();
                    value.push(character);
                    pos += character.len_utf8();
                }
            }
        }
        if !value.is_empty() || parts.is_empty() {
            parts.push(StringPart::Text(value));
        }
        Ok(parts)
    }
}

pub(crate) fn precedence(op: &str) -> Option<u8> {
    match op {
        "*" | "/" | "%" => Some(80),
        "+" | "-" => Some(70),
        "&" => Some(60),
        "^" => Some(55),
        "|" => Some(50),
        "<" | "<=" | ">" | ">=" => Some(40),
        "==" | "!=" => Some(30),
        "&&" => Some(20),
        "||" => Some(10),
        _ => None,
    }
}

pub(crate) fn is_comparison(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Binary { op, .. } if matches!(op.as_str(), "<" | "<=" | ">" | ">="))
        || matches!(
            expr.kind,
            ExprKind::Ascribe {
                predicate: true,
                ..
            }
        )
}

pub(crate) fn bounded_tree(expr: &Expr) -> bool {
    let mut pending = vec![(TreeNode::Expr(expr), 1usize)];
    while let Some((node, depth)) = pending.pop() {
        if depth > 256 {
            return false;
        }
        let depth = depth + 1;
        match node {
            TreeNode::Stmt(stmt) => match &stmt.kind {
                StmtKind::Bind { value, .. }
                | StmtKind::Emit { value, .. }
                | StmtKind::Expr(value) => pending.push((TreeNode::Expr(value), depth)),
                StmtKind::Assign { target, value } => {
                    pending.push((TreeNode::Expr(target), depth));
                    pending.push((TreeNode::Expr(value), depth));
                }
                StmtKind::Match { arms } => {
                    for (condition, body) in arms {
                        if let Some(condition) = condition {
                            pending.push((TreeNode::Expr(condition), depth));
                        }
                        pending.push((TreeNode::Stmt(body), depth));
                    }
                }
                StmtKind::TypeAlias { .. } | StmtKind::Forward { .. } => {}
            },
            TreeNode::Expr(expr) => match &expr.kind {
                ExprKind::Unary { value, .. }
                | ExprKind::Group(value)
                | ExprKind::Field { value, .. }
                | ExprKind::Ascribe { value, .. }
                | ExprKind::TypeQuery(value) => pending.push((TreeNode::Expr(value), depth)),
                ExprKind::Binary { left, right, .. } => {
                    pending.push((TreeNode::Expr(left), depth));
                    pending.push((TreeNode::Expr(right), depth));
                }
                ExprKind::Index { value, index } => {
                    pending.push((TreeNode::Expr(value), depth));
                    pending.push((TreeNode::Expr(index), depth));
                }
                ExprKind::Call { callee, args } | ExprKind::Dispatch { callee, args, .. } => {
                    pending.push((TreeNode::Expr(callee), depth));
                    for arg in args {
                        pending.push((TreeNode::Expr(arg), depth));
                    }
                    if let ExprKind::Dispatch { value, .. } = &expr.kind {
                        pending.push((TreeNode::Expr(value), depth));
                    }
                }
                ExprKind::Block(block)
                | ExprKind::Function { body: block, .. }
                | ExprKind::DispatchBlock { block, .. } => {
                    for stmt in &block.stmts {
                        pending.push((TreeNode::Stmt(stmt), depth));
                    }
                    if let ExprKind::DispatchBlock { value, .. } = &expr.kind {
                        pending.push((TreeNode::Expr(value), depth));
                    }
                }
                ExprKind::List(values) => {
                    for value in values {
                        pending.push((TreeNode::Expr(value), depth));
                    }
                }
                ExprKind::String(parts) => {
                    for part in parts {
                        if let StringPart::Value(value) = part {
                            pending.push((TreeNode::Expr(value), depth));
                        }
                    }
                }
                _ => {}
            },
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn value(source: &str) -> Expr {
        let block = parse(source).unwrap();
        match block.stmts.into_iter().next().unwrap().kind {
            StmtKind::Bind { value, .. } | StmtKind::Expr(value) => value,
            other => panic!("unexpected statement: {other:?}"),
        }
    }

    #[test]
    pub(crate) fn preserves_grouped_integer_negation() {
        let ExprKind::Unary { value: raw, .. } = value("x<int8>:-128").kind else {
            panic!()
        };
        assert!(matches!(raw.kind, ExprKind::Int(_)));
        let ExprKind::Unary { value: grouped, .. } = value("x<int8>:-(128)").kind else {
            panic!()
        };
        assert!(matches!(grouped.kind, ExprKind::Group(_)));
    }

    #[test]
    pub(crate) fn parses_functions_labels_and_emissions() {
        let source = "even<(uint32)->boolean>;even<boolean>:(n<uint32>) 'answer {|n==0|{'answer->true;'answer.leave()};->even(n-1)}";
        let block = parse(source).unwrap();
        assert_eq!(block.stmts.len(), 2);
        assert!(matches!(block.stmts[0].kind, StmtKind::Forward { .. }));
        let StmtKind::Bind { value, .. } = &block.stmts[1].kind else {
            panic!()
        };
        let ExprKind::Function { body, .. } = &value.kind else {
            panic!()
        };
        assert_eq!(body.label.as_deref(), Some("answer"));
        assert!(matches!(body.stmts[1].kind, StmtKind::Emit { .. }));
    }

    #[test]
    pub(crate) fn honors_context_for_type_suffixes() {
        let block = parse("|!(value<int32>) && accepts(value<int32>)|copy:value<int32>").unwrap();
        let StmtKind::Match { arms } = &block.stmts[0].kind else {
            panic!()
        };
        let ExprKind::Binary { left, right, .. } = &arms[0].0.as_ref().unwrap().kind else {
            panic!()
        };
        let ExprKind::Unary { value, .. } = &left.kind else {
            panic!()
        };
        let ExprKind::Group(value) = &value.kind else {
            panic!()
        };
        assert!(matches!(
            value.kind,
            ExprKind::Ascribe {
                predicate: true,
                ..
            }
        ));
        let ExprKind::Call { args, .. } = &right.kind else {
            panic!()
        };
        assert!(matches!(
            args[0].kind,
            ExprKind::Ascribe {
                predicate: false,
                ..
            }
        ));
        let StmtKind::Bind { value, .. } = &arms[0].1.kind else {
            panic!()
        };
        assert!(matches!(
            value.kind,
            ExprKind::Ascribe {
                predicate: false,
                ..
            }
        ));
    }

    #[test]
    pub(crate) fn gives_predicates_comparison_precedence() {
        let block = parse("|!value<boolean>|f()").unwrap();
        let StmtKind::Match { arms } = &block.stmts[0].kind else {
            panic!()
        };
        let ExprKind::Ascribe {
            value,
            predicate: true,
            ..
        } = &arms[0].0.as_ref().unwrap().kind
        else {
            panic!()
        };
        assert!(matches!(value.kind, ExprKind::Unary { .. }));
        assert!(parse("x:1<2<3").is_err());
        assert!(parse("|left<limit&&other>0|f()").is_ok());
        assert_eq!(parse("left<limit<other").unwrap_err()[0].code, "E004");
        let ExprKind::Binary { left, op, .. } = self::value("copy:value<int32><2").kind else {
            panic!()
        };
        assert_eq!(op, "<");
        assert!(matches!(
            left.kind,
            ExprKind::Ascribe {
                predicate: false,
                ..
            }
        ));
    }

    #[test]
    pub(crate) fn parses_dispatch_and_continuation() {
        let expr = value("answer:20\n.(increment)\n.(double,\n2)");
        let ExprKind::Dispatch { value, args, .. } = expr.kind else {
            panic!()
        };
        assert_eq!(args.len(), 1);
        assert!(matches!(value.kind, ExprKind::Dispatch { .. }));
        assert!(parse("x:1+\n2\ny:(1\n+2)").is_ok());
        assert!(parse("x:1 y:2").is_err());
    }

    #[test]
    pub(crate) fn parses_record_annotations_and_computed_types() {
        assert!(parse("<Reading>:<{-><int32>;unit<string>;count<uint32>:=}>;x<Reading>:{->2;->unit:\"c\";->count:=0}").is_ok());
        assert!(parse("other<(name<>)>:value").is_ok());
        assert!(parse("other<(element)[capacity]>:value").is_ok());
        let block = parse("<Bytes>:bounded(<uint8>,4)").unwrap();
        let StmtKind::TypeAlias { ty, .. } = &block.stmts[0].kind else {
            panic!()
        };
        assert!(matches!(ty.kind, TypeKind::Computed(_)));
    }

    #[test]
    pub(crate) fn interpolation_has_absolute_spans_and_nested_strings() {
        let expr = value("text:\"a {f(\"b {2}\")} z\"");
        let ExprKind::String(parts) = expr.kind else {
            panic!()
        };
        let StringPart::Value(expr) = &parts[1] else {
            panic!()
        };
        assert_eq!(expr.span.start, 9);
        assert!(matches!(expr.kind, ExprKind::Call { .. }));
        let errors = parse("text:\"bad {1 + }\"").unwrap_err();
        assert_eq!(errors[0].span.start, 15);
    }

    #[test]
    pub(crate) fn rejects_unavailable_generics_and_malformed_delimiters() {
        assert_eq!(parse("f<:T>:(x<T>){->x}").unwrap_err()[0].code, "B001");
        assert_eq!(parse("f<int32>(1)").unwrap_err()[0].code, "B001");
        assert_eq!(parse("bytes.filled<4>(0)").unwrap_err()[0].code, "B001");
        assert_eq!(parse("x:(1]").unwrap_err()[0].code, "E002");
        assert!(parse("|true|f();||g()").is_err());
    }

    #[test]
    pub(crate) fn bounds_nesting_without_panicking() {
        let source = format!("x:{}1{}", "(".repeat(80), ")".repeat(80));
        assert!(
            parse(&source)
                .unwrap_err()
                .iter()
                .any(|error| error.code == "B001")
        );
        let source = format!("{}f()", "|true| ".repeat(80));
        assert!(
            parse(&source)
                .unwrap_err()
                .iter()
                .any(|error| error.code == "B001")
        );
    }

    #[test]
    pub(crate) fn bounds_tree_depth_for_flat_operator_chains() {
        assert!(parse(&format!("x:{}", vec!["true"; 80].join("||"))).is_ok());
        for source in [
            format!("x:{}", vec!["1"; 20_000].join("+")),
            format!("x:{}", vec!["value"; 20_000].join(".")),
        ] {
            assert_eq!(parse(&source).unwrap_err()[0].code, "B001");
        }
    }

    #[test]
    pub(crate) fn malformed_unicode_inputs_preserve_lexer_spans_without_panics() {
        let symbols = [
            "a", "1", "0x", " ", "\n", "\r", "<", ">", "(", ")", "{", "}", "[", "]", "|", "!", "&",
            ";", ":", "=", "+", ".", "'", "@", "\"", "\\", "#", "é", "💖",
        ];
        let mut seed = 0x59eedu64;
        for case in 0..10_000 {
            let mut source = String::new();
            for _ in 0..case % 96 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                source.push_str(symbols[(seed >> 32) as usize % symbols.len()]);
            }
            let _ = parse(&source);
            if let Ok(tokens) = lexer::lex(&source) {
                assert_eq!(
                    tokens
                        .iter()
                        .map(|token| token.text.as_str())
                        .collect::<String>(),
                    source
                );
                for token in tokens {
                    assert_eq!(
                        source.get(token.span.start..token.span.end),
                        Some(token.text.as_str())
                    );
                }
            }
        }
    }
}
