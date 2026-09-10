use super::file_modules::case;

#[test]
pub(crate) fn exported_calls_example_keeps_recursion_reexports_and_exclusive_arguments() {
    case(
        include_str!("../../examples/function-modules/main.mwy"),
        &[
            (
                "api.mwy",
                include_str!("../../examples/function-modules/api.mwy"),
            ),
            (
                "ops.mwy",
                include_str!("../../examples/function-modules/ops.mwy"),
            ),
        ],
    )
    .runs(b"ops\napi\nmain\n1\n7\n4\n10\n");
}

#[test]
pub(crate) fn exported_calls_keep_all_input_bounds_and_last_use() {
    let ops = "->keep<&int32>:(value<&int32>,label<&string>){->value}";
    case("m:@\"./ops.mwy\";d:@\"debug\";x:=7;label:=\"old\";view:m.keep(&x,&label);d.print(*view);label=\"new\";x=8;d.print(label)", &[("ops.mwy",ops)]).runs(b"7\nnew\n");
    for source in [
        "m:@\"./ops.mwy\";x:=7;label:=\"old\";view:m.keep(&x,&label);label=\"new\";v:*view",
        "m:@\"./ops.mwy\";x:=7;label:=\"old\";view:m.keep(&x,&label);x=8;v:*view",
    ] {
        let case = case(source, &[("ops.mwy", ops)]);
        for profile in ["debug", "release"] {
            let output = case.command("check", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E302\""));
        }
    }
}

#[test]
pub(crate) fn exported_calls_reject_private_storage_escape_and_module_data_capture() {
    for (ops, code) in [
        ("->bad<&int32>:(){x:1;->&x}", "E303"),
        ("x:=1;->bad<int32>:(){->x}", "B001"),
        ("data:@\"./data.mwy\";->bad<int32>:(){->data.n}", "B001"),
    ] {
        let case = case(
            "m:@\"./ops.mwy\"",
            &[("ops.mwy", ops), ("data.mwy", "->n:7")],
        );
        for profile in ["debug", "release"] {
            let output = case.command("check", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(error.contains(&format!("\"code\":\"{code}\"")), "{error}");
            assert!(
                error.contains(&format!(
                    "\"path\":\"{}\"",
                    case.path.join("ops.mwy").display()
                )),
                "{error}"
            );
        }
    }
}

#[test]
pub(crate) fn exported_calls_keep_callee_panic_sites_through_facades() {
    let ops = "d:@\"debug\";d.print(\"init\");->fail<never>:(){d.panic(\"inner\")}";
    let case = case(
        "api:@\"./api.mwy\";d:@\"debug\";d.panic(\"outer {api.fail()} never\")",
        &[
            ("api.mwy", "ops:@\"./ops.mwy\";->fail<()->never>:ops.fail"),
            ("ops.mwy", ops),
        ],
    );
    let start = ops.find("d.panic").unwrap();
    let end = start + "d.panic(\"inner\")".len();
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"init\n");
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: outer panic[P006]: inner at \"{}\" bytes {start}..{end}\n",
                case.path.join("ops.mwy").display()
            )
            .as_bytes()
        );
    }
}
