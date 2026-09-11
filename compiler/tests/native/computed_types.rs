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

#[test]
pub(crate) fn computed_integers_export_calculated_capacities_and_preserve_large_widths() {
    case(
        "m:@\"./types.mwy\";d:@\"debug\";items<m.Items>:[3,7];d.print(items[2]);v<m.Wide>:4294967297;d.print(v)",
        &[("types.mwy", "-><Items>:{base<uint8>:2;capacity:base*2;-><int32[capacity]>};-><Wide>:{base<uint64>:4294967296;copy:base+1;->copy<>}")],
    ).runs(b"7\n4294967297\n");
}

#[test]
pub(crate) fn computed_integers_report_precise_dependency_failures_without_initialization() {
    for (source, code, text) in [
        (
            "#é#\nd:@\"debug\";d.print(\"init\");-><T>:{n<uint8>:255;capacity:n+1;-><int32>}",
            "E107",
            "n+1",
        ),
        (
            "d:@\"debug\";d.print(\"init\");f<int32>:(n<int32>){<T>:{capacity:n+1;-><int32>};->1}",
            "E211",
            "n+1",
        ),
        (
            "d:@\"debug\";d.print(\"init\");-><T>:{capacity<int32>:d.panic(\"bad\");-><int32>}",
            "E219",
            "d.panic",
        ),
    ] {
        let case = case("m:@\"./types.mwy\"", &[("types.mwy", source)]);
        for action in ["check", "build", "run"] {
            let output = case.command(action, &["--json"]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(error.contains(&format!("\"code\":\"{code}\"")), "{error}");
            assert!(
                error.contains(&format!(
                    "\"path\":\"{}\"",
                    case.path.join("types.mwy").display()
                )),
                "{error}"
            );
            assert!(
                error.contains(&format!("\"start\":{}", source.find(text).unwrap())),
                "{error}"
            );
        }
    }
}

#[test]
pub(crate) fn computed_integers_refuse_unproven_folded_blocks_as_static_inputs() {
    for source in [
        "n:{->4};<T>:{capacity:n+1;-><int32[capacity]>}",
        "n:{->4};<T>:{-><int32[n]>}",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"B001\""), "{error}");
        assert!(error.contains("runtime initializer eligibility"), "{error}");
    }
}

#[test]
pub(crate) fn initializer_inputs_cross_function_scopes_without_repeating_initializers() {
    case(
        "m:@\"./types.mwy\";d:@\"debug\";values<m.Items>:[1,2];d.print(m.get());d.print(values[2])",
        &[("types.mwy", "d:@\"debug\";d.print(\"types\");base<uint8>:2;capacity:base*2;-><Items>:{n:capacity;-><int32[n]>};->get<int32>:(){<Local>:{n:capacity;-><int32[n]>};values<Local>:[4,7];->values[2]}")],
    ).runs(b"types\n7\n2\n");
}

#[test]
pub(crate) fn initializer_inputs_check_and_build_without_running_application_effects() {
    let case = Case::new(
        "d:@\"debug\";d.panic(\"application startup\");capacity:4;<T>:{n:capacity;-><int32[n]>}",
    );
    for profile in ["debug", "release"] {
        for action in ["check", "build"] {
            let output = case.command(action, &["--profile", profile]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty());
            assert!(output.stderr.is_empty());
        }
        let output = case.command("run", &["--profile", profile]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("application startup"));
    }
}

#[test]
pub(crate) fn initializer_inputs_report_hidden_dependency_overflow_at_its_source() {
    let source = "#é#\n|false|{bad<uint8>:255+1;alias:bad;<T>:{n:alias;-><int32>}}";
    let case = case("m:@\"./types.mwy\"", &[("types.mwy", source)]);
    for action in ["check", "build", "run"] {
        let output = case.command(action, &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E107\""), "{error}");
        assert!(
            error.contains(&format!(
                "\"path\":\"{}\"",
                case.path.join("types.mwy").display()
            )),
            "{error}"
        );
        assert!(
            error.contains(&format!("\"start\":{}", source.find("255+1").unwrap())),
            "{error}"
        );
    }
}

#[test]
pub(crate) fn initializer_inputs_keep_runtime_captures_and_mutable_dependencies_unavailable() {
    for (source, code) in [
        ("capacity:4;f<int32>:(){->capacity}", "B001"),
        (
            "capacity:=4;copy:capacity;<T>:{n:copy;-><int32[n]>}",
            "E211",
        ),
        (
            "f<int32>:(capacity<int32>){copy:capacity;<T>:{n:copy;-><int32[n]>};->1}",
            "E211",
        ),
        (
            "d:@\"debug\";capacity:{d.print(1);->4};copy:capacity;<T>:{n:copy;-><int32[n]>}",
            "E211",
        ),
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {error}"
        );
    }
}
