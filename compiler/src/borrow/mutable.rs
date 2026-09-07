use super::{Block, Diagnostic, Expr, ExprKind, Guards, Program, Result, Span, State, Stmt, Type};
use crate::hir::WriteStep;

pub(crate) enum Item<'a> {
    Statement(&'a Stmt),
    Expression(&'a Expr),
}

pub(crate) fn push<'a>(
    pending: &mut Vec<(Item<'a>, bool, usize)>,
    item: Item<'a>,
    guarded: bool,
    depth: usize,
    guards: &mut Guards,
) -> Result<()> {
    if pending.len() >= 65_536 || depth >= 256 || !guards.spend(1) {
        return Err(State::budget(Span::default()));
    }
    pending.push((item, guarded, depth));
    Ok(())
}

pub(crate) fn check(block: &Block, program: &Program, guards: &mut Guards) -> Result<()> {
    let mut pending = Vec::new();
    for stmt in block.stmts.iter().rev() {
        push(&mut pending, Item::Statement(stmt), false, 0, guards)?;
    }
    let mut write = None;
    let mut transfer = false;
    while let Some((item, guarded, depth)) = pending.pop() {
        if !guards.spend(1) {
            return Err(State::budget(Span::default()));
        }
        let mut add = |item, conditional| {
            push(
                &mut pending,
                item,
                guarded || conditional,
                depth + 1,
                guards,
            )
        };
        match item {
            Item::Statement(stmt) => match stmt {
                Stmt::Statement { stmts, .. } => {
                    for stmt in stmts.iter().rev() {
                        add(Item::Statement(stmt), false)?;
                    }
                }
                Stmt::Assign { id, value } => {
                    if matches!(program.locals.get(*id), Some(Type::Reference(_))) {
                        if guarded {
                            return Err(Diagnostic::unsupported(
                                "shared-reference assignment under conditional evaluation",
                                value.span,
                            ));
                        }
                        write.get_or_insert(value.span);
                    }
                    add(Item::Expression(value), false)?;
                }
                Stmt::Bind { value, .. } | Stmt::Emit { value, .. } | Stmt::Expr(value) => {
                    add(Item::Expression(value), false)?;
                }
                Stmt::SetPath { path, value, .. } => {
                    add(Item::Expression(value), false)?;
                    for step in path.iter().rev() {
                        if let WriteStep::Index(step) = step {
                            add(Item::Expression(&step.index), false)?;
                        }
                    }
                }
                Stmt::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    for stmt in otherwise.iter().chain(then).rev() {
                        add(Item::Statement(stmt), true)?;
                    }
                    add(Item::Expression(condition), false)?;
                }
                Stmt::Leave(_) | Stmt::Restart(_) => transfer = true,
                Stmt::SlotAlias { .. } => {}
            },
            Item::Expression(expr) => match &expr.kind {
                ExprKind::Block(block) => {
                    for stmt in block.stmts.iter().rev() {
                        add(Item::Statement(stmt), false)?;
                    }
                }
                ExprKind::Binary { op, left, right } => {
                    add(Item::Expression(right), ["&&", "||"].contains(&op.as_str()))?;
                    add(Item::Expression(left), false)?;
                }
                ExprKind::TemporaryBorrow { value, .. }
                | ExprKind::Reborrow { value, .. }
                | ExprKind::Deref(value)
                | ExprKind::Unary { value, .. }
                | ExprKind::Field { value, .. }
                | ExprKind::Primary(value)
                | ExprKind::StringSize(value)
                | ExprKind::ListSize(value)
                | ExprKind::Coerce { value }
                | ExprKind::TypeTest { value, .. } => add(Item::Expression(value), false)?,
                ExprKind::ElementBorrow { value, index, .. }
                | ExprKind::ListIndex { value, index }
                | ExprKind::ListAdd { value, item: index } => {
                    add(Item::Expression(index), false)?;
                    add(Item::Expression(value), false)?;
                }
                ExprKind::List { values, .. }
                | ExprKind::Call { args: values, .. }
                | ExprKind::Print { parts: values, .. }
                | ExprKind::Panic { parts: values } => {
                    for value in values.iter().rev() {
                        add(Item::Expression(value), false)?;
                    }
                }
                ExprKind::Null
                | ExprKind::Bool(_)
                | ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::String(_)
                | ExprKind::Local(_)
                | ExprKind::Borrow(_) => {}
            },
        }
    }
    if transfer && let Some(span) = write {
        return Err(Diagnostic::unsupported(
            "shared-reference assignments in a body with leave or restart",
            span,
        ));
    }
    Ok(())
}
