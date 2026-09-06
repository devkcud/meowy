mod aggregate;
mod arithmetic;
mod lists;
mod output;

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
        Type::List { element, capacity } => {
            format!("{{ i64, [{capacity} x {}] }}", ir_type(element))
        }
        Type::Reference(_) => "ptr".into(),
        Type::Record { primary, fields } => {
            let mut types = vec![ir_type(primary)];
            types.extend(fields.iter().map(|field| ir_type(&field.ty)));
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
                Stmt::SetElement {
                    id, path, value, ..
                } => {
                    self.set_element(*id, path, value)?;
                }
                Stmt::SetField { place, value, .. } => {
                    self.set_field(place, value)?;
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
                                let Some(index) =
                                    fields.iter().position(|slot| &slot.name == field)
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
                                &fields[index - 1].ty
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
                    let field = &fields.get(*index).ok_or("missing reborrow field")?.ty;
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
}

#[cfg(test)]
mod tests;
