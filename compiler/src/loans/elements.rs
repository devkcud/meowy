use super::access::Kind;
use super::storage::EventKind;
use super::{Bundle, Expr, ExprKind, Graph, Node, Origin, Projection, Result, TRUE, Type};
use crate::borrow_value::Step;
use crate::hir::{ReferenceMode, WriteStep};

impl Graph<'_> {
    pub(crate) fn exclusive_element(&mut self, expr: &Expr) -> Result<Bundle> {
        let ExprKind::ExclusiveElement { place, path, index } = &expr.kind else {
            return Err(Self::budget());
        };
        let element = self
            .proofs
            .exclusive_element_type(self.program, place, path, self.guards, expr.span)
            .ok_or_else(|| {
                super::Diagnostic::unsupported("missing mutable scalar list owner proof", expr.span)
            })?;
        let diverges = index.ty == Type::Never || path.iter().any(|step| {
            matches!(step, crate::hir::WriteStep::Index(step) if step.index.ty == Type::Never)
        });
        if expr.ty != Type::Exclusive(Box::new(element.clone()))
            && !(expr.ty == Type::Never && diverges)
        {
            return Err(Self::budget());
        }
        if self.current.is_empty() {
            return Ok(Bundle::new());
        }
        let fields = place
            .fields
            .iter()
            .map(|index| Step::Slot(index + 1))
            .collect::<Vec<_>>();
        self.read_local(place.root, &fields, Kind::Read, expr.span, false)?;
        let mut source = self.proofs.source(place);
        let mut reservations = Vec::new();
        for step in path {
            self.charge(place.fields.len() + path.len() + reservations.len() + 1)?;
            match step {
                WriteStep::Field(index) => source = source.project(&[Projection::Field(*index)]),
                WriteStep::Index(step) => {
                    reservations.push(self.element_reservation(source.clone())?);
                    let index = self.expression(&step.index)?;
                    if self.current.is_empty() {
                        return Ok(Bundle::new());
                    }
                    self.append(Node {
                        uses: reservations
                            .iter()
                            .copied()
                            .chain(index.into_values())
                            .collect(),
                        ..Node::default()
                    })?;
                    source = source.project(&[Projection::Element]);
                }
            }
        }
        reservations.push(self.element_reservation(source.clone())?);
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
            uses: reservations
                .into_iter()
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

    pub(crate) fn element_reservation(&mut self, source: super::Source) -> Result<usize> {
        let reservation = self.value(vec![Origin {
            component: Vec::new(),
            source,
            guard: TRUE,
        }])?;
        self.append(Node {
            defs: vec![reservation],
            ..Node::default()
        })?;
        Ok(reservation)
    }
}
