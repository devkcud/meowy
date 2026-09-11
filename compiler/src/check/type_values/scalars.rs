use crate::ast::{self, ExprKind};
use crate::check::{Checker, Constant, Result, Value};
use crate::diagnostic::Diagnostic;
use crate::hir::Type;

impl Checker {
    pub(crate) fn scalar_input(&mut self, expr: &ast::Expr) -> Result<()> {
        self.type_work.as_mut().unwrap().enter(expr.span)?;
        let result = (|| match &expr.kind {
            ExprKind::Int(_) => Ok(()),
            ExprKind::Name(name) => match self.value(name, expr.span)? {
                Value::Static {
                    value: Constant::Int(_),
                    ..
                }
                | Value::Constant(Constant::Int(_)) => Ok(()),
                Value::Local { constant: None, .. } => Err(Self::error(
                    "E211",
                    "runtime input is unavailable during required type evaluation",
                    expr.span,
                )),
                Value::Local {
                    constant: Some(_), ..
                } => Err(Diagnostic::unsupported(
                    "runtime initializer eligibility for required type evaluation",
                    expr.span,
                )),
                _ => Err(Diagnostic::unsupported(
                    "non-integer computed scalar inputs",
                    expr.span,
                )),
            },
            ExprKind::Group(value) => self.scalar_input(value),
            ExprKind::Unary { op, value } if matches!(op.as_str(), "-" | "~") => {
                self.scalar_input(value)
            }
            ExprKind::Binary { op, left, right }
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^") =>
            {
                self.scalar_input(left)?;
                self.scalar_input(right)
            }
            _ => Err(self.type_unavailable(expr)?),
        })();
        self.type_work.as_mut().unwrap().depth -= 1;
        result
    }

    pub(crate) fn type_scalar(
        &mut self,
        expr: &ast::Expr,
        annotation: Option<&ast::TypeExpr>,
    ) -> Result<Value> {
        self.scalar_input(expr)?;
        let expected = annotation.map(|ty| self.ty(ty)).transpose()?;
        if expected
            .as_ref()
            .is_some_and(|ty| !matches!(ty, Type::Int { .. }))
        {
            return Err(Diagnostic::unsupported(
                "non-integer computed scalar bindings",
                expr.span,
            ));
        }
        let reach = std::mem::replace(&mut self.reach, crate::flow::TRUE);
        let required = std::mem::replace(&mut self.required, true);
        let result = self.expr(expr, expected.as_ref());
        self.reach = reach;
        self.required = required;
        let expr = result?;
        let Some(value @ Constant::Int(_)) = self.constant(&expr) else {
            return Err(Diagnostic::unsupported(
                "non-integer computed scalar bindings",
                expr.span,
            ));
        };
        Ok(Value::Static { value, ty: expr.ty })
    }
}
