use crate::compile;

pub(crate) const PREFIX: &str = r#"m:@"memory";f<m.Allocator>:(view<&int32>){->m.heap};"#;

pub(crate) fn accepts(body: &str) {
    let source = format!("{PREFIX}{body}");
    if let Err(errors) = compile(&source) {
        panic!("{source}\n{errors:?}");
    }
}

pub(crate) fn rejects(body: &str, code: &str) {
    let source = format!("{PREFIX}{body}");
    let errors = compile(&source).expect_err(&source);
    assert_eq!(errors[0].code, code, "{source}\n{errors:?}");
}

#[test]
pub(crate) fn allocator_call_bounds_follow_sources_and_transitive_copies() {
    accepts("owner:1;a:f(&owner);b:a");
    accepts("g<m.Allocator>:(a<m.Allocator>){->a};owner:1;h:g(f(&owner));copy:h");
    accepts("owner:1;result:{a:f(&owner);->*&a}");
    rejects("h:{owner:1;->f(&owner)}", "E303");
    rejects("h:f(&1);copy:h", "E303");
    rejects(
        "g<m.Allocator>:(a<m.Allocator>){->a};h:{owner:1;->g(f(&owner))}",
        "E303",
    );
    rejects(
        "load<m.Allocator>:(a<&m.Allocator>){->*a};owner:1;h:{a:f(&owner);->load(&a)}",
        "E303",
    );
}

#[test]
pub(crate) fn allocator_bounds_do_not_invent_physical_loans() {
    accepts("owner:=1;h:f(&owner);owner=2;copy:h");
    accepts("g<boolean>:(a<m.Allocator>){->true};owner:=1;h:f(&owner);owner=2;ok:g(h)");
}

#[test]
pub(crate) fn allocator_records_and_unions_keep_only_selected_bounds() {
    accepts("g: (v<&int32>){->handle:f(v);->n:7};answer:{x:1;r:g(&x);->r.n}");
    rejects(
        "g: (v<&int32>){->handle:f(v);->n:7};answer:{x:1;->g(&x)}",
        "E303",
    );
    accepts(
        "g<m.Allocator><null>:(v<&int32>,yes<boolean>){|yes|->f(v)};x:1;h:g(&x,true);|h<m.Allocator>|{copy:h}",
    );
    accepts(
        "g<m.Allocator><null>:(v<&int32>,yes<boolean>){|yes|->f(v)};h:'out{x:1;a:g(&x,false);|a<null>|{'out->a;'out.leave()};->null}",
    );
    rejects(
        "g<m.Allocator><null>:(v<&int32>,yes<boolean>){|yes|->f(v)};h:{x:1;->g(&x,true)}",
        "E303",
    );
}

#[test]
pub(crate) fn allocator_bound_loss_in_mutation_and_lists_is_explicit() {
    accepts("x:1;a:=f(&x);copy:a");
    accepts("x:1;a:=m.heap;a=f(&x);copy:a");
    rejects("x:1;items:[f(&x)]", "B001");
    rejects("x:1;items:=[m.heap];items[1]=f(&x)", "B001");
    accepts("x:1;holder:={->handle:=m.heap};holder.handle=f(&x);copy:holder.handle");
    rejects("x:1;items<m.Allocator[1]>:[];more:items.add(f(&x))", "B001");
    rejects(
        "g<m.Allocator[1]>:(v<&int32>){->[m.heap]};x:1;items:g(&x)",
        "B001",
    );
    accepts("a:=m.heap;items:=[a];items[1]=m.heap");
    accepts("g<m.Allocator[1]>:(a<m.Allocator>){->[a]};items:g(m.heap)");
}

#[test]
pub(crate) fn allocator_expiry_is_checked_at_real_call_entry() {
    rejects(
        "consume<boolean>:(a<m.Allocator>){->true};a:f(&1);ok:consume(a)",
        "E303",
    );
    accepts(
        "consume<boolean>:(a<m.Allocator>,b<int32>){->true};'out {consume(f(&1),{'out.leave();->0})}",
    );
    rejects(
        "consume<boolean>:(a<&m.Allocator>){->true};a:f(&1);ok:consume(&a)",
        "E303",
    );
}

#[test]
pub(crate) fn allocator_bounds_survive_shared_carriers_and_reborrows() {
    accepts("owner:1;h:{->a:f(&owner);->n:7};p:&h;copy:*(&(p.a))");
    rejects(
        "owner:1;h:{short:1;->a:f(&short);->n:7};p:&h;copy:p.a",
        "E303",
    );
    rejects("a:f(&1);p:&a;copy:*p", "E303");
    rejects("a:f(&1);holder:{->p:&a};copy:*(holder.p)", "E303");
    accepts(
        "first:1;second:2;g<m.Allocator>:(a<&int32>,b<&int32>){->m.heap};value:g(&first,&second);copy:value",
    );
    rejects(
        "first:1;g<m.Allocator>:(a<&int32>,b<&int32>){->m.heap};h:{short:2;->g(&first,&short)}",
        "E303",
    );
}

#[test]
pub(crate) fn bounded_reference_restart_headers_preserve_allocator_constraints() {
    accepts(
        "x:1;a:f(&x);p:=&a;again:=true;'loop{|again|{again=false;p=&a;'loop.restart()}};copy:*p",
    );
    accepts(
        "a<m.Allocator><null>:m.heap;p:=&a;again:=true;'loop{|again|{again=false;p=&a;'loop.restart()}};copy:*p",
    );
}

#[test]
pub(crate) fn allocator_bound_fanout_obeys_the_shared_budget() {
    let params = (0..65)
        .map(|i| format!("p{i}<&int32>"))
        .collect::<Vec<_>>()
        .join(",");
    let fields = (0..65)
        .map(|i| format!("->h{i}:m.heap;"))
        .collect::<String>();
    let owners = (0..65).map(|i| format!("v{i}:{i};")).collect::<String>();
    let args = (0..65)
        .map(|i| format!("&v{i}"))
        .collect::<Vec<_>>()
        .join(",");
    rejects(
        &format!("many:({params}){{{fields}}};{owners}result:many({args})"),
        "B001",
    );
}
