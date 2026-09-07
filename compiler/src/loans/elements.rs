use super::access::Kind;
use super::storage::EventKind;
use super::{Bundle, Expr, ExprKind, Graph, Node, Origin, Projection, Result, TRUE, Type};
use crate::borrow_value::Step;
use crate::hir::ReferenceMode;

impl Graph<'_> {
    pub(crate) fn exclusive_element(&mut self, expr: &Expr) -> Result<Bundle> {
        let ExprKind::ExclusiveElement { place, index } = &expr.kind else {
            return Err(Self::budget());
        };
        let element = self
            .proofs
            .exclusive_element_type(self.program, place, self.guards, expr.span)
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
        let path = place
            .fields
            .iter()
            .map(|index| Step::Slot(index + 1))
            .collect::<Vec<_>>();
        self.read_local(place.root, &path, Kind::Read, expr.span, false)?;
        let source = self.proofs.source(place);
        let reservation = self.value(vec![Origin {
            component: Vec::new(),
            source: source.clone(),
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
            source: source.project(&[Projection::Element]),
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
                id: self.cell(place.root),
                take: false,
            },
            expr.span,
        )?;
        let node = self.append(node)?;
        self.grant_mode(node, value, None, ReferenceMode::Exclusive)?;
        Ok(result)
    }
}
