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
        ("owner:1;result:=get(&owner)", "B001"),
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
