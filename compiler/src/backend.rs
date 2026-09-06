use crate::hir::{Block, BlockId, Expr, ExprKind, Program, Stmt, Type};
use std::collections::{BTreeSet, HashMap};
use std::ffi::c_char;
use std::path::Path;

pub const RUNTIME_ARCHIVE: &[u8] = include_bytes!(env!("MEOWY_RUNTIME_ARCHIVE"));
pub const CLANG: &str = env!("MEOWY_CLANG");
pub const LLD: &str = env!("MEOWY_LLD");
pub const LLVM_VERSION: &str = env!("MEOWY_LLVM_VERSION");

unsafe extern "C" {
    pub(crate) fn meowy_emit_object_v1(
        ir: *const c_char,
        size: usize,
        path: *const c_char,
        path_size: usize,
        release: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
}

pub fn emit_object(ir: &str, path: &Path, release: bool) -> Result<(), String> {
    let path = path.as_os_str().as_encoded_bytes();
    if path.contains(&0) {
        return Err("object path contains a null byte".into());
    }
    let mut error = [0u8; 16384];
    let status = unsafe {
        meowy_emit_object_v1(
            ir.as_ptr().cast(),
            ir.len(),
            path.as_ptr().cast(),
            path.len(),
            i32::from(release),
            error.as_mut_ptr().cast(),
            error.len(),
        )
    };
    if status == 0 {
        Ok(())
    } else {
        let size = error
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(error.len());
        Err(String::from_utf8_lossy(&error[..size]).into_owned())
    }
}

pub fn emit_ir(program: &Program) -> Result<String, String> {
    Generator::new(program).generate()
}

pub(crate) fn ir_type(ty: &Type) -> String {
    match ty {
        Type::Null | Type::Never => "i8".into(),
        Type::Bool => "i1".into(),
        Type::Int { bits, .. } => format!("i{bits}"),
        Type::Float { bits: 32 } => "float".into(),
        Type::Float { .. } => "double".into(),
        Type::String => "{ ptr, i64 }".into(),
        Type::Record { primary, fields } => {
            let mut types = vec![ir_type(primary)];
            types.extend(fields.iter().map(|(_, ty)| ir_type(ty)));
            format!("{{ {} }}", types.join(", "))
        }
        Type::Union(types) => format!("{{ i32, [{} x i64] }}", union_words(types)),
    }
}

pub(crate) fn union_words(types: &[Type]) -> usize {
    types
        .iter()
        .map(|ty| layout(ty).0)
        .max()
        .unwrap_or(0)
        .div_ceil(8)
}

pub(crate) fn layout(ty: &Type) -> (usize, usize) {
    match ty {
        Type::Null | Type::Never | Type::Bool => (1, 1),
        Type::Int { bits, .. } | Type::Float { bits } => {
            let size = (*bits as usize).div_ceil(8);
            (size, size)
        }
        Type::String => (16, 8),
        Type::Record { primary, fields } => {
            let mut size = 0usize;
            let mut align = 1usize;
            for ty in std::iter::once(primary.as_ref()).chain(fields.iter().map(|(_, ty)| ty)) {
                let (part, boundary) = layout(ty);
                size = size.next_multiple_of(boundary) + part;
                align = align.max(boundary);
            }
            (size.next_multiple_of(align), align)
        }
        Type::Union(types) => (8 + union_words(types) * 8, 8),
    }
}

#[derive(Clone)]
pub(crate) struct Destination {
    pub(crate) start: String,
    pub(crate) end: String,
    pub(crate) slot: String,
    pub(crate) ty: Type,
}

pub(crate) struct Generator<'a> {
    pub(crate) program: &'a Program,
    pub(crate) globals: Vec<String>,
    pub(crate) declarations: BTreeSet<String>,
    pub(crate) functions: Vec<String>,
    pub(crate) code: Vec<String>,
    pub(crate) slots: Vec<String>,
    pub(crate) locals: BTreeSet<usize>,
    pub(crate) blocks: HashMap<BlockId, Destination>,
    pub(crate) next: usize,
    pub(crate) ended: bool,
}

impl<'a> Generator<'a> {
    pub(crate) fn new(program: &'a Program) -> Self {
        Self {
            program,
            globals: Vec::new(),
            declarations: BTreeSet::new(),
            functions: Vec::new(),
            code: Vec::new(),
            slots: Vec::new(),
            locals: BTreeSet::new(),
            blocks: HashMap::new(),
            next: 0,
            ended: false,
        }
    }

    pub(crate) fn generate(mut self) -> Result<String, String> {
        for function in &self.program.functions {
            self.begin();
            let params: Vec<String> = function
                .params
                .iter()
                .enumerate()
                .map(|(index, id)| format!("{} %arg{index}", ir_type(&self.program.locals[*id])))
                .collect();
            for (index, id) in function.params.iter().enumerate() {
                self.local(*id);
                self.line(format!(
                    "store {} %arg{index}, ptr %local{id}",
                    ir_type(&self.program.locals[*id])
                ));
            }
            let value = self.block(&function.body)?;
            self.line(format!("ret {} {value}", ir_type(&function.result)));
            self.finish(format!(
                "define internal {} @meowy_fn_{}({})",
                ir_type(&function.result),
                function.id,
                params.join(", ")
            ));
        }
        self.begin();
        self.block(&self.program.body)?;
        self.line("ret i32 0".into());
        self.finish("define i32 @main()".into());
        let runtime = [
            "declare void @meowy_write_v1(i32, ptr, i64)",
            "declare void @meowy_int_v1(i32, i64)",
            "declare void @meowy_uint_v1(i32, i64)",
            "declare void @meowy_float_v1(i32, double, i32)",
            "declare void @meowy_bool_v1(i32, i1 zeroext)",
            "declare zeroext i1 @meowy_string_equal_v1(ptr, i64, ptr, i64)",
            "declare i32 @meowy_string_compare_v1(ptr, i64, ptr, i64)",
            "declare void @meowy_panic_v1() noreturn",
            "declare void @meowy_arithmetic_fail_v1() noreturn",
        ];
        Ok(format!(
            "target triple = \"x86_64-unknown-linux-gnu\"\n\n{}\n\n{}\n{}\n\n{}\n",
            self.globals.join("\n"),
            runtime.join("\n"),
            self.declarations.into_iter().collect::<Vec<_>>().join("\n"),
            self.functions.join("\n\n")
        ))
    }

    pub(crate) fn begin(&mut self) {
        self.code.clear();
        self.slots.clear();
        self.locals.clear();
        self.blocks.clear();
        self.ended = false;
    }

    pub(crate) fn finish(&mut self, signature: String) {
        self.functions.push(format!(
            "{signature} {{\nentry:\n{}\n{}\n}}",
            self.slots.join("\n"),
            self.code.join("\n")
        ));
    }

    pub(crate) fn name(&mut self, prefix: &str) -> String {
        let id = self.next;
        self.next += 1;
        format!("{prefix}{id}")
    }

    pub(crate) fn line(&mut self, line: String) {
        self.code.push(format!("  {line}"));
    }

    pub(crate) fn value(&mut self, operation: String) -> String {
        let name = format!("%{}", self.name("v"));
        self.line(format!("{name} = {operation}"));
        name
    }

    pub(crate) fn label(&mut self, label: &str) {
        self.code.push(format!("{label}:"));
        self.ended = false;
    }

    pub(crate) fn jump(&mut self, target: &str) {
        self.line(format!("br label %{target}"));
        self.ended = true;
    }

    pub(crate) fn branch(&mut self, condition: &str, yes: &str, no: &str) {
        self.line(format!("br i1 {condition}, label %{yes}, label %{no}"));
        self.ended = true;
    }

    pub(crate) fn slot(&mut self, ty: &Type) -> String {
        let name = format!("%{}", self.name("slot"));
        self.slots
            .push(format!("  {name} = alloca {}", ir_type(ty)));
        name
    }

    pub(crate) fn local(&mut self, id: usize) {
        if self.locals.insert(id) {
            self.slots.push(format!(
                "  %local{id} = alloca {}",
                ir_type(&self.program.locals[id])
            ));
        }
    }

    pub(crate) fn store_value(&mut self, ty: &Type, value: &str, ptr: &str) {
        let fields = match ty {
            Type::Record { primary, fields } => std::iter::once(primary.as_ref())
                .chain(fields.iter().map(|(_, ty)| ty))
                .cloned()
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        if !fields.is_empty() {
            for (index, field) in fields.iter().enumerate() {
                let part = self.value(format!("extractvalue {} {value}, {index}", ir_type(ty)));
                let dest = self.value(format!(
                    "getelementptr {}, ptr {ptr}, i32 0, i32 {index}",
                    ir_type(ty)
                ));
                self.store_value(field, &part, &dest);
            }
        } else if let Type::Union(types) = ty {
            let tag = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
            let payload = self.value(format!("extractvalue {} {value}, 1", ir_type(ty)));
            let dest = self.value(format!(
                "getelementptr {}, ptr {ptr}, i32 0, i32 1",
                ir_type(ty)
            ));
            self.line(format!("store i32 {tag}, ptr {ptr}"));
            self.line(format!(
                "store [{} x i64] {payload}, ptr {dest}",
                union_words(types)
            ));
        } else {
            self.line(format!("store {} {value}, ptr {ptr}", ir_type(ty)));
        }
    }

    pub(crate) fn union_cases<F>(
        &mut self,
        ty: &Type,
        value: &str,
        mut apply: F,
    ) -> Result<(), String>
    where
        F: FnMut(&mut Self, &Type, &str) -> Result<(), String>,
    {
        let Type::Union(types) = ty else {
            return Err("variant dispatch requires a union".into());
        };
        let slot = self.slot(ty);
        self.line(format!("store {} {value}, ptr {slot}", ir_type(ty)));
        let ptr = self.value(format!(
            "getelementptr {}, ptr {slot}, i32 0, i32 1",
            ir_type(ty)
        ));
        let tag = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
        let labels = types
            .iter()
            .map(|_| self.name("variant"))
            .collect::<Vec<_>>();
        let invalid = self.name("invalid_tag");
        let end = self.name("union_end");
        let cases = labels
            .iter()
            .enumerate()
            .map(|(index, label)| format!("i32 {index}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        self.line(format!("switch i32 {tag}, label %{invalid} [ {cases} ]"));
        self.ended = true;
        for (label, member) in labels.iter().zip(types) {
            self.label(label);
            apply(self, member, &ptr)?;
            if !self.ended {
                self.jump(&end);
            }
        }
        self.label(&invalid);
        self.line("unreachable".into());
        self.ended = true;
        self.label(&end);
        Ok(())
    }

    pub(crate) fn coerce(&mut self, from: &Type, to: &Type, value: &str) -> Result<String, String> {
        if from == to {
            return Ok(value.into());
        }
        if *from == Type::Never {
            return Ok("undef".into());
        }
        if let Type::Union(_) = from {
            if from.intersection(to) == Type::Never {
                return Err(format!("cannot convert {from:?} to {to:?}"));
            }
            let slot = self.slot(to);
            self.union_cases(from, value, |this, member, ptr| {
                if to.accepts(member) {
                    let value = this.value(format!("load {}, ptr {ptr}", ir_type(member)));
                    let value = this.coerce(member, to, &value)?;
                    this.line(format!("store {} {value}, ptr {slot}", ir_type(to)));
                } else {
                    this.line("unreachable".into());
                    this.ended = true;
                }
                Ok(())
            })?;
            return Ok(self.value(format!("load {}, ptr {slot}", ir_type(to))));
        }
        if let Type::Union(types) = to
            && let Some(tag) = types.iter().position(|ty| ty == from)
        {
            let slot = self.slot(to);
            self.line(format!("store {} zeroinitializer, ptr {slot}", ir_type(to)));
            let ptr = self.value(format!(
                "getelementptr {}, ptr {slot}, i32 0, i32 1",
                ir_type(to)
            ));
            self.store_value(from, value, &ptr);
            let value = self.value(format!("load {}, ptr {slot}", ir_type(to)));
            return Ok(self.value(format!("insertvalue {} {value}, i32 {tag}, 0", ir_type(to))));
        }
        Err(format!("cannot convert {from:?} to {to:?}"))
    }

    pub(crate) fn type_test(&mut self, from: &Type, to: &Type, value: &str) -> String {
        if to.accepts(from) {
            return "true".into();
        }
        let Type::Union(types) = from else {
            return "false".into();
        };
        let tag = self.value(format!("extractvalue {} {value}, 0", ir_type(from)));
        let mut result = "false".to_owned();
        for (index, member) in types.iter().enumerate() {
            if to.accepts(member) {
                let matched = self.value(format!("icmp eq i32 {tag}, {index}"));
                result = self.value(format!("or i1 {result}, {matched}"));
            }
        }
        result
    }

    pub(crate) fn block(&mut self, block: &Block) -> Result<String, String> {
        let start = self.name("block");
        let end = self.name("end");
        let slot = self.slot(&block.ty);
        self.blocks.insert(
            block.id,
            Destination {
                start: start.clone(),
                end: end.clone(),
                slot: slot.clone(),
                ty: block.ty.clone(),
            },
        );
        self.jump(&start);
        self.label(&start);
        self.line(format!(
            "store {} zeroinitializer, ptr {slot}",
            ir_type(&block.ty)
        ));
        self.statements(&block.stmts)?;
        if !self.ended {
            self.jump(&end);
        }
        self.label(&end);
        self.blocks.remove(&block.id);
        Ok(self.value(format!("load {}, ptr {slot}", ir_type(&block.ty))))
    }

    pub(crate) fn statements(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for statement in statements {
            if self.ended {
                break;
            }
            match statement {
                Stmt::Bind { id, value } | Stmt::Assign { id, value } => {
                    let result = self.expression(value)?;
                    if !self.ended {
                        self.local(*id);
                        let ty = &self.program.locals[*id];
                        let result = self.coerce(&value.ty, ty, &result)?;
                        self.line(format!("store {} {result}, ptr %local{id}", ir_type(ty)));
                    }
                }
                Stmt::Emit {
                    target,
                    field,
                    value,
                } => {
                    let result = self.expression(value)?;
                    if self.ended {
                        continue;
                    }
                    let destination = self
                        .blocks
                        .get(target)
                        .ok_or_else(|| format!("missing block emission target {target}"))?
                        .clone();
                    let ty = ir_type(&destination.ty);
                    let (index, slot) = match &destination.ty {
                        Type::Record { primary, fields } => {
                            let index = if let Some(field) = field {
                                let Some(index) = fields.iter().position(|(name, _)| name == field)
                                else {
                                    continue;
                                };
                                index + 1
                            } else {
                                0
                            };
                            let slot = if index == 0 {
                                primary.as_ref()
                            } else {
                                &fields[index - 1].1
                            };
                            (Some(index), slot)
                        }
                        _ if field.is_none() => (None, &destination.ty),
                        _ => continue,
                    };
                    if !slot.accepts(&value.ty) {
                        continue;
                    }
                    let result = self.coerce(&value.ty, slot, &result)?;
                    let result = if let Some(index) = index {
                        let current = self.value(format!("load {ty}, ptr {}", destination.slot));
                        self.value(format!(
                            "insertvalue {ty} {current}, {} {result}, {index}",
                            ir_type(slot)
                        ))
                    } else {
                        result
                    };
                    self.line(format!("store {ty} {result}, ptr {}", destination.slot));
                }
                Stmt::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    let condition = self.expression(condition)?;
                    if self.ended {
                        continue;
                    }
                    let yes = self.name("then");
                    let no = self.name("else");
                    let end = self.name("join");
                    self.branch(&condition, &yes, &no);
                    self.label(&yes);
                    self.statements(then)?;
                    let yes_ended = self.ended;
                    if !yes_ended {
                        self.jump(&end);
                    }
                    self.label(&no);
                    self.statements(otherwise)?;
                    let no_ended = self.ended;
                    if !no_ended {
                        self.jump(&end);
                    }
                    if !yes_ended || !no_ended {
                        self.label(&end);
                    }
                }
                Stmt::Leave(target) | Stmt::Restart(target) => {
                    let target = self
                        .blocks
                        .get(target)
                        .ok_or_else(|| format!("missing control target {target}"))?;
                    let label = if matches!(statement, Stmt::Leave(_)) {
                        &target.end
                    } else {
                        &target.start
                    }
                    .clone();
                    self.jump(&label);
                }
                Stmt::Expr(value) => {
                    self.expression(value)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn string(&mut self, value: &str) -> String {
        let name = self.name("str");
        let bytes = value
            .as_bytes()
            .iter()
            .map(|byte| format!("\\{byte:02X}"))
            .collect::<String>();
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{bytes}\", align 1",
            value.len()
        ));
        format!("{{ ptr @{name}, i64 {} }}", value.len())
    }

    pub(crate) fn expression(&mut self, expression: &Expr) -> Result<String, String> {
        let ty = ir_type(&expression.ty);
        match &expression.kind {
            ExprKind::Null => Ok("0".into()),
            ExprKind::Bool(value) => Ok(value.to_string()),
            ExprKind::Int(value) => Ok(value.to_string()),
            ExprKind::Float(value) => {
                let value = if matches!(expression.ty, Type::Float { bits: 32 }) {
                    f64::from(*value as f32)
                } else {
                    *value
                };
                Ok(format!("0x{:016X}", value.to_bits()))
            }
            ExprKind::String(value) => Ok(self.string(value)),
            ExprKind::Local(id) => {
                self.local(*id);
                let stored = &self.program.locals[*id];
                let value = self.value(format!("load {}, ptr %local{id}", ir_type(stored)));
                self.coerce(stored, &expression.ty, &value)
            }
            ExprKind::Unary { op, value } => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                match (op.as_str(), &expression.ty) {
                    ("+", _) => Ok(result),
                    ("!", Type::Bool) => Ok(self.value(format!("xor i1 {result}, true"))),
                    ("~", Type::Int { .. }) => Ok(self.value(format!("xor {ty} {result}, -1"))),
                    ("-", Type::Int { bits, signed }) => {
                        Ok(self.checked("sub", *bits, *signed, "0", &result))
                    }
                    ("-", Type::Float { .. }) => Ok(self.value(format!("fneg {ty} {result}"))),
                    _ => Err(format!("unsupported checked unary operator {op}")),
                }
            }
            ExprKind::Binary { op, left, right } => self.binary(op, left, right),
            ExprKind::Call { id, args } => {
                let mut values = Vec::new();
                for arg in args {
                    let result = self.expression(arg)?;
                    if self.ended {
                        return Ok("undef".into());
                    }
                    values.push(format!("{} {result}", ir_type(&arg.ty)));
                }
                Ok(self.value(format!("call {ty} @meowy_fn_{id}({})", values.join(", "))))
            }
            ExprKind::Print { parts, newline } => {
                self.print(parts, 1)?;
                if *newline && !self.ended {
                    let newline = self.string("\n");
                    self.print_value(&Type::String, &newline, 1)?;
                }
                Ok("0".into())
            }
            ExprKind::Panic { parts } => {
                let prefix = self.string("panic[P006]: ");
                self.print_value(&Type::String, &prefix, 2)?;
                self.print(parts, 2)?;
                if !self.ended {
                    self.line("call void @meowy_panic_v1()".into());
                    self.line("unreachable".into());
                    self.ended = true;
                }
                Ok("undef".into())
            }
            ExprKind::Block(block) => self.block(block),
            ExprKind::Field { value, index } => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                Ok(self.value(format!(
                    "extractvalue {} {result}, {}",
                    ir_type(&value.ty),
                    index + 1
                )))
            }
            ExprKind::Primary(value) => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                if matches!(value.ty, Type::Record { .. }) {
                    Ok(self.value(format!("extractvalue {} {result}, 0", ir_type(&value.ty))))
                } else {
                    Ok(result)
                }
            }
            ExprKind::StringSize(value) => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                Ok(self.value(format!("extractvalue {{ ptr, i64 }} {result}, 1")))
            }
            ExprKind::Coerce { value } => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                self.coerce(&value.ty, &expression.ty, &result)
            }
            ExprKind::TypeTest { value, ty } => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                Ok(self.type_test(&value.ty, ty, &result))
            }
        }
    }

    pub(crate) fn checked(
        &mut self,
        op: &str,
        bits: u32,
        signed: bool,
        left: &str,
        right: &str,
    ) -> String {
        let prefix = if signed { "s" } else { "u" };
        let name = format!("llvm.{prefix}{op}.with.overflow.i{bits}");
        self.declarations.insert(format!(
            "declare {{ i{bits}, i1 }} @{name}(i{bits}, i{bits})"
        ));
        let pair = self.value(format!(
            "call {{ i{bits}, i1 }} @{name}(i{bits} {left}, i{bits} {right})"
        ));
        let overflow = self.value(format!("extractvalue {{ i{bits}, i1 }} {pair}, 1"));
        self.guard(&overflow);
        self.value(format!("extractvalue {{ i{bits}, i1 }} {pair}, 0"))
    }

    pub(crate) fn guard(&mut self, failure: &str) {
        let fail = self.name("fail");
        let next = self.name("ok");
        self.branch(failure, &fail, &next);
        self.label(&fail);
        self.line("call void @meowy_arithmetic_fail_v1()".into());
        self.line("unreachable".into());
        self.label(&next);
    }

    pub(crate) fn binary(&mut self, op: &str, left: &Expr, right: &Expr) -> Result<String, String> {
        let first = self.expression(left)?;
        if self.ended {
            return Ok("undef".into());
        }
        if op == "&&" || op == "||" {
            let slot = self.slot(&Type::Bool);
            self.line(format!("store i1 {first}, ptr {slot}"));
            let rhs = self.name("rhs");
            let end = self.name("bool_end");
            if op == "&&" {
                self.branch(&first, &rhs, &end);
            } else {
                self.branch(&first, &end, &rhs);
            }
            self.label(&rhs);
            let second = self.expression(right)?;
            if !self.ended {
                self.line(format!("store i1 {second}, ptr {slot}"));
                self.jump(&end);
            }
            self.label(&end);
            return Ok(self.value(format!("load i1, ptr {slot}")));
        }
        let second = self.expression(right)?;
        if self.ended {
            return Ok("undef".into());
        }
        let ty = ir_type(&left.ty);
        if op == "==" || op == "!=" {
            let equal = self.equal(&left.ty, &first, &second)?;
            return if op == "==" {
                Ok(equal)
            } else {
                Ok(self.value(format!("xor i1 {equal}, true")))
            };
        }
        match &left.ty {
            Type::Int { bits, signed } => {
                let mut divisor = second.clone();
                let operation = match op {
                    "+" => return Ok(self.checked("add", *bits, *signed, &first, &second)),
                    "-" => return Ok(self.checked("sub", *bits, *signed, &first, &second)),
                    "*" => return Ok(self.checked("mul", *bits, *signed, &first, &second)),
                    "/" | "%" => {
                        let zero = self.value(format!("icmp eq {ty} {second}, 0"));
                        let failure = if *signed {
                            let minimum = -(1i128 << (bits - 1));
                            let minimum = self.value(format!("icmp eq {ty} {first}, {minimum}"));
                            let negative = self.value(format!("icmp eq {ty} {second}, -1"));
                            let overflow = self.value(format!("and i1 {minimum}, {negative}"));
                            if op == "%" {
                                divisor = self
                                    .value(format!("select i1 {overflow}, {ty} 1, {ty} {second}"));
                                zero
                            } else {
                                self.value(format!("or i1 {zero}, {overflow}"))
                            }
                        } else {
                            zero
                        };
                        self.guard(&failure);
                        match (op, signed) {
                            ("/", true) => "sdiv",
                            ("/", false) => "udiv",
                            (_, true) => "srem",
                            (_, false) => "urem",
                        }
                    }
                    "&" => "and",
                    "|" => "or",
                    "^" => "xor",
                    "<" | "<=" | ">" | ">=" => {
                        let prefix = if *signed { "s" } else { "u" };
                        let comparison = match op {
                            "<" => "lt",
                            "<=" => "le",
                            ">" => "gt",
                            _ => "ge",
                        };
                        return Ok(
                            self.value(format!("icmp {prefix}{comparison} {ty} {first}, {second}"))
                        );
                    }
                    _ => return Err(format!("unsupported checked integer operator {op}")),
                };
                Ok(self.value(format!("{operation} {ty} {first}, {divisor}")))
            }
            Type::Float { .. } => {
                let operation = match op {
                    "+" => "fadd",
                    "-" => "fsub",
                    "*" => "fmul",
                    "/" => "fdiv",
                    "%" => "frem",
                    "<" | "<=" | ">" | ">=" => {
                        let comparison = match op {
                            "<" => "olt",
                            "<=" => "ole",
                            ">" => "ogt",
                            _ => "oge",
                        };
                        return Ok(self.value(format!("fcmp {comparison} {ty} {first}, {second}")));
                    }
                    _ => return Err(format!("unsupported checked float operator {op}")),
                };
                Ok(self.value(format!("{operation} {ty} {first}, {second}")))
            }
            Type::String if matches!(op, "<" | "<=" | ">" | ">=") => {
                let a = self.value(format!("extractvalue {{ ptr, i64 }} {first}, 0"));
                let size_a = self.value(format!("extractvalue {{ ptr, i64 }} {first}, 1"));
                let b = self.value(format!("extractvalue {{ ptr, i64 }} {second}, 0"));
                let size_b = self.value(format!("extractvalue {{ ptr, i64 }} {second}, 1"));
                let order = self.value(format!("call i32 @meowy_string_compare_v1(ptr {a}, i64 {size_a}, ptr {b}, i64 {size_b})"));
                let comparison = match op {
                    "<" => "slt",
                    "<=" => "sle",
                    ">" => "sgt",
                    _ => "sge",
                };
                Ok(self.value(format!("icmp {comparison} i32 {order}, 0")))
            }
            _ => Err(format!(
                "unsupported checked binary operator {op} on {:?}",
                left.ty
            )),
        }
    }

    pub(crate) fn equal(&mut self, ty: &Type, left: &str, right: &str) -> Result<String, String> {
        match ty {
            Type::Null => Ok("true".into()),
            Type::Never => Err("cannot compare a never value".into()),
            Type::Bool | Type::Int { .. } => {
                Ok(self.value(format!("icmp eq {} {left}, {right}", ir_type(ty))))
            }
            Type::Float { .. } => {
                Ok(self.value(format!("fcmp oeq {} {left}, {right}", ir_type(ty))))
            }
            Type::String => {
                let a = self.value(format!("extractvalue {{ ptr, i64 }} {left}, 0"));
                let size_a = self.value(format!("extractvalue {{ ptr, i64 }} {left}, 1"));
                let b = self.value(format!("extractvalue {{ ptr, i64 }} {right}, 0"));
                let size_b = self.value(format!("extractvalue {{ ptr, i64 }} {right}, 1"));
                Ok(self.value(format!("call zeroext i1 @meowy_string_equal_v1(ptr {a}, i64 {size_a}, ptr {b}, i64 {size_b})")))
            }
            Type::Record { primary, fields } => {
                let types =
                    std::iter::once(primary.as_ref()).chain(fields.iter().map(|(_, ty)| ty));
                let mut result = "true".to_owned();
                for (index, field) in types.enumerate() {
                    let a = self.value(format!("extractvalue {} {left}, {index}", ir_type(ty)));
                    let b = self.value(format!("extractvalue {} {right}, {index}", ir_type(ty)));
                    let equal = self.equal(field, &a, &b)?;
                    result = self.value(format!("and i1 {result}, {equal}"));
                }
                Ok(result)
            }
            Type::Union(_) => {
                let slot = self.slot(&Type::Bool);
                self.line(format!("store i1 false, ptr {slot}"));
                let a = self.value(format!("extractvalue {} {left}, 0", ir_type(ty)));
                let b = self.value(format!("extractvalue {} {right}, 0", ir_type(ty)));
                let same = self.value(format!("icmp eq i32 {a}, {b}"));
                let compare = self.name("union_equal");
                let end = self.name("equal_end");
                self.branch(&same, &compare, &end);
                self.label(&compare);
                let other = self.slot(ty);
                self.line(format!("store {} {right}, ptr {other}", ir_type(ty)));
                let other = self.value(format!(
                    "getelementptr {}, ptr {other}, i32 0, i32 1",
                    ir_type(ty)
                ));
                self.union_cases(ty, left, |this, member, ptr| {
                    let a = this.value(format!("load {}, ptr {ptr}", ir_type(member)));
                    let b = this.value(format!("load {}, ptr {other}", ir_type(member)));
                    let result = this.equal(member, &a, &b)?;
                    this.line(format!("store i1 {result}, ptr {slot}"));
                    Ok(())
                })?;
                self.jump(&end);
                self.label(&end);
                Ok(self.value(format!("load i1, ptr {slot}")))
            }
        }
    }

    pub(crate) fn print(&mut self, parts: &[Expr], fd: i32) -> Result<(), String> {
        for part in parts {
            let value = self.expression(part)?;
            if self.ended {
                break;
            }
            self.print_value(&part.ty, &value, fd)?;
        }
        Ok(())
    }

    pub(crate) fn print_value(&mut self, ty: &Type, value: &str, fd: i32) -> Result<(), String> {
        match ty {
            Type::Null => {
                let value = self.string("null");
                self.print_value(&Type::String, &value, fd)?;
            }
            Type::Never => return Err("cannot print a never value".into()),
            Type::Bool => self.line(format!(
                "call void @meowy_bool_v1(i32 {fd}, i1 zeroext {value})"
            )),
            Type::Int { bits, signed } => {
                let value = if *bits < 64 {
                    self.value(format!(
                        "{} i{bits} {value} to i64",
                        if *signed { "sext" } else { "zext" }
                    ))
                } else {
                    value.to_owned()
                };
                let name = if *signed { "int" } else { "uint" };
                self.line(format!("call void @meowy_{name}_v1(i32 {fd}, i64 {value})"));
            }
            Type::Float { bits } => {
                let value = if *bits == 32 {
                    self.value(format!("fpext float {value} to double"))
                } else {
                    value.to_owned()
                };
                self.line(format!(
                    "call void @meowy_float_v1(i32 {fd}, double {value}, i32 {bits})"
                ));
            }
            Type::String => {
                let pointer = self.value(format!("extractvalue {{ ptr, i64 }} {value}, 0"));
                let size = self.value(format!("extractvalue {{ ptr, i64 }} {value}, 1"));
                self.line(format!(
                    "call void @meowy_write_v1(i32 {fd}, ptr {pointer}, i64 {size})"
                ));
            }
            Type::Record { primary, .. } => {
                let value = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
                self.print_value(primary, &value, fd)?;
            }
            Type::Union(_) => {
                self.union_cases(ty, value, |this, member, ptr| {
                    let value = this.value(format!("load {}, ptr {ptr}", ir_type(member)));
                    this.print_value(member, &value, fd)
                })?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;
    use std::process::{Command, Output, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub(crate) static NEXT: AtomicUsize = AtomicUsize::new(0);

    pub(crate) fn expr(kind: ExprKind, ty: Type) -> Expr {
        Expr {
            kind,
            ty,
            span: Span::default(),
        }
    }

    pub(crate) fn integer(value: i128, bits: u32, signed: bool) -> Expr {
        expr(ExprKind::Int(value), Type::Int { bits, signed })
    }

    pub(crate) fn binary(op: &str, left: Expr, right: Expr, ty: Type) -> Expr {
        expr(
            ExprKind::Binary {
                op: op.into(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
        )
    }

    pub(crate) fn native(parts: Vec<Expr>, release: bool, full: bool) -> Output {
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![Stmt::Expr(expr(
                    ExprKind::Print {
                        parts,
                        newline: true,
                    },
                    Type::Null,
                ))],
            },
            functions: Vec::new(),
            locals: Vec::new(),
        };
        native_program(&program, release, full)
    }

    pub(crate) fn native_program(program: &Program, release: bool, full: bool) -> Output {
        let dir = std::env::temp_dir().join(format!(
            "meowy-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        let ir = emit_ir(program).unwrap();
        let object = dir.join("main.o");
        emit_object(&ir, &object, release).unwrap_or_else(|error| panic!("{error}\n{ir}"));
        let runtime = dir.join("runtime.a");
        std::fs::write(&runtime, RUNTIME_ARCHIVE).unwrap();
        let executable = dir.join("main");
        let link = Command::new(CLANG)
            .arg(format!("-fuse-ld={LLD}"))
            .arg("-Wl,--gc-sections")
            .arg(&object)
            .arg(&runtime)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            link.status.success(),
            "{}",
            String::from_utf8_lossy(&link.stderr)
        );
        let mut command = Command::new(&executable);
        if full {
            command.stdout(Stdio::from(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            ));
        }
        let output = command.output().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
        output
    }

    #[test]
    pub(crate) fn invalid_ir_returns_a_bridge_error() {
        let path = std::env::temp_dir().join(format!(
            "meowy-invalid-{}-{}.o",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let error = emit_object("invalid LLVM", &path, false).unwrap_err();
        assert!(error.contains("LLVM IR parser"));
        assert!(!path.exists());
    }

    #[test]
    pub(crate) fn invalid_ssa_is_rejected_before_object_output() {
        let path = std::env::temp_dir().join(format!(
            "meowy-invalid-ssa-{}-{}.o",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let error = emit_object(
            "define i32 @main() { entry: ret i32 %value dead: %value = add i32 1, 2 ret i32 0 }",
            &path,
            false,
        )
        .unwrap_err();
        assert!(error.contains("LLVM verification"));
        assert!(!path.exists());
    }

    #[test]
    pub(crate) fn object_is_an_x86_64_elf_in_each_profile() {
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: Vec::new(),
            },
            functions: Vec::new(),
            locals: Vec::new(),
        };
        let ir = emit_ir(&program).unwrap();
        let size = ir.len();
        let bounded = format!("{ir}invalid trailing bytes");
        for release in [false, true] {
            let path = std::env::temp_dir().join(format!(
                "meowy-object-{}-{}.o",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            emit_object(&bounded[..size], &path, release).unwrap();
            let object = std::fs::read(&path).unwrap();
            assert_eq!(&object[..4], b"\x7fELF");
            assert_eq!(object[4], 2);
            assert_eq!(u16::from_le_bytes([object[18], object[19]]), 62);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    pub(crate) fn integer_boundaries_fail_in_each_profile() {
        for release in [false, true] {
            for bits in [8, 16, 32, 64] {
                for signed in [false, true] {
                    let ty = Type::Int { bits, signed };
                    let maximum = (1i128 << (bits - u32::from(signed))) - 1;
                    let value = binary(
                        "+",
                        integer(maximum, bits, signed),
                        integer(1, bits, signed),
                        ty,
                    );
                    let output = native(vec![value], release, false);
                    assert_eq!(
                        output.status.code(),
                        Some(1),
                        "bits={bits}, signed={signed}, release={release}"
                    );
                    assert!(output.stderr.starts_with(b"panic[P002]: integer overflow"));
                }
            }
            let ty = Type::Int {
                bits: 64,
                signed: true,
            };
            for op in ["/", "%"] {
                let value = binary(
                    op,
                    integer(i64::MIN.into(), 64, true),
                    integer(-1, 64, true),
                    ty.clone(),
                );
                let output = native(vec![value], release, false);
                if op == "/" {
                    assert_eq!(output.status.code(), Some(1));
                } else {
                    assert!(output.status.success());
                    assert_eq!(output.stdout, b"0\n");
                }
                let value = binary(op, integer(8, 64, true), integer(0, 64, true), ty.clone());
                assert_eq!(native(vec![value], release, false).status.code(), Some(1));
            }
            let minimum = expr(
                ExprKind::Unary {
                    op: "-".into(),
                    value: Box::new(integer(i64::MIN.into(), 64, true)),
                },
                ty,
            );
            assert_eq!(native(vec![minimum], release, false).status.code(), Some(1));
        }
    }

    #[test]
    pub(crate) fn strings_numbers_and_short_circuit_execute_in_each_profile() {
        for release in [false, true] {
            let ty = Type::Int {
                bits: 64,
                signed: true,
            };
            let divide = binary("/", integer(1, 64, true), integer(0, 64, true), ty.clone());
            let danger = binary("==", divide, integer(0, 64, true), Type::Bool);
            let short = binary(
                "&&",
                expr(ExprKind::Bool(false), Type::Bool),
                danger.clone(),
                Type::Bool,
            );
            let other = binary(
                "||",
                expr(ExprKind::Bool(true), Type::Bool),
                danger,
                Type::Bool,
            );
            let order = binary(
                ">",
                expr(ExprKind::String("é".into()), Type::String),
                expr(ExprKind::String("z".into()), Type::String),
                Type::Bool,
            );
            let equal = binary(
                "==",
                expr(ExprKind::String("a\0b".into()), Type::String),
                expr(ExprKind::String("a\0b".into()), Type::String),
                Type::Bool,
            );
            let parts = vec![
                short,
                other,
                order,
                equal,
                binary("/", integer(-7, 64, true), integer(3, 64, true), ty.clone()),
                binary("%", integer(-7, 64, true), integer(3, 64, true), ty),
                integer(u64::MAX.into(), 64, false),
                expr(ExprKind::String("猫\0".into()), Type::String),
                expr(ExprKind::Float(1.25), Type::Float { bits: 32 }),
                expr(ExprKind::Float(-0.0), Type::Float { bits: 64 }),
            ];
            let output = native(parts, release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.stdout,
                "falsetruetruetrue-2-118446744073709551615猫\x001.25-0\n".as_bytes()
            );
        }
    }

    #[test]
    pub(crate) fn output_failure_is_not_success() {
        let output = native(
            vec![expr(ExprKind::String("hello".into()), Type::String)],
            false,
            true,
        );
        assert_eq!(output.status.code(), Some(1));
    }

    pub(crate) fn coerce(value: Expr, ty: &Type) -> Expr {
        expr(
            ExprKind::Coerce {
                value: Box::new(value),
            },
            ty.clone(),
        )
    }

    pub(crate) fn type_test(value: Expr, ty: Type) -> Expr {
        expr(
            ExprKind::TypeTest {
                value: Box::new(value),
                ty,
            },
            Type::Bool,
        )
    }

    pub(crate) fn record(id: usize, primary: Expr, field: Expr) -> Expr {
        let ty = Type::Record {
            primary: Box::new(primary.ty.clone()),
            fields: vec![("field".into(), field.ty.clone())],
        };
        expr(
            ExprKind::Block(Block {
                id,
                ty: ty.clone(),
                stmts: vec![
                    Stmt::Emit {
                        target: id,
                        field: None,
                        value: primary,
                    },
                    Stmt::Emit {
                        target: id,
                        field: Some("field".into()),
                        value: field,
                    },
                ],
            }),
            ty,
        )
    }

    #[test]
    pub(crate) fn union_scalars_retag_and_extract_in_each_profile() {
        let small = Type::union([Type::Null, Type::String]);
        let wide = Type::union([
            Type::Null,
            Type::Bool,
            Type::Int {
                bits: 8,
                signed: true,
            },
            Type::Int {
                bits: 64,
                signed: false,
            },
            Type::Float { bits: 32 },
            Type::Float { bits: 64 },
            Type::String,
        ]);
        let text = expr(ExprKind::String("猫\0".into()), Type::String);
        let value = coerce(coerce(text, &small), &wide);
        for release in [false, true] {
            let output = native(
                vec![
                    coerce(expr(ExprKind::Null, Type::Null), &wide),
                    coerce(expr(ExprKind::Bool(true), Type::Bool), &wide),
                    coerce(integer(-128, 8, true), &wide),
                    coerce(integer(u64::MAX.into(), 64, false), &wide),
                    coerce(expr(ExprKind::Float(1.25), Type::Float { bits: 32 }), &wide),
                    coerce(expr(ExprKind::Float(-0.0), Type::Float { bits: 64 }), &wide),
                    type_test(value.clone(), Type::String),
                    type_test(value.clone(), Type::Null),
                    coerce(coerce(value.clone(), &small), &Type::String),
                ],
                release,
                false,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.stdout,
                "nulltrue-128184467440737095516151.25-0truefalse猫\0\n".as_bytes()
            );
        }
    }

    #[test]
    pub(crate) fn union_records_preserve_padding_and_compare_active_payloads() {
        let field = Type::union([Type::Null, Type::String]);
        let first = record(
            1,
            integer(7, 8, false),
            coerce(expr(ExprKind::String("one".into()), Type::String), &field),
        );
        let other = record(
            2,
            integer(7, 8, false),
            coerce(expr(ExprKind::String("two".into()), Type::String), &field),
        );
        let nested = record(3, integer(9, 8, false), first.clone());
        let ty = Type::union([
            Type::Null,
            Type::String,
            first.ty.clone(),
            nested.ty.clone(),
        ]);
        let a = coerce(first.clone(), &ty);
        let b = coerce(other, &ty);
        let none = coerce(expr(ExprKind::Null, Type::Null), &ty);
        let text = coerce(expr(ExprKind::String("hello".into()), Type::String), &ty);
        for release in [false, true] {
            let output = native(
                vec![
                    a.clone(),
                    coerce(nested.clone(), &ty),
                    type_test(a.clone(), Type::Null),
                    type_test(a.clone(), first.ty.clone()),
                    binary("==", a.clone(), a.clone(), Type::Bool),
                    binary("==", a.clone(), b.clone(), Type::Bool),
                    binary("==", none.clone(), a.clone(), Type::Bool),
                    binary("==", none.clone(), text.clone(), Type::Bool),
                    binary("==", none.clone(), none.clone(), Type::Bool),
                    binary("==", text.clone(), text.clone(), Type::Bool),
                ],
                release,
                false,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"79falsetruetruefalsefalsefalsetruetrue\n");
        }
    }

    #[test]
    pub(crate) fn inferred_union_emissions_and_nullable_defaults_execute() {
        let ty = Type::union([Type::Null, Type::String]);
        let shape = Type::Record {
            primary: Box::new(ty.clone()),
            fields: vec![("name".into(), ty.clone())],
        };
        let value = |id, present| {
            expr(
                ExprKind::Block(Block {
                    id,
                    ty: shape.clone(),
                    stmts: vec![Stmt::If {
                        condition: expr(ExprKind::Bool(present), Type::Bool),
                        then: vec![Stmt::Emit {
                            target: id,
                            field: Some("name".into()),
                            value: expr(ExprKind::String("hello".into()), Type::String),
                        }],
                        otherwise: Vec::new(),
                    }],
                }),
                shape.clone(),
            )
        };
        for release in [false, true] {
            let output = native(
                vec![
                    expr(
                        ExprKind::Field {
                            value: Box::new(value(1, true)),
                            index: 0,
                        },
                        ty.clone(),
                    ),
                    expr(
                        ExprKind::Field {
                            value: Box::new(value(2, false)),
                            index: 0,
                        },
                        ty.clone(),
                    ),
                    value(3, true),
                    expr(
                        ExprKind::Block(Block {
                            id: 4,
                            ty: ty.clone(),
                            stmts: vec![Stmt::Emit {
                                target: 4,
                                field: None,
                                value: expr(ExprKind::String("primary".into()), Type::String),
                            }],
                        }),
                        ty.clone(),
                    ),
                ],
                release,
                false,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"hellonullnullprimary\n");
        }
    }

    #[test]
    pub(crate) fn union_storage_and_restart_defaults_execute() {
        let ty = Type::union([Type::Null, Type::String]);
        let shape = Type::Record {
            primary: Box::new(Type::Null),
            fields: vec![("name".into(), ty.clone())],
        };
        let block = expr(
            ExprKind::Block(Block {
                id: 1,
                ty: shape.clone(),
                stmts: vec![Stmt::If {
                    condition: expr(
                        ExprKind::Unary {
                            op: "!".into(),
                            value: Box::new(expr(ExprKind::Local(1), Type::Bool)),
                        },
                        Type::Bool,
                    ),
                    then: vec![
                        Stmt::Emit {
                            target: 1,
                            field: Some("name".into()),
                            value: expr(ExprKind::String("discarded".into()), Type::String),
                        },
                        Stmt::Assign {
                            id: 1,
                            value: expr(ExprKind::Bool(true), Type::Bool),
                        },
                        Stmt::Restart(1),
                    ],
                    otherwise: Vec::new(),
                }],
            }),
            shape,
        );
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![
                    Stmt::Bind {
                        id: 0,
                        value: expr(ExprKind::String("stored".into()), Type::String),
                    },
                    Stmt::Bind {
                        id: 1,
                        value: expr(ExprKind::Bool(false), Type::Bool),
                    },
                    Stmt::Expr(expr(
                        ExprKind::Print {
                            parts: vec![
                                expr(ExprKind::Local(0), Type::String),
                                expr(ExprKind::Local(0), ty.clone()),
                                expr(
                                    ExprKind::Field {
                                        value: Box::new(block),
                                        index: 0,
                                    },
                                    ty.clone(),
                                ),
                            ],
                            newline: true,
                        },
                        Type::Null,
                    )),
                    Stmt::Assign {
                        id: 0,
                        value: expr(ExprKind::Null, Type::Null),
                    },
                    Stmt::Expr(expr(
                        ExprKind::Print {
                            parts: vec![expr(ExprKind::Local(0), ty.clone())],
                            newline: true,
                        },
                        Type::Null,
                    )),
                ],
            },
            functions: Vec::new(),
            locals: vec![ty, Type::Bool],
        };
        for release in [false, true] {
            let output = native_program(&program, release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"storedstorednull\nnull\n");
        }
    }

    #[test]
    pub(crate) fn coercions_and_type_tests_evaluate_operands_once() {
        let ty = Type::union([Type::Null, Type::String]);
        let value = expr(
            ExprKind::Block(Block {
                id: 1,
                ty: Type::String,
                stmts: vec![
                    Stmt::Expr(expr(
                        ExprKind::Print {
                            parts: vec![expr(ExprKind::String("once:".into()), Type::String)],
                            newline: false,
                        },
                        Type::Null,
                    )),
                    Stmt::Emit {
                        target: 1,
                        field: None,
                        value: expr(ExprKind::String("value".into()), Type::String),
                    },
                ],
            }),
            Type::String,
        );
        for release in [false, true] {
            let output = native(
                vec![
                    type_test(coerce(value.clone(), &ty), Type::String),
                    type_test(value.clone(), Type::Null),
                ],
                release,
                false,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"once:trueonce:false\n");
        }
    }

    #[test]
    pub(crate) fn union_function_arguments_and_results_execute() {
        let small = Type::union([Type::Null, Type::String]);
        let wide = Type::union([Type::Null, Type::Bool, Type::String]);
        let call = |value| {
            expr(
                ExprKind::Call {
                    id: 0,
                    args: vec![coerce(value, &small)],
                },
                wide.clone(),
            )
        };
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![Stmt::Expr(expr(
                    ExprKind::Print {
                        parts: vec![
                            call(expr(ExprKind::String("argument".into()), Type::String)),
                            call(expr(ExprKind::Null, Type::Null)),
                            coerce(
                                call(expr(ExprKind::String("result".into()), Type::String)),
                                &Type::String,
                            ),
                        ],
                        newline: true,
                    },
                    Type::Null,
                ))],
            },
            functions: vec![crate::hir::Function {
                id: 0,
                name: "widen".into(),
                params: vec![0],
                result: wide.clone(),
                body: Block {
                    id: 1,
                    ty: wide,
                    stmts: vec![Stmt::Emit {
                        target: 1,
                        field: None,
                        value: expr(ExprKind::Local(0), small.clone()),
                    }],
                },
            }],
            locals: vec![small],
        };
        for release in [false, true] {
            let output = native_program(&program, release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"argumentnullresult\n");
        }
    }

    #[test]
    pub(crate) fn discarded_emission_slots_preserve_operand_effects() {
        let effect = |id, text: &str| {
            expr(
                ExprKind::Block(Block {
                    id,
                    ty: Type::String,
                    stmts: vec![
                        Stmt::Expr(expr(
                            ExprKind::Print {
                                parts: vec![expr(ExprKind::String(text.into()), Type::String)],
                                newline: false,
                            },
                            Type::Null,
                        )),
                        Stmt::Emit {
                            target: id,
                            field: None,
                            value: expr(ExprKind::String("discarded".into()), Type::String),
                        },
                    ],
                }),
                Type::String,
            )
        };
        let ty = Type::Int {
            bits: 32,
            signed: true,
        };
        let value = expr(
            ExprKind::Block(Block {
                id: 1,
                ty: ty.clone(),
                stmts: vec![
                    Stmt::If {
                        condition: expr(
                            ExprKind::Unary {
                                op: "!".into(),
                                value: Box::new(expr(ExprKind::Local(0), Type::Bool)),
                            },
                            Type::Bool,
                        ),
                        then: vec![
                            Stmt::Emit {
                                target: 1,
                                field: None,
                                value: coerce(
                                    effect(2, "primary:"),
                                    &Type::union([Type::Bool, Type::String]),
                                ),
                            },
                            Stmt::Emit {
                                target: 1,
                                field: Some("absent".into()),
                                value: effect(3, "field:"),
                            },
                            Stmt::Assign {
                                id: 0,
                                value: expr(ExprKind::Bool(true), Type::Bool),
                            },
                            Stmt::Restart(1),
                        ],
                        otherwise: Vec::new(),
                    },
                    Stmt::Emit {
                        target: 1,
                        field: None,
                        value: integer(7, 32, true),
                    },
                ],
            }),
            ty,
        );
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![
                    Stmt::Bind {
                        id: 0,
                        value: expr(ExprKind::Bool(false), Type::Bool),
                    },
                    Stmt::Expr(expr(
                        ExprKind::Print {
                            parts: vec![value],
                            newline: true,
                        },
                        Type::Null,
                    )),
                ],
            },
            functions: Vec::new(),
            locals: vec![Type::Bool],
        };
        for release in [false, true] {
            let output = native_program(&program, release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"primary:field:7\n");
        }
    }
}
