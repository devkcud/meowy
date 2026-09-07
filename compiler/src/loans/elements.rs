use super::access::Kind;
use super::storage::EventKind;
use super::{Bundle, Expr, ExprKind, Graph, Node, Origin, Projection, Result, Source, TRUE, Type};
use crate::hir::ReferenceMode;

impl Graph<'_> {
    pub(crate) fn exclusive_element(&mut self, expr: &Expr) -> Result<Bundle> {
        let ExprKind::ExclusiveElement { id, index } = &expr.kind else {
            return Err(Self::budget());
        };
        let element = self
            .proofs
            .exclusive_element_type(self.program, *id)
            .ok_or_else(|| {
                super::Diagnostic::unsupported("missing mutable scalar list owner proof", expr.span)
            })?;
        if expr.ty != Type::Exclusive(Box::new(element.clone()))
            && !(expr.ty == Type::Never && index.ty == Type::Never)
        {
            return Err(Self::budget());
        }
        if self.current.is_empty() {
            return Ok(Bundle::new());
        }
        self.read_local(*id, &[], Kind::Read, expr.span, false)?;
        let reservation = self.value(vec![Origin {
            component: Vec::new(),
            source: Source::Local {
                id: *id,
                fields: Vec::new(),
            },
            guard: TRUE,
        }])?;
        self.append(Node {
            defs: vec![reservation],
            ..Node::default()
        })?;
        let index = self.expression(index)?;
        if self.current.is_empty() {
            return Ok(Bundle::new());
        }
        let value = self.value(vec![Origin {
            component: Vec::new(),
            source: Source::Local {
                id: *id,
                fields: vec![Projection::Element],
            },
            guard: TRUE,
        }])?;
        let result = Bundle::from([(Vec::new(), value)]);
        let access = self.pointee_access(&result, &[], Kind::Borrow, expr.span)?;
        let mut node = Node {
            uses: std::iter::once(reservation)
                .chain(index.into_values())
                .collect(),
            defs: vec![value],
            access: Some(access),
            ..Node::default()
        };
        self.event(
            &mut node,
            EventKind::Use {
                id: *id,
                take: false,
            },
            expr.span,
        )?;
        let node = self.append(node)?;
        self.grant_mode(node, value, None, ReferenceMode::Exclusive)?;
        Ok(result)
    }
}
