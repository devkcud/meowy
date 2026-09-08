use super::{accepts, rejects};

#[test]
pub(crate) fn foundation_aliases_keep_module_item_and_type_identity() {
    accepts(
        r#"m:@"memory";s:@"strings";alias:m;heap:alias.heap;copy:s.copy;again:copy;<A>:<alias.Allocator>;<B>:<A>;<E>:<m.AllocationFailure>;<S>:<s.Owned>;kind:s.Owned;<T>:<(kind)>"#,
    );
    let source = r#"m:@"memory";s:@"strings";copy:s.copy;again:copy;again("text",m.heap)"#;
    let errors = crate::compile(source).unwrap_err();
    assert_eq!(errors[0].code, "B001");
    assert!(errors[0].message.contains("strings.copy"));
}

#[test]
pub(crate) fn foundation_storage_and_construction_stay_gated() {
    for source in [
        r#"m:@"memory";s:@"strings";s.copy("text",m.heap)"#,
        r#"s:@"strings";<S>:<s.Owned>;f<S>:(){->null}"#,
        r#"m:@"memory";heap<m.Allocator>:m.heap"#,
        r#"m:@"memory";<E>:<m.AllocationFailure>;f:(e<E>){->e}"#,
        r#"s:@"strings";<S>:<s.Owned>;x<&S>:null"#,
        r#"s:@"strings";x:null;found:x<s.Owned>"#,
        r#"s:@"strings";copy:=s.copy"#,
        r#"m:@"memory";m.heap"#,
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub(crate) fn module_shadowing_does_not_forge_foundational_identities() {
    accepts(r#"m:@"memory";alias:m;{m:{->heap:7};value:m.heap};heap:alias.heap"#);
    accepts(r#"strings:@"debug";strings.print("ordinary module alias")"#);
    rejects(r#"s:@"strings";s.missing"#, "B001");
    rejects(r#"m:@"memory";<Bad>:<m.Missing>"#, "B001");
    rejects(r#"m:{->Allocator:1};<Bad>:<m.Allocator>"#, "E202");
    rejects(r#"s:{->copy:1};s.copy("text")"#, "B001");
    rejects(r#"m:@"memory";{m:@"debug";<Bad>:<m.Allocator>}"#, "E202");
    rejects(r#"s:@"unavailable""#, "B001");
}
