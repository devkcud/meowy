use super::Case;

#[test]
pub fn reference_record_projections_end_unrelated_component_loans() {
    Case::new(
        r#"
d:@"debug"
a:=1
b:=2
pair:{->left:&a;->right:&b;->count:3}
b=20
d.print(*pair.left)
a=10
d.print(pair.count)
a=11
record<{value<&int32>;nested<{other<&int32>}>}>:{->value:&a;->nested:{->other:&b}}
copy:{->record}
leaf:copy.nested.other
a=12
d.print(*leaf)
b=21
d.print(a)
d.print(b)
primary:{->3;->view:&a}
a=13
number<int32>:primary
d.print(number)
d.print("primary: {primary}")
outer:9
selected:{local:5;record:{->safe:&outer;->local:&local};->record.safe}
d.print(*selected)
"#,
    )
    .runs(b"1\n3\n20\n12\n21\n3\nprimary: 3\n9\n");
}

#[test]
pub fn reference_record_fields_preserve_guarded_origins_and_named_results() {
    Case::new(
        r#"
d:@"debug"
choose<int32>:(flag<boolean>){
    left:=11
    right:=22
    other:=7
    pair:{|flag|->value:&left;|!flag|->value:&right;->other:&other}
    |flag|right=33
    |!flag|left=44
    other=8
    ->*pair.value
}
d.print(choose(true))
d.print(choose(false))
owner:=42
again:=true
result:'result {
    |again|{
        local:0
        'result->nested:{->view:&local}
        again=false
        'result.restart()
    }
    'result->nested:{->view:&owner}
    'result.leave()
}
d.print(*result.nested.view)
owner=43
d.print(owner)
"#,
    )
    .runs(b"11\n22\n42\n43\n");
}

#[test]
pub fn reference_record_copies_and_pending_results_protect_all_live_components() {
    for source in [
        "a:=1;b:=2;pair:{->left:&a;->right:&b};b=3;copy:pair;value:*copy.left",
        "a:=1;pair:{->view:&a};a=2;value:*pair.view",
        "a:=1;pair:{->view:&a;a=2};value:*pair.view",
        "a:=1;b:2;pair:{->view:&a};same:pair=={a=2;->view:&b}",
        "a:=1;b:2;left:{->view:&a};right:{->view:&b};same:left=={a=3;->right}",
        "a:=1;b:=2;pair:{->nested:{->left:&a;->right:&b}};b=3;copy:pair.nested;value:*copy.left",
        "d:@\"debug\";a:=1;pair:{->view:&a};i:=0;'loop{d.print(*pair.view);a=2;i=i+1;|i<2|'loop.restart()}",
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

#[test]
pub fn reference_record_local_escapes_and_unsupported_contracts_stay_explicit() {
    Case::new("owner:1;pair:={->view:&owner}").runs(b"");
    Case::new("f<null>:(flag<boolean>){owner:1;pair:={|flag|->view:&owner}}").runs(b"");
    for (source, code) in [
        ("pair:{local:1;->view:&local}", "E303"),
        (
            "pair:{local:1;->nested:{->view:&local};->count:3};value:pair.count",
            "E303",
        ),
        (
            "outer:9;pair:{local:5;record:{->safe:&outer;->local:&local};->record};value:*pair.safe",
            "E303",
        ),
        ("bad:(){owner:1;pair:{->view:&owner};->pair}", "E303"),
    ] {
        let result = Case::new(source).command("check", &["--json"]);
        assert_eq!(result.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}

#[test]
pub fn reference_record_equality_preserves_fields_through_block_returns() {
    Case::new(
        r#"
d:@"debug"
a:1
b:2
left:{->view:&a}
right:{->view:&b}
d.print(left==right)
d.print(left=={->right})
d.print(left!={->view:&b})
d.print(left=={->left})
first:{->7;->view:&a}
second:{->7;->view:&b}
d.print(first=={->second})
d.print(first=={->first})
x:=3
packet:{->7;->view:&x}
x=4
d.print(packet=={->7})
d.print({->7}==packet)
"#,
    )
    .runs(b"false\nfalse\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\n");
}
