use crate::ast::Span;
use crate::hir::{Block, BlockId, Expr, ExprKind, Place, Program, Stmt, Type};
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
        Type::List { element, capacity } => {
            format!("{{ i64, [{capacity} x {}] }}", ir_type(element))
        }
        Type::Reference(_) => "ptr".into(),
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
    ty.layout().expect("validated native layout")
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
            if !self.ended {
                self.line(format!("ret {} {value}", ir_type(&function.result)));
            }
            self.finish(format!(
                "define internal {} @meowy_fn_{}({})",
                ir_type(&function.result),
                function.id,
                params.join(", ")
            ));
        }
        self.begin();
        self.block(&self.program.body)?;
        if !self.ended {
            self.line("ret i32 0".into());
        }
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
            "declare void @meowy_panic_site_v1(i64, i64) noreturn",
            "declare void @meowy_arithmetic_fail_v2(i32, i32, i32, i64, i64, i64, i64) noreturn",
            "declare void @meowy_index_fail_v1(i64, i64, i32, i64, i64) noreturn",
            "declare void @meowy_list_full_v1(i64, i64, i64, i64) noreturn",
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

    pub(crate) fn place(&mut self, place: &Place) -> Result<(String, Type), String> {
        let mut ty = self
            .program
            .locals
            .get(place.root)
            .ok_or_else(|| format!("missing storage root {}", place.root))?
            .clone();
        self.local(place.root);
        let mut ptr = format!("%local{}", place.root);
        for index in &place.fields {
            let Type::Record { fields, .. } = &ty else {
                return Err("storage projection requires a declared record".into());
            };
            let field = fields
                .get(*index)
                .ok_or_else(|| format!("missing storage field {index}"))?
                .1
                .clone();
            ptr = self.value(format!(
                "getelementptr {}, ptr {ptr}, i32 0, i32 {}",
                ir_type(&ty),
                index + 1
            ));
            ty = field;
        }
        Ok((ptr, ty))
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
        } else if let Type::List { element, .. } = ty {
            let source = self.slot(ty);
            self.line(format!("store {} {value}, ptr {source}", ir_type(ty)));
            let length = self.value(format!("extractvalue {} {value}, 0", ir_type(ty)));
            self.line(format!("store i64 {length}, ptr {ptr}"));
            let position = self.slot(&Type::Int {
                bits: 64,
                signed: false,
            });
            self.line(format!("store i64 0, ptr {position}"));
            let head = self.name("list_copy");
            let body = self.name("list_copy_item");
            let end = self.name("list_copy_end");
            self.jump(&head);
            self.label(&head);
            let index = self.value(format!("load i64, ptr {position}"));
            let active = self.value(format!("icmp ult i64 {index}, {length}"));
            self.branch(&active, &body, &end);
            self.label(&body);
            let item = self.list_item(ty, &source, &index);
            let value = self.value(format!("load {}, ptr {item}", ir_type(element)));
            let dest = self.list_item(ty, ptr, &index);
            self.store_value(element, &value, &dest);
            let next = self.value(format!("add i64 {index}, 1"));
            self.line(format!("store i64 {next}, ptr {position}"));
            self.jump(&head);
            self.label(&end);
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
        } else if *ty == Type::Bool {
            let byte = self.value(format!("zext i1 {value} to i8"));
            self.line(format!("store i8 {byte}, ptr {ptr}"));
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
        if block.ty == Type::Never {
            self.line("unreachable".into());
            self.ended = true;
            return Ok("undef".into());
        }
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
                    ..
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
            ExprKind::List { values, list } => self.list(values, list),
            ExprKind::ListSize(value) => {
                let result = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                Ok(self.value(format!("extractvalue {} {result}, 0", ir_type(&value.ty))))
            }
            ExprKind::ListIndex { value, index } => self.list_index(value, index, expression.span),
            ExprKind::ElementBorrow { value, index, .. } => {
                self.element_borrow(value, index, &expression.ty, expression.span)
            }
            ExprKind::ListAdd { value, item } => self.list_add(value, item, expression.span),
            ExprKind::Local(id) => {
                self.local(*id);
                let stored = &self.program.locals[*id];
                let value = self.value(format!("load {}, ptr %local{id}", ir_type(stored)));
                self.coerce(stored, &expression.ty, &value)
            }
            ExprKind::Borrow(place) => {
                let (ptr, stored) = self.place(place)?;
                if expression.ty != Type::Reference(Box::new(stored)) {
                    return Err("borrow type differs from declared storage".into());
                }
                Ok(ptr)
            }
            ExprKind::Reborrow { value, fields, .. } => {
                let Type::Reference(target) = &value.ty else {
                    return Err("reborrow requires a shared reference".into());
                };
                let mut ptr = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                let mut ty = target.as_ref();
                for index in fields {
                    let Type::Record { fields, .. } = ty else {
                        return Err("reborrow projection requires a concrete record".into());
                    };
                    let field = &fields.get(*index).ok_or("missing reborrow field")?.1;
                    ptr = self.value(format!(
                        "getelementptr {}, ptr {ptr}, i32 0, i32 {}",
                        ir_type(ty),
                        index + 1
                    ));
                    ty = field;
                }
                if expression.ty != Type::Reference(Box::new(ty.clone())) {
                    return Err("reborrow result type mismatch".into());
                }
                Ok(ptr)
            }
            ExprKind::Deref(value) => {
                let Type::Reference(stored) = &value.ty else {
                    return Err("dereference requires a shared reference".into());
                };
                let ptr = self.expression(value)?;
                if self.ended {
                    return Ok("undef".into());
                }
                let value = self.value(format!("load {}, ptr {ptr}", ir_type(stored)));
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
                    ("-", Type::Int { .. }) => {
                        Ok(self.checked("-", &expression.ty, &result, None, expression.span))
                    }
                    ("-", Type::Float { .. }) => Ok(self.value(format!("fneg {ty} {result}"))),
                    _ => Err(format!("unsupported checked unary operator {op}")),
                }
            }
            ExprKind::Binary { op, left, right } => self.binary(op, left, right, expression.span),
            ExprKind::Call { id, args, .. } => {
                let mut values = Vec::new();
                for arg in args {
                    let result = self.expression(arg)?;
                    if self.ended {
                        return Ok("undef".into());
                    }
                    values.push(format!("{} {result}", ir_type(&arg.ty)));
                }
                let value = self.value(format!("call {ty} @meowy_fn_{id}({})", values.join(", ")));
                if expression.ty == Type::Never {
                    self.line("unreachable".into());
                    self.ended = true;
                }
                Ok(value)
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
                    self.line(format!(
                        "call void @meowy_panic_site_v1(i64 {}, i64 {})",
                        expression.span.start, expression.span.end
                    ));
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

    pub(crate) fn list_item(&mut self, ty: &Type, ptr: &str, index: &str) -> String {
        self.value(format!(
            "getelementptr {}, ptr {ptr}, i32 0, i32 1, i64 {index}",
            ir_type(ty)
        ))
    }

    pub(crate) fn list(&mut self, values: &[Expr], ty: &Type) -> Result<String, String> {
        let Type::List { element, capacity } = ty else {
            return Err("list literal requires a concrete list type".into());
        };
        if values.len() > *capacity {
            return Err("list literal exceeds its capacity".into());
        }
        let ptr = self.slot(ty);
        self.line(format!("store {} zeroinitializer, ptr {ptr}", ir_type(ty)));
        for (index, value) in values.iter().enumerate() {
            let result = self.expression(value)?;
            if self.ended {
                return Ok("undef".into());
            }
            let result = self.coerce(&value.ty, element, &result)?;
            let dest = self.list_item(ty, &ptr, &index.to_string());
            self.store_value(element, &result, &dest);
        }
        self.line(format!("store i64 {}, ptr {ptr}", values.len()));
        Ok(self.value(format!("load {}, ptr {ptr}", ir_type(ty))))
    }

    pub(crate) fn list_index(
        &mut self,
        value: &Expr,
        index: &Expr,
        span: Span,
    ) -> Result<String, String> {
        let result = self.expression(value)?;
        if self.ended {
            return Ok("undef".into());
        }
        let Type::List { element, .. } = &value.ty else {
            return Err("indexing requires a list value".into());
        };
        let ptr = self.slot(&value.ty);
        self.line(format!("store {} {result}, ptr {ptr}", ir_type(&value.ty)));
        let length = self.value(format!("extractvalue {} {result}, 0", ir_type(&value.ty)));
        let position = self.expression(index)?;
        if self.ended {
            return Ok("undef".into());
        }
        let offset = self.list_offset(index, position, &length, span)?;
        let ptr = self.list_item(&value.ty, &ptr, &offset);
        Ok(self.value(format!("load {}, ptr {ptr}", ir_type(element))))
    }

    pub(crate) fn element_borrow(
        &mut self,
        value: &Expr,
        index: &Expr,
        result: &Type,
        span: Span,
    ) -> Result<String, String> {
        let ptr = self.expression(value)?;
        if self.ended {
            return Ok("undef".into());
        }
        let Type::Reference(list) = &value.ty else {
            return Err("element borrowing requires a shared list reference".into());
        };
        let Type::List { element, .. } = list.as_ref() else {
            return Err("element borrowing requires concrete list storage".into());
        };
        let length = self.value(format!("load i64, ptr {ptr}"));
        let position = self.expression(index)?;
        if self.ended {
            return Ok("undef".into());
        }
        if result != &Type::Reference(element.clone()) {
            return Err("element borrow result type mismatch".into());
        }
        let offset = self.list_offset(index, position, &length, span)?;
        Ok(self.list_item(list, &ptr, &offset))
    }

    pub(crate) fn list_offset(
        &mut self,
        index: &Expr,
        mut position: String,
        length: &str,
        span: Span,
    ) -> Result<String, String> {
        let Type::Int { bits, signed } = index.ty else {
            return Err("list position requires an integer".into());
        };
        if bits < 64 {
            position = self.value(format!(
                "{} i{bits} {position} to i64",
                if signed { "sext" } else { "zext" }
            ));
        } else if bits != 64 {
            return Err("list positions wider than 64 bits are unavailable".into());
        }
        let zero = self.value(format!("icmp eq i64 {position}, 0"));
        let beyond = self.value(format!("icmp ugt i64 {position}, {length}"));
        let invalid = self.value(format!("or i1 {zero}, {beyond}"));
        self.list_guard(
            &invalid,
            format!(
                "call void @meowy_index_fail_v1(i64 {position}, i64 {length}, i32 {}, i64 {}, i64 {})",
                i32::from(signed), span.start, span.end
            ),
        );
        Ok(self.value(format!("sub i64 {position}, 1")))
    }

    pub(crate) fn list_add(
        &mut self,
        value: &Expr,
        item: &Expr,
        span: Span,
    ) -> Result<String, String> {
        let result = self.expression(value)?;
        if self.ended {
            return Ok("undef".into());
        }
        let Type::List { element, capacity } = &value.ty else {
            return Err("append requires a list value".into());
        };
        let ptr = self.slot(&value.ty);
        self.line(format!("store {} {result}, ptr {ptr}", ir_type(&value.ty)));
        let length = self.value(format!("extractvalue {} {result}, 0", ir_type(&value.ty)));
        let result = self.expression(item)?;
        if self.ended {
            return Ok("undef".into());
        }
        let result = self.coerce(&item.ty, element, &result)?;
        let full = self.value(format!("icmp uge i64 {length}, {capacity}"));
        self.list_guard(
            &full,
            format!(
                "call void @meowy_list_full_v1(i64 {length}, i64 {capacity}, i64 {}, i64 {})",
                span.start, span.end
            ),
        );
        let dest = self.list_item(&value.ty, &ptr, &length);
        self.store_value(element, &result, &dest);
        let size = self.value(format!("add i64 {length}, 1"));
        self.line(format!("store i64 {size}, ptr {ptr}"));
        Ok(self.value(format!("load {}, ptr {ptr}", ir_type(&value.ty))))
    }

    pub(crate) fn list_guard(&mut self, invalid: &str, call: String) {
        let fail = self.name("list_fail");
        let next = self.name("list_ok");
        self.branch(invalid, &fail, &next);
        self.label(&fail);
        self.line(call);
        self.line("unreachable".into());
        self.label(&next);
    }

    pub(crate) fn checked(
        &mut self,
        op: &str,
        ty: &Type,
        left: &str,
        right: Option<&str>,
        span: Span,
    ) -> String {
        let Type::Int { bits, signed } = ty else {
            unreachable!()
        };
        let prefix = if *signed { "s" } else { "u" };
        let operation = match op {
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            _ => unreachable!(),
        };
        let name = format!("llvm.{prefix}{operation}.with.overflow.i{bits}");
        self.declarations.insert(format!(
            "declare {{ i{bits}, i1 }} @{name}(i{bits}, i{bits})"
        ));
        let (first, second) = right.map(|right| (left, right)).unwrap_or(("0", left));
        let pair = self.value(format!(
            "call {{ i{bits}, i1 }} @{name}(i{bits} {first}, i{bits} {second})"
        ));
        let overflow = self.value(format!("extractvalue {{ i{bits}, i1 }} {pair}, 1"));
        self.arithmetic_guard(&overflow, op, ty, (left, right), span);
        self.value(format!("extractvalue {{ i{bits}, i1 }} {pair}, 0"))
    }

    pub(crate) fn arithmetic_guard(
        &mut self,
        failure: &str,
        op: &str,
        ty: &Type,
        operands: (&str, Option<&str>),
        span: Span,
    ) {
        let Type::Int { bits, signed } = ty else {
            unreachable!()
        };
        let fail = self.name("fail");
        let next = self.name("ok");
        self.branch(failure, &fail, &next);
        self.label(&fail);
        let extend = if *signed { "sext" } else { "zext" };
        let mut left = operands.0.to_owned();
        let mut right = operands.1.unwrap_or("0").to_owned();
        if *bits < 64 {
            left = self.value(format!("{extend} i{bits} {left} to i64"));
            right = self.value(format!("{extend} i{bits} {right} to i64"));
        }
        let operation = if operands.1.is_none() {
            0
        } else {
            i32::from(op.as_bytes()[0])
        };
        self.line(format!(
            "call void @meowy_arithmetic_fail_v2(i32 {operation}, i32 {bits}, i32 {}, i64 {left}, i64 {right}, i64 {}, i64 {})",
            i32::from(*signed), span.start, span.end
        ));
        self.line("unreachable".into());
        self.label(&next);
    }

    pub(crate) fn binary(
        &mut self,
        op: &str,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Result<String, String> {
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
                    "+" | "-" | "*" => {
                        return Ok(self.checked(op, &left.ty, &first, Some(&second), span));
                    }
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
                        self.arithmetic_guard(
                            &failure,
                            op,
                            &left.ty,
                            (&first, Some(&second)),
                            span,
                        );
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
            Type::Bool | Type::Int { .. } | Type::Reference(_) => {
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
            Type::List { element, .. } => self.list_equal(ty, element, left, right),
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

    pub(crate) fn list_equal(
        &mut self,
        ty: &Type,
        element: &Type,
        left: &str,
        right: &str,
    ) -> Result<String, String> {
        let a = self.slot(ty);
        let b = self.slot(ty);
        self.line(format!("store {} {left}, ptr {a}", ir_type(ty)));
        self.line(format!("store {} {right}, ptr {b}", ir_type(ty)));
        let size = self.value(format!("extractvalue {} {left}, 0", ir_type(ty)));
        let other = self.value(format!("extractvalue {} {right}, 0", ir_type(ty)));
        let same = self.value(format!("icmp eq i64 {size}, {other}"));
        let result = self.slot(&Type::Bool);
        self.line(format!("store i1 false, ptr {result}"));
        let position = self.slot(&Type::Int {
            bits: 64,
            signed: false,
        });
        self.line(format!("store i64 0, ptr {position}"));
        let head = self.name("list_equal");
        let body = self.name("list_equal_item");
        let step = self.name("list_equal_next");
        let done = self.name("list_equal_done");
        let end = self.name("list_equal_end");
        self.branch(&same, &head, &end);
        self.label(&head);
        let index = self.value(format!("load i64, ptr {position}"));
        let active = self.value(format!("icmp ult i64 {index}, {size}"));
        self.branch(&active, &body, &done);
        self.label(&body);
        let left = self.list_item(ty, &a, &index);
        let right = self.list_item(ty, &b, &index);
        let left = self.value(format!("load {}, ptr {left}", ir_type(element)));
        let right = self.value(format!("load {}, ptr {right}", ir_type(element)));
        let equal = self.equal(element, &left, &right)?;
        self.branch(&equal, &step, &end);
        self.label(&step);
        let next = self.value(format!("add i64 {index}, 1"));
        self.line(format!("store i64 {next}, ptr {position}"));
        self.jump(&head);
        self.label(&done);
        self.line(format!("store i1 true, ptr {result}"));
        self.jump(&end);
        self.label(&end);
        Ok(self.value(format!("load i1, ptr {result}")))
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
            Type::Reference(_) => return Err("shared-reference formatting is unavailable".into()),
            Type::List { .. } => return Err("list formatting is unavailable".into()),
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

    pub(crate) fn list(element: Type, capacity: usize, values: Vec<Expr>) -> Expr {
        let ty = Type::List {
            element: Box::new(element),
            capacity,
        };
        expr(
            ExprKind::List {
                values,
                list: ty.clone(),
            },
            ty,
        )
    }

    pub(crate) fn index(value: Expr, position: Expr) -> Expr {
        let Type::List { element, .. } = &value.ty else {
            unreachable!()
        };
        let ty = *element.clone();
        expr(
            ExprKind::ListIndex {
                value: Box::new(value),
                index: Box::new(position),
            },
            ty,
        )
    }

    pub(crate) fn borrow(root: usize, ty: Type) -> Expr {
        expr(
            ExprKind::Borrow(Place {
                root,
                fields: Vec::new(),
            }),
            Type::Reference(Box::new(ty)),
        )
    }

    pub(crate) fn element_ref(value: Expr, index: Expr) -> Expr {
        let Type::Reference(list) = &value.ty else {
            unreachable!()
        };
        let Type::List { element, .. } = list.as_ref() else {
            unreachable!()
        };
        let ty = if index.ty == Type::Never {
            Type::Never
        } else {
            Type::Reference(element.clone())
        };
        expr(
            ExprKind::ElementBorrow {
                site: 0,
                value: Box::new(value),
                index: Box::new(index),
            },
            ty,
        )
    }

    pub(crate) fn size(value: Expr) -> Expr {
        expr(
            ExprKind::ListSize(Box::new(value)),
            Type::Int {
                bits: 64,
                signed: false,
            },
        )
    }

    pub(crate) fn separated(parts: Vec<Expr>) -> Vec<Expr> {
        parts
            .into_iter()
            .flat_map(|part| [part, expr(ExprKind::String("|".into()), Type::String)])
            .collect()
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
    pub(crate) fn lists_compare_initialized_prefixes_and_preserve_element_equality() {
        let int = Type::Int {
            bits: 32,
            signed: true,
        };
        let values = list(
            int.clone(),
            3,
            vec![integer(11, 32, true), integer(22, 32, true)],
        );
        let appended = expr(
            ExprKind::ListAdd {
                value: Box::new(values.clone()),
                item: Box::new(integer(33, 32, true)),
            },
            values.ty.clone(),
        );
        let empty = list(Type::Bool, 0, Vec::new());
        let short = list(int, 3, vec![integer(11, 32, true)]);
        let float = Type::Float { bits: 64 };
        let zero = list(
            float.clone(),
            2,
            vec![expr(ExprKind::Float(0.0), float.clone())],
        );
        let negative = list(
            float.clone(),
            2,
            vec![expr(ExprKind::Float(-0.0), float.clone())],
        );
        let nan = binary(
            "/",
            expr(ExprKind::Float(0.0), float.clone()),
            expr(ExprKind::Float(0.0), float.clone()),
            float.clone(),
        );
        let nan = list(float, 1, vec![nan]);
        for release in [false, true] {
            let output = native(
                separated(vec![
                    binary("==", empty.clone(), empty.clone(), Type::Bool),
                    size(values.clone()),
                    index(values.clone(), integer(2, 8, false)),
                    size(appended.clone()),
                    index(appended.clone(), integer(3, 64, true)),
                    size(values.clone()),
                    binary("==", values.clone(), short.clone(), Type::Bool),
                    binary("==", zero.clone(), negative.clone(), Type::Bool),
                    binary("==", nan.clone(), nan.clone(), Type::Bool),
                ]),
                release,
                false,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"true|2|22|3|33|2|false|true|false|\n");
        }
    }

    #[test]
    pub(crate) fn lists_preserve_nested_and_padded_union_payloads() {
        let record = record(
            1,
            expr(ExprKind::Bool(true), Type::Bool),
            integer(41, 64, true),
        );
        let values = list(record.ty.clone(), 3, vec![record.clone()]);
        let union = Type::union([Type::Null, values.ty.clone()]);
        let packed = coerce(values.clone(), &union);
        let restored = coerce(packed.clone(), &values.ty);
        let first = index(restored.clone(), integer(1, 32, true));
        let inner = list(Type::Bool, 3, vec![expr(ExprKind::Bool(true), Type::Bool)]);
        let outer = list(inner.ty.clone(), 2, vec![inner]);
        let outer_union = Type::union([Type::Null, outer.ty.clone()]);
        let nested = coerce(coerce(outer.clone(), &outer_union), &outer.ty);
        for release in [false, true] {
            let output = native(
                separated(vec![
                    expr(ExprKind::Primary(Box::new(first.clone())), Type::Bool),
                    expr(
                        ExprKind::Field {
                            value: Box::new(first.clone()),
                            index: 0,
                        },
                        Type::Int {
                            bits: 64,
                            signed: true,
                        },
                    ),
                    size(restored.clone()),
                    binary("==", packed.clone(), packed.clone(), Type::Bool),
                    index(
                        index(nested.clone(), integer(1, 32, true)),
                        integer(1, 32, true),
                    ),
                ]),
                release,
                false,
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"true|41|1|true|true|\n");
        }
    }

    #[test]
    pub(crate) fn list_failures_keep_index_sign_length_capacity_and_site() {
        let values = list(
            Type::Bool,
            256,
            vec![expr(ExprKind::Bool(true), Type::Bool); 256],
        );
        for release in [false, true] {
            for (position, text) in [
                (integer(-1, 8, true), "-1"),
                (integer(0, 32, true), "0"),
                (integer(u64::MAX.into(), 64, false), "18446744073709551615"),
            ] {
                let mut value = index(values.clone(), position);
                value.span = Span { start: 10, end: 20 };
                let output = native(vec![value], release, false);
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert_eq!(
                    String::from_utf8_lossy(&output.stderr),
                    format!(
                        "panic[P001]: index {text} is outside initialized length 256 at bytes 10..20\n"
                    )
                );
            }
            let value = list(Type::Bool, 1, vec![expr(ExprKind::Bool(true), Type::Bool)]);
            let ty = value.ty.clone();
            let mut full = expr(
                ExprKind::ListAdd {
                    value: Box::new(value),
                    item: Box::new(expr(ExprKind::Bool(false), Type::Bool)),
                },
                ty,
            );
            full.span = Span { start: 30, end: 40 };
            let output = native(vec![size(full)], release, false);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                b"panic[P003]: bounded list is full (length 1, capacity 1) at bytes 30..40\n"
            );
        }
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
                    let minimum = if signed { -(1i128 << (bits - 1)) } else { 0 };
                    let name = if signed { "int" } else { "uint" };
                    for (op, left, right) in
                        [("+", maximum, 1), ("-", minimum, 1), ("*", maximum, 2)]
                    {
                        let mut value = binary(
                            op,
                            integer(left, bits, signed),
                            integer(right, bits, signed),
                            ty.clone(),
                        );
                        value.span = Span { start: 11, end: 17 };
                        let output = native(vec![value], release, false);
                        assert_eq!(
                            output.status.code(),
                            Some(1),
                            "bits={bits}, signed={signed}, release={release}, op={op}"
                        );
                        assert_eq!(output.stderr, format!("panic[P002]: {name}{bits} {op} overflow (left {left}, right {right}; range {minimum}..{maximum}) at bytes 11..17\n").as_bytes());
                    }
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
                    assert_eq!(output.stderr, b"panic[P002]: int64 / overflow (left -9223372036854775808, right -1; range -9223372036854775808..9223372036854775807) at bytes 0..0\n");
                } else {
                    assert!(output.status.success());
                    assert_eq!(output.stdout, b"0\n");
                }
                let value = binary(op, integer(8, 64, true), integer(0, 64, true), ty.clone());
                let output = native(vec![value], release, false);
                assert_eq!(output.status.code(), Some(1));
                assert_eq!(output.stderr, format!("panic[P002]: int64 {op} zero divisor (left 8, right 0; range -9223372036854775808..9223372036854775807) at bytes 0..0\n").as_bytes());
            }
            let minimum = expr(
                ExprKind::Unary {
                    op: "-".into(),
                    value: Box::new(integer(i64::MIN.into(), 64, true)),
                },
                ty,
            );
            let output = native(vec![minimum], release, false);
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stderr, b"panic[P002]: int64 unary - overflow (value -9223372036854775808; range -9223372036854775808..9223372036854775807) at bytes 0..0\n");
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
                        id: 0,
                        target: id,
                        field: None,
                        value: primary,
                    },
                    Stmt::Emit {
                        id: 0,
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
    pub(crate) fn shared_references_preserve_storage_identity_and_field_addresses() {
        let ty = Type::Int {
            bits: 32,
            signed: true,
        };
        let reference = Type::Reference(Box::new(ty.clone()));
        let borrow =
            |root, fields| expr(ExprKind::Borrow(Place { root, fields }), reference.clone());
        let value = record(
            1,
            integer(2, 8, false),
            record(2, integer(3, 16, true), integer(7, 32, true)),
        );
        let shape = value.ty.clone();
        let alias = expr(ExprKind::Local(2), reference.clone());
        let parts = vec![
            binary("==", alias.clone(), borrow(0, vec![]), Type::Bool),
            binary("==", borrow(0, vec![]), borrow(1, vec![]), Type::Bool),
            expr(ExprKind::Deref(Box::new(alias)), ty.clone()),
            expr(ExprKind::Deref(Box::new(borrow(3, vec![0, 0]))), ty.clone()),
            binary(
                "==",
                borrow(3, vec![0, 0]),
                borrow(3, vec![0, 0]),
                Type::Bool,
            ),
            expr(
                ExprKind::Deref(Box::new(expr(
                    ExprKind::Borrow(Place {
                        root: 3,
                        fields: vec![],
                    }),
                    Type::Reference(Box::new(shape.clone())),
                ))),
                shape.clone(),
            ),
        ];
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![
                    Stmt::Bind {
                        id: 0,
                        value: integer(42, 32, true),
                    },
                    Stmt::Bind {
                        id: 1,
                        value: integer(42, 32, true),
                    },
                    Stmt::Bind {
                        id: 2,
                        value: borrow(0, vec![]),
                    },
                    Stmt::Bind { id: 3, value },
                    Stmt::Expr(expr(
                        ExprKind::Print {
                            parts,
                            newline: true,
                        },
                        Type::Null,
                    )),
                ],
            },
            functions: Vec::new(),
            locals: vec![ty.clone(), ty, reference, shape],
        };
        for release in [false, true] {
            let output = native_program(&program, release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"truefalse427true2\n");
        }
    }

    #[test]
    pub(crate) fn element_references_preserve_owner_identity_and_nested_offsets() {
        let int = Type::Int {
            bits: 32,
            signed: true,
        };
        let values = list(
            int.clone(),
            3,
            vec![integer(11, 32, true), integer(22, 32, true)],
        );
        let row = record(
            2,
            integer(7, 16, false),
            list(
                int.clone(),
                4,
                vec![integer(88, 32, true), integer(99, 32, true)],
            ),
        );
        let rows = list(row.ty.clone(), 2, vec![row.clone()]);
        let nested = record(1, expr(ExprKind::Bool(true), Type::Bool), rows.clone());
        let list_ref = Type::Reference(Box::new(values.ty.clone()));
        let first = element_ref(borrow(0, values.ty.clone()), integer(1, 8, false));
        let second = element_ref(borrow(0, values.ty.clone()), integer(2, 16, true));
        let inner = expr(
            ExprKind::Reborrow {
                site: 1,
                value: Box::new(borrow(3, nested.ty.clone())),
                fields: vec![0],
            },
            Type::Reference(Box::new(rows.ty.clone())),
        );
        let Type::Record { fields, .. } = &row.ty else {
            unreachable!()
        };
        let inner = expr(
            ExprKind::Reborrow {
                site: 2,
                value: Box::new(element_ref(inner, integer(1, 32, false))),
                fields: vec![0],
            },
            Type::Reference(Box::new(fields[0].1.clone())),
        );
        let inner = element_ref(inner, integer(2, 64, false));
        let parts = separated(vec![
            binary("==", first.clone(), first.clone(), Type::Bool),
            binary("==", first.clone(), second.clone(), Type::Bool),
            binary(
                "==",
                first,
                element_ref(borrow(1, values.ty.clone()), integer(1, 32, true)),
                Type::Bool,
            ),
            binary(
                "==",
                second.clone(),
                element_ref(
                    expr(ExprKind::Local(2), list_ref.clone()),
                    integer(2, 8, true),
                ),
                Type::Bool,
            ),
            expr(ExprKind::Deref(Box::new(second)), int.clone()),
            expr(ExprKind::Deref(Box::new(inner.clone())), int),
            binary("==", inner.clone(), inner, Type::Bool),
        ]);
        let program = Program {
            body: Block {
                id: 0,
                ty: Type::Null,
                stmts: vec![
                    Stmt::Bind {
                        id: 0,
                        value: values.clone(),
                    },
                    Stmt::Bind {
                        id: 1,
                        value: expr(ExprKind::Local(0), values.ty.clone()),
                    },
                    Stmt::Bind {
                        id: 2,
                        value: borrow(0, values.ty.clone()),
                    },
                    Stmt::Bind {
                        id: 3,
                        value: nested.clone(),
                    },
                    Stmt::Expr(expr(
                        ExprKind::Print {
                            parts,
                            newline: true,
                        },
                        Type::Null,
                    )),
                ],
            },
            functions: Vec::new(),
            locals: vec![values.ty.clone(), values.ty, list_ref, nested.ty],
        };
        for release in [false, true] {
            let output = native_program(&program, release, false);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout, b"true|false|false|true|22|99|true|\n");
            assert!(output.stderr.is_empty());
        }
    }

    #[test]
    pub(crate) fn element_reference_failures_preserve_index_widths_and_initialized_length() {
        let values = list(Type::Bool, 4, vec![expr(ExprKind::Bool(true), Type::Bool)]);
        for release in [false, true] {
            for bits in [8, 16, 32, 64] {
                for signed in [false, true] {
                    let number = if signed {
                        -(1i128 << (bits - 1))
                    } else {
                        (1i128 << bits) - 1
                    };
                    let mut value =
                        element_ref(borrow(0, values.ty.clone()), integer(number, bits, signed));
                    value.span = Span { start: 12, end: 34 };
                    let program = Program {
                        body: Block {
                            id: 0,
                            ty: Type::Null,
                            stmts: vec![
                                Stmt::Bind {
                                    id: 0,
                                    value: values.clone(),
                                },
                                Stmt::Expr(value),
                            ],
                        },
                        functions: Vec::new(),
                        locals: vec![values.ty.clone()],
                    };
                    let output = native_program(&program, release, false);
                    assert_eq!(output.status.code(), Some(1));
                    assert!(output.stdout.is_empty());
                    assert_eq!(output.stderr, format!("panic[P001]: index {number} is outside initialized length 1 at bytes 12..34\n").as_bytes());
                }
            }
        }
    }

    #[test]
    pub(crate) fn element_references_evaluate_parent_then_index_and_stop_on_divergence() {
        let print = |text: &str| {
            Stmt::Expr(expr(
                ExprKind::Print {
                    parts: vec![expr(ExprKind::String(text.into()), Type::String)],
                    newline: true,
                },
                Type::Null,
            ))
        };
        for release in [false, true] {
            for panic in [false, true] {
                let values = list(Type::Bool, 0, Vec::new());
                let parent = expr(
                    ExprKind::Block(Block {
                        id: 1,
                        ty: Type::Reference(Box::new(values.ty.clone())),
                        stmts: vec![
                            print("parent"),
                            Stmt::Emit {
                                id: 0,
                                target: 1,
                                field: None,
                                value: borrow(0, values.ty.clone()),
                            },
                        ],
                    }),
                    Type::Reference(Box::new(values.ty.clone())),
                );
                let result = if panic {
                    Type::Never
                } else {
                    integer(1, 32, true).ty
                };
                let end = if panic {
                    Stmt::Expr(Expr {
                        kind: ExprKind::Panic {
                            parts: vec![expr(ExprKind::String("stop".into()), Type::String)],
                        },
                        ty: Type::Never,
                        span: Span { start: 30, end: 40 },
                    })
                } else {
                    Stmt::Emit {
                        id: 1,
                        target: 2,
                        field: None,
                        value: integer(1, 32, true),
                    }
                };
                let index = expr(
                    ExprKind::Block(Block {
                        id: 2,
                        ty: result.clone(),
                        stmts: vec![print("index"), end],
                    }),
                    result,
                );
                let mut value = element_ref(parent, index);
                value.span = Span { start: 10, end: 50 };
                let program = Program {
                    body: Block {
                        id: 0,
                        ty: Type::Null,
                        stmts: vec![
                            Stmt::Bind {
                                id: 0,
                                value: values.clone(),
                            },
                            Stmt::Expr(value),
                        ],
                    },
                    functions: Vec::new(),
                    locals: vec![values.ty],
                };
                if panic {
                    assert!(
                        !emit_ir(&program)
                            .unwrap()
                            .contains("call void @meowy_index_fail_v1")
                    );
                }
                let output = native_program(&program, release, false);
                assert_eq!(output.status.code(), Some(1));
                assert_eq!(output.stdout, b"parent\nindex\n");
                assert_eq!(
                    output.stderr,
                    if panic {
                        b"panic[P006]: stop at bytes 30..40\n".as_slice()
                    } else {
                        b"panic[P001]: index 1 is outside initialized length 0 at bytes 10..50\n"
                            .as_slice()
                    }
                );
            }
        }
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
                            id: 0,
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
                                id: 0,
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
                            id: 0,
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
                        id: 0,
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
                    site: 0,
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
                        id: 0,
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
                            id: 0,
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
                                id: 0,
                                target: 1,
                                field: None,
                                value: coerce(
                                    effect(2, "primary:"),
                                    &Type::union([Type::Bool, Type::String]),
                                ),
                            },
                            Stmt::Emit {
                                id: 0,
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
                        id: 0,
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
