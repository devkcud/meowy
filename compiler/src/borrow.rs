use std::collections::{BTreeMap, BTreeSet};

use crate::ast::Span;
use crate::diagnostic::Diagnostic;
use crate::hir::{Block, BlockId, Expr, ExprKind, LocalId, Place, Program, Stmt, Type};

pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;

#[derive(Clone)]
pub(crate) struct Storage {
    pub(crate) block: BlockId,
    pub(crate) origin: Option<Place>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Exit {
    Leave(BlockId),
    Restart(BlockId),
}

pub(crate) struct Flow {
    pub(crate) next: bool,
    pub(crate) exits: BTreeSet<Exit>,
}

impl Flow {
    pub(crate) fn new() -> Self {
        Self {
            next: true,
            exits: BTreeSet::new(),
        }
    }

    pub(crate) fn append(&mut self, next: Self) {
        self.next = next.next;
        self.exits.extend(next.exits);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.next |= other.next;
        self.exits.extend(other.exits);
    }
}

pub(crate) struct Value {
    pub(crate) origin: Option<Place>,
    pub(crate) flow: Flow,
}

pub(crate) struct Checker<'a> {
    pub(crate) program: &'a Program,
    pub(crate) locals: BTreeMap<LocalId, Storage>,
    pub(crate) blocks: Vec<BlockId>,
    pub(crate) types: BTreeMap<BlockId, Type>,
    pub(crate) scopes: Vec<Vec<LocalId>>,
    pub(crate) branches: usize,
}

pub(crate) fn check(program: &Program) -> std::result::Result<(), Vec<Diagnostic>> {
    let mut checker = Checker {
        program,
        locals: BTreeMap::new(),
        blocks: Vec::new(),
        types: BTreeMap::new(),
        scopes: Vec::new(),
        branches: 0,
    };
    let result = (|| {
        checker.block(&program.body)?;
        for function in &program.functions {
            checker.locals.clear();
            for id in &function.params {
                checker.locals.insert(
                    *id,
                    Storage {
                        block: function.body.id,
                        origin: None,
                    },
                );
            }
            checker.block(&function.body)?;
        }
        Ok(())
    })();
    result.map_err(|error| vec![error])
}

impl Checker<'_> {
    pub(crate) fn unsupported(span: Span) -> Diagnostic {
        Diagnostic::unsupported("borrow origins outside immutable local storage", span)
    }

    pub(crate) fn close_scope(&mut self) {
        for id in self.scopes.pop().expect("storage scope") {
            self.locals.remove(&id);
        }
    }

    pub(crate) fn bind(&mut self, id: LocalId, origin: Option<Place>, span: Span) -> Result<()> {
        let ty = self
            .program
            .locals
            .get(id)
            .ok_or_else(|| Self::unsupported(span))?;
        if self.locals.contains_key(&id)
            || (ty.has_reference() && (!matches!(ty, Type::Reference(_)) || origin.is_none()))
        {
            return Err(Self::unsupported(span));
        }
        self.locals.insert(
            id,
            Storage {
                block: *self.blocks.last().expect("storage block"),
                origin,
            },
        );
        self.scopes.last_mut().expect("storage scope").push(id);
        Ok(())
    }

    pub(crate) fn live(&self, place: &Place, span: Span) -> Result<&Storage> {
        self.locals.get(&place.root).ok_or_else(|| {
            Diagnostic::new("E303", "borrowed storage has ended before this use", span)
        })
    }

    pub(crate) fn block(&mut self, block: &Block) -> Result<Flow> {
        self.blocks.push(block.id);
        self.types.insert(block.id, block.ty.clone());
        self.scopes.push(Vec::new());
        let mut flow = self.statements(&block.stmts, true)?;
        self.close_scope();
        self.blocks.pop();
        self.types.remove(&block.id);
        flow.next |= flow.exits.remove(&Exit::Leave(block.id));
        flow.exits.remove(&Exit::Restart(block.id));
        Ok(flow)
    }

    pub(crate) fn branch(&mut self, stmts: &[Stmt]) -> Result<Flow> {
        self.scopes.push(Vec::new());
        self.branches += 1;
        let flow = self.statements(stmts, false)?;
        self.branches -= 1;
        self.close_scope();
        Ok(flow)
    }

    pub(crate) fn statements(&mut self, stmts: &[Stmt], direct: bool) -> Result<Flow> {
        let mut flow = Flow::new();
        for (index, stmt) in stmts.iter().enumerate() {
            if !flow.next {
                break;
            }
            let next = match stmt {
                Stmt::Bind { id, value } => {
                    let result = self.expression(value)?;
                    if result.flow.next {
                        self.bind(*id, result.origin, value.span)?;
                    }
                    result.flow
                }
                Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    if result.flow.next
                        && (result.origin.is_some() || self.program.locals[*id].has_reference())
                    {
                        return Err(Self::unsupported(value.span));
                    }
                    result.flow
                }
                Stmt::Emit {
                    target,
                    field,
                    value,
                } => {
                    let result = self.expression(value)?;
                    if result.flow.next && value.ty.has_reference() {
                        let place = result.origin.ok_or_else(|| Self::unsupported(value.span))?;
                        let storage = self.live(&place, value.span)?;
                        let source = self
                            .blocks
                            .iter()
                            .position(|id| *id == storage.block)
                            .ok_or_else(|| Self::unsupported(value.span))?;
                        let target_index = self
                            .blocks
                            .iter()
                            .position(|id| id == target)
                            .ok_or_else(|| Self::unsupported(value.span))?;
                        let ty = self
                            .types
                            .get(target)
                            .ok_or_else(|| Self::unsupported(value.span))?;
                        let slot = match (ty, field) {
                            (Type::Record { primary, .. }, None) => Some(primary.as_ref()),
                            (Type::Record { fields, .. }, Some(name)) => fields
                                .iter()
                                .find(|(field, _)| field == name)
                                .map(|(_, ty)| ty),
                            (ty, None) => Some(ty),
                            _ => None,
                        };
                        let completes = direct
                            && self.branches == 0
                            && index + 1 == stmts.len()
                            && self.blocks.last() == Some(target)
                            && slot.is_some_and(|ty| ty.has_reference() && ty.accepts(&value.ty));
                        return Err(if source >= target_index && completes {
                            Diagnostic::new(
                                "E303",
                                "emitted borrow outlives its local storage",
                                value.span,
                            )
                        } else if source < target_index {
                            Diagnostic::unsupported("borrowed block result contracts", value.span)
                        } else {
                            Diagnostic::unsupported(
                                "guarded or discarded reference emissions",
                                value.span,
                            )
                        });
                    }
                    result.flow
                }
                Stmt::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    let mut result = self.expression(condition)?.flow;
                    if result.next {
                        let next = match &condition.kind {
                            ExprKind::Bool(true) => self.branch(then)?,
                            ExprKind::Bool(false) => self.branch(otherwise)?,
                            _ => {
                                let mut next = self.branch(then)?;
                                next.merge(self.branch(otherwise)?);
                                next
                            }
                        };
                        result.append(next);
                    }
                    result
                }
                Stmt::Leave(id) | Stmt::Restart(id) => Flow {
                    next: false,
                    exits: BTreeSet::from([if matches!(stmt, Stmt::Leave(_)) {
                        Exit::Leave(*id)
                    } else {
                        Exit::Restart(*id)
                    }]),
                },
                Stmt::Expr(value) => self.expression(value)?.flow,
            };
            flow.append(next);
        }
        Ok(flow)
    }

    pub(crate) fn expression(&mut self, expr: &Expr) -> Result<Value> {
        let mut flow = Flow::new();
        let origin = match &expr.kind {
            ExprKind::Borrow(place) => {
                if !self.locals.contains_key(&place.root) {
                    return Err(Self::unsupported(expr.span));
                }
                let mut ty = self
                    .program
                    .locals
                    .get(place.root)
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                for index in &place.fields {
                    let Type::Record { fields, .. } = ty else {
                        return Err(Self::unsupported(expr.span));
                    };
                    ty = &fields
                        .get(*index)
                        .ok_or_else(|| Self::unsupported(expr.span))?
                        .1;
                }
                if ty.has_reference() || expr.ty != Type::Reference(Box::new(ty.clone())) {
                    return Err(Self::unsupported(expr.span));
                }
                Some(place.clone())
            }
            ExprKind::Local(id) if expr.ty.has_reference() => {
                let place = self
                    .locals
                    .get(id)
                    .and_then(|storage| storage.origin.clone())
                    .ok_or_else(|| Self::unsupported(expr.span))?;
                self.live(&place, expr.span)?;
                Some(place)
            }
            ExprKind::Coerce { value } => {
                let value = self.expression(value)?;
                flow = value.flow;
                value.origin
            }
            ExprKind::Deref(value) => {
                let value = self.expression(value)?;
                flow = value.flow;
                if flow.next {
                    let place = value.origin.ok_or_else(|| Self::unsupported(expr.span))?;
                    self.live(&place, expr.span)?;
                }
                None
            }
            ExprKind::Unary { value, .. }
            | ExprKind::Field { value, .. }
            | ExprKind::Primary(value)
            | ExprKind::StringSize(value)
            | ExprKind::TypeTest { value, .. } => {
                flow = self.expression(value)?.flow;
                None
            }
            ExprKind::Binary { op, left, right } => {
                flow = self.expression(left)?.flow;
                if flow.next {
                    let skip = match (&left.kind, op.as_str()) {
                        (ExprKind::Bool(value), "&&") => Some(!value),
                        (ExprKind::Bool(value), "||") => Some(*value),
                        _ => None,
                    };
                    if skip != Some(true) {
                        let mut next = self.expression(right)?.flow;
                        if skip.is_none() && ["&&", "||"].contains(&op.as_str()) {
                            next.next = true;
                        }
                        flow.append(next);
                    }
                }
                None
            }
            ExprKind::Call { args, .. } => {
                for arg in args {
                    if !flow.next {
                        break;
                    }
                    let value = self.expression(arg)?;
                    if value.flow.next && arg.ty.has_reference() {
                        return Err(Diagnostic::unsupported(
                            "function borrow contracts",
                            arg.span,
                        ));
                    }
                    flow.append(value.flow);
                }
                None
            }
            ExprKind::Print { parts, .. } | ExprKind::Panic { parts } => {
                for part in parts {
                    if !flow.next {
                        break;
                    }
                    flow.append(self.expression(part)?.flow);
                }
                if matches!(expr.kind, ExprKind::Panic { .. }) {
                    flow.next = false;
                }
                None
            }
            ExprKind::Block(block) => {
                flow = self.block(block)?;
                None
            }
            ExprKind::Null
            | ExprKind::Bool(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Local(_) => None,
        };
        if flow.next
            && expr.ty.has_reference()
            && (!matches!(expr.ty, Type::Reference(_)) || origin.is_none())
        {
            return Err(Self::unsupported(expr.span));
        }
        if expr.ty == Type::Never {
            flow.next = false;
        }
        Ok(Value { origin, flow })
    }
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::ast::Span;
    use crate::hir::{Block, Expr, ExprKind, Function, Place, Program, Stmt, Type};

    pub(crate) fn int() -> Type {
        Type::Int {
            bits: 32,
            signed: true,
        }
    }

    pub(crate) fn view() -> Type {
        Type::Reference(Box::new(int()))
    }

    pub(crate) fn expr(kind: ExprKind, ty: Type) -> Expr {
        Expr {
            kind,
            ty,
            span: Span { start: 0, end: 1 },
        }
    }

    pub(crate) fn owner() -> Stmt {
        Stmt::Bind {
            id: 0,
            value: expr(ExprKind::Int(1), int()),
        }
    }

    pub(crate) fn borrow() -> Expr {
        expr(
            ExprKind::Borrow(Place {
                root: 0,
                fields: Vec::new(),
            }),
            view(),
        )
    }

    pub(crate) fn local(id: usize) -> Expr {
        expr(ExprKind::Local(id), view())
    }

    pub(crate) fn block(id: usize, stmts: Vec<Stmt>) -> Block {
        Block {
            id,
            ty: Type::Null,
            stmts,
        }
    }

    pub(crate) fn program(stmts: Vec<Stmt>) -> Program {
        Program {
            body: block(0, stmts),
            functions: Vec::new(),
            locals: vec![int(), view(), view()],
        }
    }

    #[test]
    pub(crate) fn aliases_preserve_the_original_storage_across_nested_scopes() {
        let nested = block(
            1,
            vec![
                Stmt::Bind {
                    id: 2,
                    value: local(1),
                },
                Stmt::Expr(expr(ExprKind::Deref(Box::new(local(2))), int())),
            ],
        );
        let program = program(vec![
            owner(),
            Stmt::Bind {
                id: 1,
                value: borrow(),
            },
            Stmt::Expr(expr(ExprKind::Block(nested), Type::Null)),
            Stmt::Expr(expr(ExprKind::Deref(Box::new(local(1))), int())),
        ]);
        assert!(check(&program).is_ok());
    }

    #[test]
    pub(crate) fn nested_and_function_local_returns_outlive_their_owner() {
        let mut escaping = block(
            1,
            vec![
                owner(),
                Stmt::Bind {
                    id: 1,
                    value: borrow(),
                },
                Stmt::Emit {
                    target: 1,
                    field: None,
                    value: local(1),
                },
            ],
        );
        escaping.ty = view();
        let nested = program(vec![Stmt::Expr(expr(
            ExprKind::Block(escaping.clone()),
            view(),
        ))]);
        assert_eq!(check(&nested).unwrap_err()[0].code, "E303");
        let mut function = program(Vec::new());
        function.functions.push(Function {
            id: 0,
            name: "invalid".into(),
            params: Vec::new(),
            result: view(),
            body: escaping,
        });
        assert_eq!(check(&function).unwrap_err()[0].code, "E303");
    }

    #[test]
    pub(crate) fn ancestor_borrows_require_result_contracts() {
        let nested = block(
            1,
            vec![Stmt::Emit {
                target: 1,
                field: None,
                value: borrow(),
            }],
        );
        let program = program(vec![
            owner(),
            Stmt::Expr(expr(ExprKind::Block(nested), view())),
        ]);
        assert_eq!(check(&program).unwrap_err()[0].code, "B001");
    }

    #[test]
    pub(crate) fn an_outer_emission_cannot_retain_an_iteration_local() {
        let nested = block(
            1,
            vec![
                owner(),
                Stmt::Emit {
                    target: 0,
                    field: None,
                    value: borrow(),
                },
                Stmt::Restart(1),
            ],
        );
        let program = program(vec![Stmt::Expr(expr(ExprKind::Block(nested), Type::Null))]);
        assert_eq!(check(&program).unwrap_err()[0].code, "B001");
    }

    #[test]
    pub(crate) fn control_transfers_skip_unreachable_borrow_emissions() {
        for transfer in [Stmt::Leave(0), Stmt::Restart(0)] {
            let nested = block(1, vec![transfer]);
            let program = program(vec![
                owner(),
                Stmt::Expr(expr(ExprKind::Block(nested), Type::Null)),
                Stmt::Emit {
                    target: 0,
                    field: None,
                    value: borrow(),
                },
            ]);
            assert!(check(&program).is_ok());
        }
    }

    #[test]
    pub(crate) fn function_state_does_not_reuse_previous_function_origins() {
        let mut program = program(Vec::new());
        program.functions = vec![
            Function {
                id: 0,
                name: "first".into(),
                params: Vec::new(),
                result: Type::Null,
                body: block(
                    1,
                    vec![
                        owner(),
                        Stmt::Bind {
                            id: 1,
                            value: borrow(),
                        },
                    ],
                ),
            },
            Function {
                id: 1,
                name: "second".into(),
                params: Vec::new(),
                result: Type::Null,
                body: block(2, vec![Stmt::Expr(local(1))]),
            },
        ];
        assert_eq!(check(&program).unwrap_err()[0].code, "B001");
    }

    #[test]
    pub(crate) fn source_compilation_checks_lexical_borrow_origins() {
        assert!(crate::compile("a:1;view:&a;{copy:view;value:*copy};value:*view").is_ok());
        for source in ["value:{a:1;->&a}", "make:(){a:1;view:&a;->view}"] {
            assert_eq!(crate::compile(source).unwrap_err()[0].code, "E303");
        }
        assert_eq!(
            crate::compile("a:1;value:{->&a}").unwrap_err()[0].code,
            "B001"
        );
    }

    #[test]
    pub(crate) fn discarded_and_guarded_emissions_require_result_origin_analysis() {
        for source in [
            "'loop { x:1; -> &x; 'loop.restart() }",
            "debug:@\"debug\"; { x:1; -> &x; debug.panic(\"stop\") }",
            "'outer { { x:1; -> &x; 'outer.leave() } }",
            "f:false; 'scope { x:1; | f | -> &x }",
            "f:true; 'scope { x:1; | f | -> &x }",
            "'scope { x:1; | true | -> &x }",
            "condition:(x<boolean>){->x}; | condition(false) | {x:1;->&x}",
        ] {
            let errors = crate::compile(source).unwrap_err();
            assert_eq!(errors[0].code, "B001", "{source}: {errors:?}");
        }
    }
}
