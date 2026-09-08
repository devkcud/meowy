use super::{accepts, rejects};

pub(crate) const RECORD: &str = "x:=1;y:=2;r:={->p:&x;->q:&y;->n:=0};";
pub(crate) const NULLABLE: &str = "p<&int32><null>:=null;";

#[test]
pub(crate) fn mutable_carriers_replace_sources_without_repairing_old_copies() {
    accepts(&format!("{RECORD}r={{->p:&y;->q:&y;->n:=3}};x=4;copy:r"));
    rejects(
        &format!("{RECORD}old:r;r={{->p:&y;->q:&y;->n:=3}};x=4;copy:old"),
        "E302",
    );
    accepts(&format!(
        "{RECORD}{{z:3;r={{->p:&z;->q:&y;->n:=3}}}};safe:*r.q;r={{->p:&x;->q:&y;->n:=4}};copy:r"
    ));
    rejects(
        &format!("{RECORD}{{z:3;r={{->p:&z;->q:&y;->n:=3}}}};copy:r"),
        "E303",
    );
    rejects(
        &format!("{RECORD}old:r;r={{->p:&y;->q:&y;->n:=3}};x=4;copy:old.p"),
        "E302",
    );
}

#[test]
pub(crate) fn mutable_carriers_keep_field_loans_and_post_rhs_state() {
    accepts(&format!(
        "{RECORD}r.n={{r={{->p:&y;->q:&y;->n:=3}};->4}};x=3;copy:r"
    ));
    rejects(
        &format!("{RECORD}r.n={{r={{->p:&y;->q:&y;->n:=3}};->4}};y=3;copy:r"),
        "E302",
    );
    accepts(&format!(
        "{RECORD}cell:&r.n;r={{->p:&x;->q:&y;->n:=*cell}};copy:r"
    ));
    rejects(&format!("{RECORD}cell:&r.n;r.n=3;copy:*cell"), "E302");
    rejects(
        &format!("{RECORD}cell:&r;r={{->p:&y;->q:&y;->n:=3}};copy:*cell"),
        "E302",
    );
}

#[test]
pub(crate) fn nullable_references_preserve_current_predicates_and_expiry() {
    accepts(&format!(
        "{NULLABLE}{{x:1;p=&x}};|p<null>|{{copy:p}};p=null;copy:p"
    ));
    rejects(
        &format!("{NULLABLE}{{x:1;p=&x}};|p<&int32>|{{copy:p}}"),
        "E303",
    );
    rejects(
        &format!("{NULLABLE}|p<null>|{{{{x:1;p=&x}};|p<&int32>|{{copy:p}}}}"),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}{{x:1;p=&x}};|p<&int32>|{{p=null;|p<null>|{{copy:p}}}}"
    ));
    rejects(
        &format!("{NULLABLE}q<&int32><null>:=null;{{x:1;p=&x;q=p}};p=null;copy:q"),
        "E303",
    );
}

#[test]
pub(crate) fn mutable_carriers_keep_nested_union_parent_activity() {
    let types = "<A>:<{p<&int32><null>}>;<B>:<{n<int32>}>;";
    accepts(&format!(
        "{types}r<A><B>:={{->n:2}};{{x:1;r={{->p:&x}}}};|r<B>|{{copy:r.n}};|r<A>|{{|r.p<null>|{{copy:r.p}}}}"
    ));
    rejects(
        &format!(
            "{types}r<A><B>:={{->n:2}};{{x:1;r={{->p:&x}}}};|r<A>|{{|r.p<&int32>|{{copy:r.p}}}}"
        ),
        "E303",
    );
    accepts(&format!(
        "{types}r<A><B>:={{->n:2}};x:1;n:=2;'loop{{|r<B>|{{copy:r.n}};r={{->p:&x}};n=n-1;|n>0|'loop.restart()}}"
    ));
}

#[test]
pub(crate) fn mutable_carriers_preserve_completed_leave_and_argument_effects() {
    rejects(
        &format!(
            "{RECORD}'out{{z:3;r.n={{r={{->p:&z;->q:&y;->n:=3}};'out.leave();->4}}}};copy:r.p"
        ),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}x:1;'out{{p={{p=&x;'out.leave();->null}}}};|p<&int32>|{{copy:*p}}"
    ));
    let call =
        "same<boolean>:(a<&int32><null>,b<&int32><null>){->a==b};x:=1;y:2;p<&int32><null>:=&x;";
    accepts(&format!("{call}v:same(p,{{p=&y;->p}});x=3"));
    rejects(&format!("{call}v:same(p,{{p=&y;x=3;->p}})"), "E302");
}

#[test]
pub(crate) fn nullable_reference_headers_require_active_origins_only() {
    accepts(&format!(
        "{NULLABLE}x:1;n:=2;'loop{{|p<&int32>|{{copy:*p}};p=&x;n=n-1;|n>0|'loop.restart()}};p=null;copy:p"
    ));
    accepts(&format!(
        "{NULLABLE}again:=true;'loop{{|p<null>|{{copy:p}};x:1;|again|{{again=false;p=&x;'loop.restart()}}}};p=null;copy:p"
    ));
    rejects(
        &format!("{NULLABLE}'loop{{|p<&int32>|{{copy:*p}};x:1;p=&x;'loop.restart()}}"),
        "E303",
    );
    accepts(&format!(
        "{NULLABLE}again:=true;'loop{{p=null;copy:p;x:1;|again|{{again=false;p=&x;'loop.restart()}}}}"
    ));
}

#[test]
pub(crate) fn mutable_carriers_keep_transitive_and_public_call_bounds() {
    accepts("x:=1;y:=2;p:&x;q:&y;r:={->p:&p};r={->p:&q};x=3;copy:**r.p");
    rejects("x:=1;p:&x;r:={->p:&p};x=3;copy:**r.p", "E302");
    let prefix = "first<&int32>:(p<&int32>,q<&int32>){->p};x:1;p<&int32><null>:=null;";
    rejects(
        &format!("{prefix}{{y:2;p=first(&x,&y)}};|p<&int32>|{{copy:*p}}"),
        "E303",
    );
    let call = "g<null>:(p<& &int32>,n<int32>){};x:1;q:=&x;r:={->p:&x;->n:=0};";
    rejects(&format!("{call}q=&1;g(&q,{{r.n=2;->0}})"), "E303");
    accepts(&format!(
        "{call}q=&1;'out{{g(&q,{{r.n=2;'out.leave();->0}})}}"
    ));
}

#[test]
pub(crate) fn mutable_carrier_unsupported_storage_stays_explicit() {
    accepts("x:1;r:={->p:=&x}");
    rejects("x:1;r:={->p:&x;->list:[1]}", "B001");
    rejects("x:1;r:={->p:&[1]}", "B001");
    rejects("x:=1;r:={->p:&!x}", "B001");
    rejects("x:1;r:={->p:&x};r.p=&x", "E305");
    rejects("x:1;r:={->p:&x};r={->p:&true}", "E207");
}

#[test]
pub(crate) fn nullable_reference_headers_distinguish_inactive_from_missing_origins() {
    use crate::ast::Span;
    use crate::borrow::header::Shape;
    use crate::borrow_value::State;
    use crate::flow::Flow;
    use crate::hir::Type;

    let mut flow = Flow::default();
    let span = Span::default();
    let ty = Type::Union(vec![
        Type::Null,
        Type::Reference(Box::new(Type::Int {
            bits: 32,
            signed: true,
        })),
    ]);
    let shape = Shape::new(&ty, &mut flow, span).unwrap();
    let null = State::default()
        .convert(&Type::Null, &ty, &mut flow, span)
        .unwrap();
    shape.validate(&null, true, &mut flow, span).unwrap();
    let mut state = crate::borrow_contract::input(0, &ty, &mut flow, span).unwrap();
    shape.validate(&state, true, &mut flow, span).unwrap();
    state.bounds = state.origins.clone();
    state.origins.clear();
    assert_eq!(
        shape
            .validate(&state, true, &mut flow, span)
            .unwrap_err()
            .code,
        "B001"
    );
}
