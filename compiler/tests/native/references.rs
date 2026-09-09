use super::Case;

#[test]
pub fn shared_references_preserve_identity_and_copy_referents() {
    Case::new(include_str!(
        "../../../docs/conformance/sources/reference_identity.mwy"
    ))
    .runs(b"true\nfalse\n");
    Case::new(
        r#"
debug:@"debug"
a:41
b:41
left<&int32>:&a
copy:left
debug.print(copy==left)
debug.print(copy==&b)
debug.print(*copy+1)
record:{->small<uint8>:7;->nested:{->value<int64>:99;->active:true}}
view:&record
field:&(record.nested.value)
same:&(record.nested.value)
other:&(record.nested.active)
debug.print(field==same)
debug.print(*field)
debug.print(*other)
debug.print(view.small)
snapshot:*view
debug.print(snapshot.nested.value)
|true|{a:17;inner:&a;debug.print(*inner);debug.print(copy==&a)}
debug.print(*copy)
read<int32>:(){owner:12;ref:&owner;->*ref}
debug.print(read())
"#,
    )
    .runs(b"true\nfalse\n42\ntrue\n99\ntrue\n7\n99\n17\nfalse\n41\n12\n");
}

#[test]
pub fn shared_reference_scope_checks_run_before_native_lowering() {
    for (source, code) in [
        ("view:{owner:1;->&owner}", "E303"),
        ("bad:(){owner:1;ref:&owner;alias:ref;->alias}", "E303"),
        ("owner:=1;view:&!owner;owner=2;value:*view", "E302"),
        ("view:&(1+2);value:*view", "E303"),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let result = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(result.status.code(), Some(1), "{source}");
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(
                error.contains(&format!("\"code\":\"{code}\"")),
                "{source}: {error}"
            );
            assert!(!case.path.join("build").exists());
        }
    }
}

#[test]
pub fn guarded_block_results_preserve_all_reference_origins() {
    Case::new(
        r#"
debug:@"debug"
choose<int32>:(flag<boolean>){
    left:11
    right:22
    view:{|flag|->&left;|!flag|->&right}
    alias:{->view}
    debug.print(alias==&left)
    ->*alias
}
debug.print(choose(true))
debug.print(choose(false))
owner:{->value:37}
view:'result{{'result->&(owner.value);'result.leave()}}
debug.print(*view)
copy:{->view}
debug.print(copy==view)
"#,
    )
    .runs(b"true\n11\nfalse\n22\n37\ntrue\n");
}

#[test]
pub fn discarded_reference_results_preserve_effects_and_owner_lifetimes() {
    Case::new(
        r#"
debug:@"debug"
owner:42
again:=true
view:'result {
    |again|{
        local:1
        debug.print("retry")
        'result->&local
        again=false
        'result.restart()
    }
    ->&owner
}
debug.print(*view)
debug.print(view==&owner)
'outer {
    {local:8;->&local;debug.print("leave");'outer.leave()}
}
flag:false
'scope {local:1;|flag|->&local}
debug.print("done")
"#,
    )
    .runs(b"retry\n42\ntrue\nleave\ndone\n");
    let case = Case::new("debug:@\"debug\";{local:1;->&local;debug.panic(\"stop\")}");
    for profile in ["debug", "release"] {
        let result = case.command("run", &["--profile", profile]);
        assert_eq!(result.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&result.stderr).contains("P006"));
    }
}

#[test]
pub fn guarded_local_escapes_are_rejected_before_lowering() {
    Case::new("owner:1;view:={->field:&owner}").runs(b"");
    Case::new("f<null>:(flag<boolean>){owner:1;view:={|flag|->&owner}}").runs(b"");
    for (source, code) in [
        (
            "bad:(flag<boolean>){outer:1;view:'result{|flag|{local:2;'result->&local};|!flag|->&outer};->*view}",
            "E303",
        ),
        (
            "bad:(flag<boolean>){view:{local:2;|flag|->&local;|!flag|->&local};->*view}",
            "E303",
        ),
        ("'result{{local:1;'result->&local;'result.leave()}}", "E303"),
    ] {
        let case = Case::new(source);
        let result = case.command("check", &["--json"]);
        assert_eq!(result.status.code(), Some(1), "{source}");
        let errors = String::from_utf8_lossy(&result.stderr);
        assert!(
            errors.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {errors}"
        );
    }
}

#[test]
pub fn shared_borrows_end_after_final_use_in_assignments_and_operands() {
    Case::new(
        r#"
debug:@"debug"
owner:=1
unused:&owner
owner=2
view:&owner
alias:{->view}
owner=*alias+1
debug.print(owner)
current:&owner
sum:*current+{owner=4;->owner}
debug.print(sum)
debug.print(owner)
last:&owner
skipped:false&&{owner=9;->true}
debug.print(skipped)
debug.print(*last)
owner=5
debug.print(owner)
"#,
    )
    .runs(b"3\n7\n4\nfalse\n4\n5\n");
}

#[test]
pub fn shared_borrow_liveness_follows_loop_and_named_leave_edges() {
    Case::new(
        r#"
debug:@"debug"
owner:=10
view:&owner
'finished {
    debug.print(*view)
    'finished.leave()
    owner=99
    debug.print(*view)
}
owner=11
debug.print(owner)
count:=0
'loop {
    count=count+1
    current:&count
    debug.print(*current)
    |count<3|'loop.restart()
}
count=4
debug.print(count)
owner=20
result:'result {
    'result->&owner
    'result.leave()
    owner=99
}
debug.print(*result)
owner=21
debug.print(owner)
"#,
    )
    .runs(b"10\n11\n1\n2\n3\n4\n20\n21\n");
}

#[test]
pub fn guarded_borrows_allow_writes_to_unselected_storage() {
    Case::new(
        r#"
debug:@"debug"
choose<int32>:(flag<boolean>){
    left:=11
    right:=22
    view:{|flag|->&left;|!flag|->&right}
    |flag|right=33
    |!flag|left=44
    ->*view
}
disjoint<null>:(flag<boolean>){
    owner:=7
    view:&owner
    |flag|owner=8
    |!flag|debug.print(*view)
    debug.print(owner)
}
debug.print(choose(true))
debug.print(choose(false))
disjoint(true)
disjoint(false)
"#,
    )
    .runs(b"11\n22\n8\n7\n7\n");
}

#[test]
pub fn live_shared_borrows_reject_overlapping_writes_before_lowering() {
    for source in [
        "owner:=1;view:&owner;owner=2;value:*view",
        "owner:=1;view:&owner;alias:view;owner=2;value:*alias",
        "owner:=1;view:&owner;owner=*view+1;value:*view",
        "owner:={->value:1;->other:2};view:&(owner.value);owner={->value:3;->other:4};value:*view",
        "owner:=1;view:{->&owner;owner=2};value:*view",
        "owner:=1;view:&owner;other:3;same:view=={owner=2;->&other}",
        "owner:=1;other:3;same:&owner=={owner=2;->&other}",
        "owner:=1;view:&owner;again:=true;'loop{value:*view;|again|{owner=2;again=false;'loop.restart()}}",
        "d:@\"debug\";owner:=1;view:&owner;later:=false;i:=0;'loop{|later|d.print(*view);|!later|owner=2;later=true;i=i+1;|i<2|'loop.restart()}",
        "f<null>:(flag<boolean>){owner:=1;view:&owner;flag&&{owner=2;->true};value:*view}",
        "f<int32>:(flag<boolean>){left:=1;right:=2;view:{|flag|->&left;|!flag|->&right};left=3;->*view}",
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let result = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(result.status.code(), Some(1), "{source}");
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(error.contains("\"code\":\"E302\""), "{source}: {error}");
            assert!(!case.path.join("build").exists());
        }
    }
}
