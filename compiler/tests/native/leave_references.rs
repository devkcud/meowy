use super::Case;

pub(crate) fn rejects(source: &str, code: &str) {
    let output = Case::new(source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1), "{source}");
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error
            .lines()
            .next()
            .unwrap_or_default()
            .contains(&format!("\"code\":\"{code}\"")),
        "{source}: {error}"
    );
}

#[test]
pub fn leaves_merge_skipped_and_completed_reference_assignments() {
    Case::new(
        r#"
d:@"debug"
choose<int32>:(flag<boolean>){
    a:1;b:2;p:=&a
    'out{|flag|'out.leave();p=&b}
    ->*p
}
assigned<int32>:(flag<boolean>){
    a:1;b:2;c:3;p:=&a
    'out{|flag|{p=&b;'out.leave()};p=&c}
    ->*p
}
d.print(choose(false));d.print(choose(true))
d.print(assigned(false));d.print(assigned(true))
nested<int32>:(first<boolean>,second<boolean>){
    a:4;b:5;c:6;p:=&a
    'outer{
        'inner{
            |first|{p=&b;'inner.leave()}
            |second|{p=&c;'outer.leave()}
            p=&b
        }
        p=&a
    }
    ->*p
}
d.print(nested(false,false));d.print(nested(false,true))
d.print(nested(true,false));d.print(nested(true,true))
"#,
    )
    .runs(b"2\n1\n3\n2\n4\n6\n4\n4\n");
}

#[test]
pub fn rhs_leaves_keep_prior_effects_and_skip_unfinished_stores_and_calls() {
    Case::new(
        r#"
d:@"debug"
rhs<int32>:(flag<boolean>){
    a:1;b:2;c:3;p:=&a
    'out{
        p={d.print("rhs");p=&b;|flag|'out.leave();->&c}
        d.print("stored")
    }
    ->*p
}
d.print(rhs(false));d.print(rhs(true))
same<boolean>:(a<&int32>,b<&int32>){d.print("call");->a==b}
call<int32>:(flag<boolean>){
    a:4;b:5;p:=&a
    'out{
        value:same(p,{p=&b;|flag|'out.leave();->p})
        d.print(value)
    }
    ->*p
}
d.print(call(false));d.print(call(true))
"#,
    )
    .runs(b"rhs\nstored\n3\nrhs\n2\ncall\nfalse\n5\n5\n");
}

#[test]
pub fn index_and_short_circuit_leaves_restore_only_the_target_continuation() {
    Case::new(
        r#"
d:@"debug"
index<int32>:(flag<boolean>){
    a:7;b:8;p:=&a;values:=[1,2]
    'out{
        values[{p=&b;|flag|'out.leave();->1}]={d.print("rhs");->3}
    }
    d.print(values[1])
    ->*p
}
d.print(index(false));d.print(index(true))
short<int32>:(flag<boolean>){
    a:1;b:2;c:3;p:=&a
    'out{value:flag&&{p=&b;'out.leave();->true};p=&c}
    ->*p
}
d.print(short(false));d.print(short(true))
"#,
    )
    .runs(b"rhs\n3\n8\n1\n8\n3\n2\n");
}

#[test]
pub fn leave_versions_preserve_guarded_owner_and_physical_cell_loans() {
    Case::new(
        r#"
d:@"debug"
choose<int32>:(flag<boolean>){
    a:=1;b:=2;p:=&a
    'out{|flag|'out.leave();p=&b}
    |flag|b=3
    |!flag|a=4
    ->*p
}
d.print(choose(false));d.print(choose(true))
cell<int32>:(flag<boolean>){
    a:5;b:6;p:=&a;view:&p
    'out{|flag|'out.leave();p={d.print(**view);->&b}}
    |flag|d.print(**view)
    ->*p
}
d.print(cell(false));d.print(cell(true))
"#,
    )
    .runs(b"2\n1\n5\n6\n5\n5\n");
    for source in [
        "f<int32>:(flag<boolean>){a:=1;b:2;p:=&a;'out{|flag|'out.leave();p=&b};a=3;->*p}",
        "f<int32>:(flag<boolean>){a:=1;b:2;p:=&a;old:p;'out{|flag|'out.leave();p=&b};a=3;->*old}",
        "f<int32>:(flag<boolean>){a:1;b:2;p:=&a;cell:&p;'out{|flag|'out.leave();p=&b};->**cell}",
        "<R>:<&int32>;load<R>:(p<&R>){->*p};f<int32>:(flag<boolean>){a:1;b:2;p:=&a;old:load(&p);'out{|flag|'out.leave();p=&b};->*old}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn leave_results_preserve_initialized_emissions_and_reference_loans() {
    Case::new(
        r#"
d:@"debug"
choose<int32>:(flag<boolean>)'out{
    a:1;b:2;p:=&a
    |flag|{p=&b;'out->*p;'out.leave()}
    ->*p
}
d.print(choose(false));d.print(choose(true))
record<null>:(flag<boolean>){
    a:3;b:4;p:=&a
    result:'out{
        |flag|{p=&b;'out->view:p;'out->count:2;'out.leave()}
        ->view:p;->count:1
    }
    d.print(*result.view);d.print(result.count)
}
record(false);record(true)
"#,
    )
    .runs(b"1\n2\n3\n1\n4\n2\n");
    for source in [
        "f<int32>:(flag<boolean>){a:=1;b:=2;p:=&a;value:'out{|flag|{p=&b;'out->p;'out.leave()};->p};a=3;b=3;->*value}",
        "a:=1;b:2;p:=&a;'out{->held:p;p=&b;a=3;'out.leave()}",
    ] {
        rejects(source, "E302");
    }
}

#[test]
pub fn leave_snapshots_do_not_extend_source_or_temporary_lifetimes() {
    Case::new(
        r#"
d:@"debug"
a:=1;b:2;p:=&a
'out{a=3;'out.leave()}
p=&b
d.print(*p)
'out{local:4;p=&local;'out.leave()}
p=&a
d.print(*p)
choose<int32>:(flag<boolean>){
    a:5;p:=&1
    'out{|flag|{p=&a;'out.leave()};p=&a}
    ->*p
}
d.print(choose(false));d.print(choose(true))
"#,
    )
    .runs(b"2\n3\n5\n5\n");
    for source in [
        "a:1;p:=&a;'out{local:2;p=&local;'out.leave()};value:*p",
        "a:1;p:=&a;'out{p=&2;'out.leave()};value:*p",
        "f<int32>:(flag<boolean>){a:1;p:=&a;'out{|flag|'out.leave();p=&2};->*p}",
        "f<int32>:(flag<boolean>){a:1;p:=&2;'out{|flag|'out.leave();p=&a};->*p}",
        "first<&int32>:(p<&int32>,q<&string>){->p};a:1;p:=&a;'out{p=first(&a,&\"short\");'out.leave()};value:*p",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub fn leave_slot_lifetimes_follow_the_target_instead_of_the_alias_scope() {
    Case::new(
        r#"
d:@"debug"
a:1;p:=&a
'outer{
    'inner{'outer->n:7;p=&n;'inner.leave()}
    d.print(*p)
}
p=&a
d.print(*p)
"#,
    )
    .runs(b"7\n1\n");
    for source in [
        "a:1;p:=&a;'out{->n:2;p=&n;'out.leave()};value:*p",
        "a:1;p:=&a;'out{'inner{->n:2;p=&n;'out.leave()}};value:*p",
        "a:1;p:=&a;value:'out{->n:2;p=&n;->saved:p;'out.leave()}",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub fn panic_discards_leave_continuations_and_restart_limits_stay_explicit() {
    let source = "d:@\"debug\";a:1;b:2;p:=&a;'out{p=&b;d.print(*p);d.panic(\"stop\");'out.leave()};d.print(\"after\")";
    let call = "d.panic(\"stop\")";
    let start = source.find(call).unwrap();
    let end = start + call.len();
    for profile in ["debug", "release"] {
        let output = Case::new(source).command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"2\n");
        assert_eq!(
            output.stderr,
            format!("panic[P006]: stop at bytes {start}..{end}\n").as_bytes()
        );
    }
    for source in [
        "a:1;p:=&a;'loop{local:2;p=&local;'loop.restart()}",
        "a:1;p:=&a;'out{p=&2;'inner{'out.restart()}}",
        "a:1;holder:={->view:&a};'out{'out.leave()}",
        "a:1;p<&int32><null>:=&a;'out{'out.leave()}",
    ] {
        rejects(source, "B001");
    }
}
