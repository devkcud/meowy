use super::{Case, file_modules::case};

#[test]
pub(crate) fn computed_types_exports_preserve_aliases_initialization_and_storage() {
    case(
        "m:@\"./facade.mwy\";d:@\"debug\";rows<m.Rows>:[{->n:3},{->n:7}];d.print(rows[2].n)",
        &[
            ("facade.mwy", "m:@\"./types.mwy\";#| [[<m.Row>]] |#-><Rows>:{element:<m.Row>;-><(element)[4]>}"),
            ("types.mwy", "d:@\"debug\";d.print(\"init\");#| Record. |#-><Row>:{<Number>:<int32>;-><{n<Number>}>}"),
        ],
    ).runs(b"init\n7\n");
}

#[test]
pub(crate) fn computed_types_queries_and_references_keep_runtime_values_at_runtime() {
    Case::new("d:@\"debug\";f<int32>:(n<int32>){<T>:{->n<>};x<T>:n;->x};<Ref>:{-><&int32>};n:7;r<Ref>:&n;d.print(f(*r))").runs(b"7\n");
    let case = Case::new("f<int32>:(n<int32>){<T>:{->n};->1}");
    let output = case.command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E211\""));
}

#[test]
pub(crate) fn computed_types_effects_report_dependency_spans_before_any_execution() {
    let source = "#é#\nd:@\"debug\";d.print(\"init\");-><T>:{-><int32>;d.panic(\"forbidden\")}";
    let case = case("m:@\"./types.mwy\"", &[("types.mwy", source)]);
    for action in ["check", "build", "run"] {
        let output = case.command(action, &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E219\""), "{error}");
        assert!(
            error.contains(&format!(
                "\"path\":\"{}\"",
                case.path.join("types.mwy").display()
            )),
            "{error}"
        );
        assert!(
            error.contains(&format!("\"start\":{}", source.find("d.panic").unwrap())),
            "{error}"
        );
    }
}

#[test]
pub(crate) fn computed_types_documentation_checks_local_links_and_derived_signatures() {
    let case = Case::new(include_str!("../../examples/computed-types.mwy"));
    case.runs(b"computed\n7\n");
    let output = super::documentation::doc(&case, "check", &["--standalone"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bad = Case::new("#| Outer. |#<T>:{#| [[missing]] |#element:<int32>;->element}");
    let output = bad.command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E802\""));
}

#[test]
pub(crate) fn computed_types_budget_failure_stays_bootstrap_and_keeps_the_dependency_path() {
    let binds = (0..2200)
        .map(|id| format!("t{id}:<int32>;"))
        .collect::<String>();
    let source = format!("-><T>:{{{binds}-><int32>}}");
    let case = case("m:@\"./types.mwy\"", &[("types.mwy", &source)]);
    let output = case.command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("\"code\":\"B001\""), "{error}");
    assert!(error.contains("computed type bootstrap budget"), "{error}");
    assert!(
        error.contains(&format!(
            "\"path\":\"{}\"",
            case.path.join("types.mwy").display()
        )),
        "{error}"
    );
}
