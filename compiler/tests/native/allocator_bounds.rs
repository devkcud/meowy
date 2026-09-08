use super::Case;

#[test]
pub fn allocator_bounds_preserve_execution_and_allow_unrelated_writes() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
get<m.Allocator>:(value<&int32>){d.print(*value);->m.heap}
pass<m.Allocator>:(handle<m.Allocator>){->handle}
maybe<m.Allocator><null>:(value<&int32>,yes<boolean>){|yes|->m.heap}
pack:(value<&int32>){->handle:m.heap;->n:9}
owner:=7
handle:get(&owner)
owner=8
copy:pass(handle)
|copy<m.Allocator>|d.print(owner)
holder:{->handle:copy;->n:9}
view:&holder
selected:*(&view.handle)
|selected<m.Allocator>|d.print("heap")
missing:maybe(&owner,false)
|missing<null>|d.print("empty")
number:{short:1;record:pack(&short);->record.n}
d.print(number)
"#,
    )
    .runs(b"7\n8\nheap\nempty\n9\n");
}

#[test]
pub fn an_argument_leave_skips_allocator_contract_entry() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
get<m.Allocator>:(value<&int32>){d.print(*value);->m.heap}
consume<boolean>:(handle<m.Allocator>,n<int32>){d.print("entered");->true}
'out{consume(get(&1),{'out.leave();->0})}
d.print("after")
"#,
    )
    .runs(b"1\nafter\n");
}

#[test]
pub fn allocator_expiry_and_unmodeled_storage_report_distinct_codes() {
    let prefix = r#"m:@"memory";get<m.Allocator>:(value<&int32>){->m.heap};"#;
    for (body, code) in [
        ("result:{local:1;->get(&local)}", "E303"),
        ("result:get(&1);copy:result", "E303"),
        (
            "load<m.Allocator>:(p<&m.Allocator>){->*p};owner:1;result:{h:get(&owner);->load(&h)}",
            "E303",
        ),
        ("owner:1;result:={->handle:get(&owner)}", "B001"),
        ("owner:1;items:[get(&owner)]", "B001"),
    ] {
        let case = Case::new(&format!("{prefix}{body}"));
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
pub fn mutable_allocator_restarts_preserve_effects_and_empty_versions() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
get<m.Allocator>:(value<&int32>){d.print(*value);->m.heap}
context:=7
handle:=m.heap
n:=2
'loop{
    |n==2|handle=get(&context)
    |n==1|handle=m.heap
    copy:handle
    d.print(n)
    n=n-1
    |n>0|'loop.restart()
}
copy:handle
context=8
d.print(context)
"#,
    )
    .runs(b"7\n2\n1\n8\n");
}

#[test]
pub fn mutable_allocator_leave_keeps_prior_write_and_skips_outer_store() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
get<m.Allocator>:(value<&int32>){d.print(*value);->m.heap}
context:=7
handle:=m.heap
'out{
    handle={handle=get(&context);d.print("leave");'out.leave();->m.heap}
    d.print("late")
}
copy:handle
context=8
d.print(context)
"#,
    )
    .runs(b"7\nleave\n8\n");
}

#[test]
pub fn overwritten_allocator_expiry_and_live_cell_conflicts_are_distinct() {
    let prefix = r#"m:@"memory";get<m.Allocator>:(value<&int32>){->m.heap};"#;
    Case::new(&format!("{prefix}h:=m.heap;again:=true;'loop{{local:1;|again|{{again=false;h=get(&local);'loop.restart()}}}};h=m.heap;copy:h")).runs(b"");
    for (body, code) in [
        (
            "h:=m.heap;again:=true;'loop{local:1;|again|{again=false;h=get(&local);'loop.restart()}};copy:h",
            "E303",
        ),
        (
            "h:=m.heap;old:=m.heap;{local:1;h=get(&local);old=h};h=m.heap;copy:old",
            "E303",
        ),
        ("context:1;h:=get(&context);p:&h;h=m.heap;copy:*p", "E302"),
    ] {
        let case = Case::new(&format!("{prefix}{body}"));
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
pub fn nullable_allocator_header_overwrite_ends_the_old_bound_path() {
    Case::new(
        r#"
m:@"memory"
d:@"debug"
get<m.Allocator>:(context<&int32>){->m.heap}
a<m.Allocator><null>:get(&1)
b<m.Allocator><null>:null
view:=&a
again:=true
'loop{
    view=&b
    current:*view
    |current<null>|d.print("empty")
    |again|{again=false;'loop.restart()}
}
"#,
    )
    .runs(b"empty\nempty\n");
}
