use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static NEXT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Case {
    pub(crate) path: PathBuf,
    pub(crate) source: PathBuf,
}

impl Case {
    pub(crate) fn new(source: &str) -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("meowy-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        let entry = path.join("main.mwy");
        fs::write(&entry, source).unwrap();
        Self {
            path,
            source: entry,
        }
    }

    pub(crate) fn command(&self, action: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_meowy"))
            .arg(action)
            .arg(&self.source)
            .args(["--standalone", "--quiet", "--color", "never"])
            .args(args)
            .current_dir(&self.path)
            .output()
            .unwrap()
    }

    pub(crate) fn runs(&self, expected: &[u8]) {
        for profile in ["debug", "release"] {
            let result = self.command("run", &["--profile", profile]);
            assert!(
                result.status.success(),
                "{profile}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stdout, expected, "{profile}");
            assert!(
                result.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
pub fn examples_execute_in_both_profiles() {
    for (source, stdout) in [
        (include_str!("../examples/hello.mwy"), "Hello, meowy!\n"),
        (include_str!("../examples/factorial.mwy"), "3628800\n"),
        (
            include_str!("../examples/records.mwy"),
            "Sensor 7: 24 celsius\n48\n",
        ),
        (include_str!("../examples/loop.mwy"), "5050\n"),
        (
            include_str!("../examples/borrow-results.mwy"),
            "11\n22\n42\ntrue\n",
        ),
        (
            include_str!("../examples/borrow-liveness.mwy"),
            "41\n42\n7\n9\n0\n1\n2\n3\n",
        ),
        (
            include_str!("../examples/borrowed-records.mwy"),
            "11\n44\n33\n44\nvalue\n0\n1\n2\n3\n",
        ),
        (
            include_str!("../examples/optional-borrows.mwy"),
            "7\n8\nmissing\n8\n42\ntext\n",
        ),
        (
            include_str!("../examples/borrow-functions.mwy"),
            "11\n22\n12\n23\n12\n24\n",
        ),
        (
            include_str!("../examples/reborrows.mwy"),
            "true\n41\n7\n42\n8\n",
        ),
        (
            include_str!("../examples/scope-borrows.mwy"),
            "false\n7\n8\ntrue\n9\n10\n",
        ),
        (
            include_str!("../examples/bounded-lists.mwy"),
            "2\n3\nAda\nLin\ntrue\n0\n",
        ),
        (
            include_str!("../examples/list-unions.mwy"),
            "20\nmeowy\n300\n2\n",
        ),
        (
            include_str!("../examples/compound-lists.mwy"),
            "128\n-128\nfalse\n260\n",
        ),
        (
            include_str!("../examples/element-borrows.mwy"),
            "true\nfalse\n20\n10\n30\n",
        ),
        (
            include_str!("../examples/references.mwy"),
            "true\nfalse\n42\nmeowy\n",
        ),
        (
            include_str!("../examples/nullable.mwy"),
            "meowy\nnull\nmeowy\nguest\n",
        ),
    ] {
        Case::new(source).runs(stdout.as_bytes());
    }
}

#[test]
pub fn emissions_keep_executing_and_dispatch_evaluates_once() {
    Case::new(
        r#"
debug:@"debug"
source<int32>:(){debug.print("source");->21;debug.print("after")}
double<int32>:(x<int32>){->x*2}
debug.print(source().(double))
"#,
    )
    .runs(b"source\nafter\n42\n");
}

#[test]
pub fn intrinsics_alias_and_names_shadow_normally() {
    Case::new(
        r#"
core:@"core"
d:@"debug"
show:d.print
{
    true:"shadowed"
    show(true)
    show(core.true)
    debug:{->print:9}
    show(debug.print)
}
'done {
    finish:'done.leave
    show("done")
    finish()
    show("unreachable")
}
"#,
    )
    .runs(b"shadowed\ntrue\n9\ndone\n");
}

#[test]
pub fn booleans_short_circuit_and_floats_preserve_types() {
    Case::new(
        r#"
d:@"debug"
probe<boolean>:(){d.print("probe");->true}
d.print(false&&probe())
d.print(true||probe())
d.print(1.5+2.25)
d.print("hé\0".size())
"#,
    )
    .runs(b"false\ntrue\n3.75\n4\n");
}

#[test]
pub fn integer_boundaries_and_bitwise_values_are_exact() {
    Case::new(
        r#"
d:@"debug"
a<int8>:-128
b<uint64>:18446744073709551615
c<int64>:-9223372036854775808
d.print(a)
d.print(b)
d.print(c)
d.print((5&3)^8|2)
"#,
    )
    .runs(b"-128\n18446744073709551615\n-9223372036854775808\n11\n");
}

#[test]
pub fn dynamic_arithmetic_panics_in_both_profiles() {
    for (ty, body, args) in [
        ("int8", "x+1", "127"),
        ("uint8", "x+1", "255"),
        ("int32", "10/x", "0"),
        ("int32", "-x", "-2147483648"),
        ("int32", "x/-1", "-2147483648"),
    ] {
        let case = Case::new(&format!(
            "d:@\"debug\";f<{ty}>:(x<{ty}>){{->{body}}};d.print(f({args}))"
        ));
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(
                result.status.code(),
                Some(1),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("P002"),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

#[test]
pub fn arithmetic_panics_report_typed_original_operands_and_source_bytes() {
    for (ty, operation, args, evidence) in [
        (
            "int8",
            "left+right",
            "127,1",
            "int8 + overflow (left 127, right 1; range -128..127)",
        ),
        (
            "uint8",
            "left-right",
            "0,1",
            "uint8 - overflow (left 0, right 1; range 0..255)",
        ),
        (
            "int16",
            "left*right",
            "-32768,-1",
            "int16 * overflow (left -32768, right -1; range -32768..32767)",
        ),
        (
            "uint64",
            "left+right",
            "18446744073709551615,1",
            "uint64 + overflow (left 18446744073709551615, right 1; range 0..18446744073709551615)",
        ),
        (
            "int64",
            "left/right",
            "-9223372036854775808,-1",
            "int64 / overflow (left -9223372036854775808, right -1; range -9223372036854775808..9223372036854775807)",
        ),
        (
            "int8",
            "left/right",
            "-7,0",
            "int8 / zero divisor (left -7, right 0; range -128..127)",
        ),
        (
            "uint64",
            "left%right",
            "18446744073709551615,0",
            "uint64 % zero divisor (left 18446744073709551615, right 0; range 0..18446744073709551615)",
        ),
    ] {
        let source = format!(
            "#é🙂#\r\nd:@\"debug\";f<{ty}>:(left<{ty}>,right<{ty}>){{->{operation}}};d.print(f({args}))"
        );
        let start = source.find(operation).unwrap();
        let end = start + operation.len();
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{ty}: {profile}");
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                format!("panic[P002]: {evidence} at bytes {start}..{end}\n").as_bytes()
            );
        }
    }
    let source = "#é#\nd:@\"debug\";f<int8>:(value<int8>){->-value};d.print(f(-128))";
    let start = source.find("-value").unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("panic[P002]: int8 unary - overflow (value -128; range -128..127) at bytes {start}..{}\n", start + "-value".len()).as_bytes());
    }
}

#[test]
pub fn arithmetic_evidence_does_not_repeat_operand_effects() {
    let source = r#"
d:@"debug"
left<int8>:(){d.print("left");->127}
right<int8>:(){d.print("right");->1}
d.print(left()+right())
"#;
    let operation = "left()+right()";
    let start = source.find(operation).unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"left\nright\n");
        assert_eq!(output.stderr, format!("panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes {start}..{}\n", start + operation.len()).as_bytes());
    }
}

#[test]
pub fn explicit_panic_appends_call_site_after_streamed_effects() {
    let source = r#"#é🙂#
d:@"debug"
mark<string>:(value<string>){d.print(value);->value}
die:d.panic
die("first={mark("A")}, second={mark("B")}")
"#;
    let call = r#"die("first={mark("A")}, second={mark("B")}")"#;
    let start = source.find(call).unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"A\nB\n");
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: first=A, second=B at bytes {start}..{}\n",
                start + call.len()
            )
            .as_bytes()
        );
    }
}

#[test]
pub fn interrupted_panic_messages_do_not_append_the_outer_site() {
    let called = r#"d:@"debug";stop<never>:(){d.panic("inner")};d.panic("outer {stop()} tail")"#;
    let call = r#"d.panic("inner")"#;
    let start = called.find(call).unwrap();
    let case = Case::new(called);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: outer panic[P006]: inner at bytes {start}..{}\n",
                start + call.len()
            )
            .as_bytes()
        );
    }
    let nested = r#"d:@"debug";d.panic("outer {d.panic("inner")} tail")"#;
    let inner = r#"d.panic("inner")"#;
    let start = nested.find(inner).unwrap();
    let case = Case::new(nested);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: outer panic[P006]: inner at bytes {start}..{}\n",
                start + inner.len()
            )
            .as_bytes()
        );
    }
    let arithmetic =
        r#"d:@"debug";f<never>:(value<int8>){d.panic("before {value+1} after")};f(127)"#;
    let start = arithmetic.find("value+1").unwrap();
    let case = Case::new(arithmetic);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("panic[P006]: before panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes {start}..{}\n", start + "value+1".len()).as_bytes());
    }
    let source =
        r#"d:@"debug";'out {d.panic("before {{'out.leave();->0}} after")};d.print("continued")"#;
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert!(output.status.success());
        assert_eq!(output.stdout, b"continued\n");
        assert_eq!(output.stderr, b"panic[P006]: before ");
    }
}

#[test]
pub fn never_calls_evaluate_arguments_and_end_the_returning_continuation() {
    let source = r#"
d:@"debug"
mark<int32>:(value<int32>){d.print(value);->value}
stop<never>:(left<int32>,right<int32>){d.print(left+right);d.panic("stop")}
sum<int32>:(left<int32>,right<int32>){d.print("sum");->left+right}
d.print(sum(mark(1),stop(mark(2),mark(3))))
d.print("after")
"#;
    let call = r#"d.panic("stop")"#;
    let start = source.find(call).unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"1\n2\n3\n5\n");
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: stop at bytes {start}..{}\n",
                start + call.len()
            )
            .as_bytes()
        );
    }
}

#[test]
pub fn checking_never_runs_application_effects() {
    let case = Case::new("d:@\"debug\";d.print(\"must not run\");d.panic(\"must not panic\")");
    let result = case.command("check", &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stdout.is_empty());
    assert!(!case.path.join("build").exists());
}

#[test]
pub fn rejection_codes_and_original_byte_spans_are_stable() {
    for (source, code) in [
        ("x<int8>:-(128)", "E216"),
        ("value:1__0", "E001"),
        ("x:missing", "E201"),
        ("x:1;x:2", "E203"),
        ("x:{->1;->2}", "E205"),
        ("f<int32>:(){}", "E204"),
        ("x<int8>:127+1", "E107"),
        ("x<int32>:1;y<int64>:2;z:x+y", "E213"),
        ("|1|{}", "E215"),
        ("f<int32>:(){->1};v:f==f", "E222"),
    ] {
        let result = Case::new(source).command("check", &["--json"]);
        assert_eq!(result.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {stderr}"
        );
        assert!(result.stdout.is_empty());
    }
    let result = Case::new("#é#\r\nx:1__0").command("check", &["--json"]);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("\"start\":8"), "{stderr}");
    assert!(stderr.contains("\"line\":2"), "{stderr}");
}

#[test]
pub fn output_is_an_elf_and_remains_intact_when_rebuild_fails() {
    let case = Case::new("d:@\"debug\";d.print(42)");
    let output = case.path.join("app");
    let result = case.command(
        "build",
        &[
            "--output",
            output.to_str().unwrap(),
            "--emit-llvm",
            "app.ll",
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = fs::read(&output).unwrap();
    assert_eq!(&bytes[..4], b"\x7fELF");
    assert!(
        fs::read_to_string(case.path.join("app.ll"))
            .unwrap()
            .contains("target triple")
    );
    fs::write(&case.source, "x:unknown").unwrap();
    assert_eq!(
        case.command("build", &["--output", output.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(fs::read(&output).unwrap(), bytes);
    let failed = case.command("run", &[]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(failed.stdout.is_empty());
}

#[test]
pub fn protected_inputs_and_usage_errors_are_rejected() {
    let case = Case::new("x:1");
    assert_eq!(
        case.command("build", &["--output", case.source.to_str().unwrap()])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(fs::read_to_string(&case.source).unwrap(), "x:1");
    assert_eq!(
        case.command("build", &["--output", "mod.lock"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        case.command("check", &["--output", "app"]).status.code(),
        Some(2)
    );
    let result = case.command(
        "check",
        &["--target", "aarch64-unknown-linux-gnu", "--json"],
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("E507"));
    #[cfg(unix)]
    {
        let hard = case.path.join("hard");
        fs::hard_link(&case.source, &hard).unwrap();
        assert_eq!(
            case.command("build", &["--output", hard.to_str().unwrap()])
                .status
                .code(),
            Some(2)
        );
        let sym = case.path.join("sym");
        std::os::unix::fs::symlink(&case.source, &sym).unwrap();
        assert_eq!(
            case.command("build", &["--output", sym.to_str().unwrap()])
                .status
                .code(),
            Some(2)
        );
    }
}

#[test]
pub fn unavailable_features_and_manifests_are_explicit() {
    let case = Case::new("values<int32[]>:[]");
    let result = case.command("check", &["--json"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("B001"));
    fs::write(&case.source, "x:1").unwrap();
    fs::write(case.path.join("mod.mwy"), "->build:{->entry:\"main.mwy\"}").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_meowy"))
        .arg("check")
        .arg(&case.source)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("B001"));
    assert!(case.command("check", &[]).status.success());
}

#[test]
pub fn invalid_utf8_is_a_source_error() {
    let case = Case::new("");
    fs::write(&case.source, [b'x', b':', 0xff]).unwrap();
    let result = case.command("check", &["--json"]);
    assert_eq!(result.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("\"code\":\"E001\""));
    assert!(stderr.contains("\"start\":2"));
}

#[test]
pub fn conditional_fields_and_primaries_have_real_null_defaults() {
    Case::new(
        r#"
d:@"debug"
make:(flag<boolean>){|flag|->name:"hello"}
a<{name<string><null>}>:make(false)
b:make(true)
d.print(a.name)
d.print(b.name)
maybe<int8><null>:(flag<boolean>){|flag|->-128}
d.print(maybe(false))
d.print(maybe(true))
empty<{name<string><null>}>:{}
d.print(empty.name)
"#,
    )
    .runs(b"null\nhello\nnull\n-128\nnull\n");
}

#[test]
pub fn union_normalization_and_retagging_preserve_payloads() {
    Case::new(
        r#"
d:@"debug"
<Small>:<string><null><never><string>
<Wide>:<string><boolean><null>
widen<Wide>:(value<Small>){->value}
narrow<Small>:(value<Wide>) 'result {
    |value<boolean>|{'result->"boolean";'result.leave()}
    ->value
}
d.print(widen("payload"))
d.print(widen(null))
d.print(narrow(widen("same")))
d.print(narrow(true))
d.print(narrow(null))
"#,
    )
    .runs(b"payload\nnull\nsame\nboolean\nnull\n");
}

#[test]
pub fn union_equality_compares_only_the_active_variant() {
    Case::new(
        r#"
d:@"debug"
same<boolean>:(a<int32><string><null>,b<null><string><int32>){->a==b}
d.print(same(7,7))
d.print(same(7,"7"))
d.print(same("same","same"))
d.print(same("same","different"))
d.print(same(null,null))
d.print(same(null,""))
a<{name<string><null>}>:{}
b<{name<string><null>}>:{->name:""}
d.print(a==a)
d.print(a!=b)
"#,
    )
    .runs(b"true\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\ntrue\n");
}

#[test]
pub fn type_tests_narrow_values_fields_and_short_circuit_operands() {
    Case::new(
        r#"
d:@"debug"
inspect<null>:(value<int32><string><null>){
    |value<int32>|d.print(value+1)
    |value<string>&&value.size()>0|d.print(value<string>)
    |value<null>|d.print("missing")
}
inspect(4)
inspect("text")
inspect(null)
fallback<string>:(value<string><null>) 'result {
    |value<null>|{'result->"fallback";'result.leave()}
    ->value<string>
}
d.print(fallback(null))
d.print(fallback("kept"))
field<null>:(record<{name<string><null>}>){
    |record.name<string>|d.print(record.name.size())
    |record.name<null>||record.name.size()==0|d.print("empty")
}
field({->name:"four"})
field({})
"#,
    )
    .runs(b"5\ntext\nmissing\nfallback\nkept\n4\nempty\n");
}

#[test]
pub fn complementary_conditions_prove_exactly_one_emission() {
    Case::new(
        r#"
d:@"debug"
select<int32>:(value<string><null>){
    |value<null>|->1
    |!(value<null>)|->2
}
boolean<int32>:(flag<boolean>){
    |flag|->3
    |!flag|->4
}
d.print(select(null))
d.print(select("yes"))
d.print(boolean(true))
d.print(boolean(false))
"#,
    )
    .runs(b"1\n2\n3\n4\n");
}

#[test]
pub fn record_variants_keep_full_identity_and_nested_null_defaults() {
    Case::new(
        r#"
d:@"debug"
<Reading>:<{name<string><null>;count<int32>}>
read<null>:(value<Reading><string><null>){
    |value<Reading>|{
        d.print(value.count)
        d.print(value.name)
    }
    |value<null>|d.print("missing")
    |value<string>|d.print(value)
}
r<Reading>:{->count:7}
read(r)
read(null)
read("text")
|r<null>|d.print("incorrect")
wrapped<{reading<Reading><null>}>:{->reading:r}
|wrapped.reading<Reading>|d.print(wrapped.reading.name)
"#,
    )
    .runs(b"7\nnull\nmissing\ntext\nnull\n");
}

#[test]
pub fn predicate_operands_evaluate_once_even_for_known_results() {
    Case::new(
        r#"
d:@"debug"
make<int32><null>:(){d.print("union");->7}
scalar<int32>:(){d.print("scalar");->4}
|make()<int32>|d.print("matched")
|scalar()<null>|d.print("incorrect")
"#,
    )
    .runs(b"union\nmatched\nscalar\n");
}

#[test]
pub fn nullable_slot_defaults_reset_on_restart() {
    Case::new(
        r#"
d:@"debug"
make<{name<string><null>}>:(){
    first:=true
    ->'again {
        |first|{
            'again->name:"discarded"
            first=false
            'again.restart()
        }
    }
}
d.print(make().name)
"#,
    )
    .runs(b"null\n");
}

#[test]
pub fn expected_record_composition_preserves_fields_without_inventing_them() {
    Case::new(
        r#"
d:@"debug"
record<{name<string><null>;count<int32>}>:{
    ->{->name:"kept"}
    ->count:7
}
d.print(record.name)
d.print(record.count)
"#,
    )
    .runs(b"kept\n7\n");
}

#[test]
pub fn variant_fields_with_the_same_name_have_distinct_type_proofs() {
    Case::new(
        r#"
d:@"debug"
<Text>:<{name<string><null>}>
<Number>:<{name<int32><null>}>
read<null>:(value<Text><Number>){
    |value<Text>|{
        |value.name<string>|d.print(value.name.size())
        |value.name<null>|d.print("missing text")
    }
    |value<Number>|{
        |value.name<int32>|d.print(value.name+1)
        |value.name<null>|d.print("missing number")
    }
}
text<Text>:{->name:"test"}
number<Number>:{->name:8}
empty<Text>:{}
read(text)
read(number)
read(empty)
"#,
    )
    .runs(b"4\n9\nmissing text\n");
}

#[test]
pub fn branch_proofs_do_not_survive_mutation_or_continuing_arms() {
    for (source, code) in [
        ("f<null>:(x<string><null>){|x<null>|{};y:x<string>}", "E208"),
        (
            "x<string><null>:=\"ok\";|x<string>|{x=null;y:x<string>}",
            "E208",
        ),
        (
            "x<{name<string><null>}>:={->name:\"ok\"};|x.name<string>|{x={};y:x.name<string>}",
            "E208",
        ),
        ("f<int32>:(flag<boolean>){|flag|->1}", "E204"),
        ("f<int32><null>:(flag<boolean>){|flag|->1;->2}", "E205"),
        ("f<int32>:(x<int32>){|x>0|->1;|x>10|->2}", "E205"),
        ("x<int8><null>:128", "E216"),
        ("x<int8>:1;y<int16><null>:x", "E207"),
    ] {
        let result = Case::new(source).command("check", &["--json"]);
        assert_eq!(result.status.code(), Some(1), "{source}");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains(&format!("\"code\":\"{code}\"")),
            "{source}: {stderr}"
        );
    }
}

#[test]
pub fn independent_matchers_do_not_enumerate_every_path() {
    let params: Vec<_> = (0..24).map(|i| format!("a{i}<boolean>")).collect();
    let arms: String = (0..24).map(|i| format!("|a{i}|{{}};")).collect();
    let args = vec!["true"; 24].join(",");
    let source = format!(
        "d:@\"debug\";f<int32>:({}){{{arms}->7}};d.print(f({args}))",
        params.join(",")
    );
    Case::new(&source).runs(b"7\n");
}

#[test]
pub fn shared_references_preserve_identity_and_copy_referents() {
    Case::new(include_str!(
        "../../docs/conformance/sources/reference_identity.mwy"
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
field:&record.nested.value
same:&record.nested.value
other:&record.nested.active
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
        ("owner:=1;view:&!owner", "B001"),
        ("view:&(1+2)", "B001"),
        ("owner:1;view:&owner;view.{ref:&self}", "B001"),
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
view:'result{{'result->&owner.value;'result.leave()}}
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
        ("owner:1;view:={->field:&owner}", "B001"),
        (
            "f<null>:(flag<boolean>){owner:1;view:={|flag|->&owner}}",
            "B001",
        ),
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
        "owner:={->value:1;->other:2};view:&owner.value;owner={->value:3;->other:4};value:*view",
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

#[test]
pub fn short_circuit_blocks_and_nonreturning_operands_preserve_effects() {
    Case::new(
        r#"
d:@"debug"
d.print(false&&{d.print("unexpected");->true})
d.print(true||{d.print("unexpected");->false})
f<boolean>:(flag<boolean>){->flag&&d.panic("stop")}
g<boolean>:(flag<boolean>){->flag||d.panic("stop")}
d.print(f(false))
d.print(g(true))
'outer {
    value:{'outer.leave()}&&true
    d.print("unexpected")
}
d.print("done")
"#,
    )
    .runs(b"false\ntrue\nfalse\ntrue\ndone\n");
    for expr in [
        "true&&d.panic(\"stop\")",
        "false||d.panic(\"stop\")",
        "d.panic(\"stop\")&&false",
    ] {
        let case = Case::new(&format!(
            "d:@\"debug\";value:{expr};d.print(\"unexpected\")"
        ));
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1), "{expr}");
            assert!(result.stdout.is_empty(), "{expr}");
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(error.contains("P006"), "{expr}: {error}");
        }
    }
}

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
        ("owner:1;pair:={->view:&owner}", "B001"),
        ("owner:1;pair:{->view:&owner};view:&pair", "B001"),
        ("f<null>:(pair<{view<&int32>}>){alias:&pair}", "B001"),
        (
            "f<null>:(flag<boolean>){owner:1;pair:={|flag|->view:&owner}}",
            "B001",
        ),
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

#[test]
pub fn nullable_references_preserve_absence_and_last_use() {
    Case::new(
        r#"
d:@"debug"
inspect<null>:(flag<boolean>){
    owner:=7
    view<&int32><null>:{|flag|->&owner}
    empty<&int32><null>:null
    d.print(view==empty)
    |view<null>|owner=9
    |view<&int32>|d.print(*view<&int32>)
    owner=10
    d.print(owner)
    record<{nested<{view<&int32><null>}>;count<int32>}>:{->nested:{|flag|->view:&owner};->count:3}
    |record.nested.view<&int32>|d.print(*record.nested.view<&int32>)
    owner=11
    d.print(record.count)
}
inspect(true)
inspect(false)
"#,
    )
    .runs(b"false\n7\n10\n10\n3\ntrue\n10\n3\n");
}

#[test]
pub fn reference_union_retagging_keeps_member_and_field_origins() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{view<&int32>;extra<int32>}>
<Text>:<{view<&string>}>
inspect<null>:(flag<boolean>){
    left:=41
    right:="original"
    value<Row><Text>:{|flag|->{->view:&left;->extra:2};|!flag|->{->view:&right}}
    wide<Row><Text><null><boolean>:value
    |wide<Row>|{right="changed";d.print(*wide.view);d.print(wide.extra)}
    |wide<Text>|{left=42;d.print(*wide.view)}
    left=43
    right="done"
    d.print(left)
    d.print(right)
}
inspect(true)
inspect(false)
x:5
s:"text"
<Small>:<&int32><&string>
a<Small>:&x
b<Small><null><boolean>:a
|b<Small>|{c<Small>:b;|c<&int32>|d.print(*c<&int32>)}
other<Small><null><boolean>:&s
d.print(b==other)
"#,
    )
    .runs(b"41\n2\n43\ndone\noriginal\n43\ndone\n5\nfalse\n");
}

#[test]
pub fn optional_reference_results_reset_when_control_discards_them() {
    Case::new(
        r#"
d:@"debug"
again:=true
value<{view<&int32><null>}>:'out {
    |again|{local:5;'out->view:&local;again=false;'out.restart()}
}
|value.view<null>|d.print("missing")
owner:=12
count:=0
'loop {
    item:{|count<2|->view:&owner}
    |item.view<&int32>|d.print(*item.view<&int32>)
    owner=owner+1
    count=count+1
    |count<3|'loop.restart()
}
d.print(owner)
present:=true
number:=20
step:=0
'choose {
    view<&int32><null>:{|present|->&number}
    |view<&int32>|d.print(*view<&int32>)
    |view<null>|number=21
    present=!present
    step=step+1
    |step<2|'choose.restart()
}
d.print(number)
"#,
    )
    .runs(b"missing\n12\n13\n15\n20\n21\n");
}

#[test]
pub fn reference_unions_protect_live_copies_operands_and_slots() {
    for source in [
        "a:=1;r<&int32><null>:&a;a=2;copy:r",
        "a:=1;flag:=true;r:{|flag|->view:&a};a=2;copy:r",
        "a:=1;r<&int32><null>:&a;same:r=={a=2;->null}",
        "a:=1;flag:=true;r:{|flag|->&a;a=2}",
        "a:=1;flag:=true;r:{|flag|->view:&a;a=2}",
        "a:=1;flag:=true;|({|flag|->&a;a=2})<null>|{}",
        "a:=1;r<&int32><null>:&a;i:=0;'loop{|r<&int32>|{value:*r<&int32>};a=2;i=i+1;|i<2|'loop.restart()}",
        "a:=1;r<&int32><null>:&a;first:=true;i:=0;'loop{|!first&&r<&int32>|{value:*r<&int32>};|first|a=2;first=false;i=i+1;|i<2|'loop.restart()}",
        "f<null>:(flag<boolean>){a:=1;r<&int32><null>:{|flag|->&a};|r<&int32>|{a=2;value:*r<&int32>}}",
        "tag<int32><null>:=null;tag=1;copy:tag;a:=1;r:{|copy<int32>|->view:&a};a=2;|r.view<&int32>|{value:*r.view<&int32>}",
        "tag<{value<int32><null>}>:={};tag={->value:1};copy:tag;a:=1;r:{|copy.value<int32>|->view:&a};a=2;|r.view<&int32>|{value:*r.view<&int32>}",
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
pub fn reference_union_escapes_and_remaining_contracts_are_explicit() {
    for (source, code) in [
        ("flag:=true;r:{a:1;|flag|->&a}", "E303"),
        ("flag:=true;r:{a:1;|flag|->view:&a}", "E303"),
        ("flag:=true;r:{a:1;->nested:{|flag|->view:&a}}", "E303"),
        (
            "outer:1;flag:=true;r:{local:2;|flag|->&outer;|!flag|->&local}",
            "E303",
        ),
        ("a:1;r<&int32><null>:=&a", "B001"),
        ("a:1;r<&int32><null>:&a;alias:&r", "B001"),
        ("f<null>:(r<&int32><null>){copy:=r}", "B001"),
        ("a:1;r<&int32><null>:&a;d:@\"debug\";d.print(r)", "B001"),
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
pub fn nested_optional_tags_stay_conditioned_on_the_outer_variant() {
    Case::new(
        r#"
d:@"debug"
<A>:<{view<&int32><null>;count<int32>}>
<B>:<{view<&int32><null>;name<string>}>
inspect<null>:(flag<boolean>){
    owner:=21
    present<A>:{->view:&owner;->count:1}
    absent<B>:{->name:"empty"}
    value<A><B>:{|flag|->present;|!flag|->absent}
    |value<B>|{owner=22;d.print(value.name);|value.view<null>|d.print("missing")}
    |value<A>|{|value.view<&int32>|d.print(*value.view<&int32>)}
    owner=23
    d.print(owner)
}
inspect(true)
inspect(false)
"#,
    )
    .runs(b"21\n23\nempty\nmissing\n23\n");
    let source = r#"
<A>:<{view<&int32><null>;count<int32>}>
<B>:<{view<&int32><null>;name<string>}>
f<null>:(flag<boolean>){
    owner:=21
    present<A>:{->view:&owner;->count:1}
    absent<B>:{->name:"empty"}
    value<A><B>:{|flag|->present;|!flag|->absent}
    |value<A>|{owner=22;|value.view<&int32>|{read:*value.view<&int32>}}
}
"#;
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let result = case.command("build", &["--json", "--profile", profile]);
        assert_eq!(result.status.code(), Some(1));
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("\"code\":\"E302\""), "{error}");
    }
}

#[test]
pub fn union_record_constructors_preserve_widths_defaults_and_discarded_fields() {
    Case::new(
        r#"
d:@"debug"
<R>:<{view<&int32><null>;count<int64>}>
owner:=42
present<R><null>:{->view:&owner;->count:7}
|present<R>|{|present.view<&int32>|d.print(*present.view<&int32>);d.print(present.count)}
owner=43
absent<R><null>:{->count:9}
empty_view<&int32><null>:null
|absent<R>|{d.print(absent.view==empty_view);d.print(absent.count)}
<P>:<{-><int64>;tag<string>}>
primary<P><null>:{->7;->tag:"ok"}
|primary<P>|{number<int64>:primary;d.print(number);d.print(primary.tag)}
<B>:<{value<uint8>;view<&int32>}>
small<B><null>:{->value:255;->view:&owner}
|small<B>|{d.print(small.value);d.print(*small.view)}
<View>:<{view<&int32>}>
again:=true
empty<View><null>:'drop {
    |again|{local:1;'drop->view:&local;again=false;'drop.restart()}
}
|empty<null>|d.print("discarded")
"#,
    )
    .runs(b"42\n7\ntrue\n9\n7\nok\n255\n43\ndiscarded\n");
}

#[test]
pub fn function_borrows_preserve_pointer_identity_and_final_use() {
    Case::new(
        r#"
d:@"debug"
identity<&int32>:(value<&int32>){->value}
relay<&int32>:(value<&int32>){->identity(value)}
inferred:(value<&int32>){->relay(value)}
read<int32>:(value<&int32>){->*value}
alias:relay
owner:=41
view:alias(&owner)
d.print(view==&owner)
d.print(read(view))
owner=42
owner=read(&owner)+1
d.print(owner)
d.print(*inferred(&owner))
record:{->value:7}
field:identity(&record.value)
d.print(*field)
"#,
    )
    .runs(b"true\n41\n43\n43\n7\n");
}

#[test]
pub fn function_carriers_keep_temporary_inputs_and_returned_components() {
    Case::new(
        r#"
d:@"debug"
<R>:<{view<&int32>;nested<{other<&string>}>;count<int32>}>
copy<R>:(value<R>){->value}
head<&int32>:(value<{view<&int32>}>){->value.view}
owner:=17
text:="word"
record:copy({->view:&owner;->nested:{->other:&text};->count:3})
d.print(*record.view)
d.print(*record.nested.other)
owner=18
text="changed"
d.print(record.count)
view:head({->view:&owner})
d.print(*view)
owner=19
d.print(owner)
"#,
    )
    .runs(b"17\nword\n3\n18\n19\n");
}

#[test]
pub fn function_contracts_ignore_absent_inputs_and_scalar_only_results() {
    Case::new(
        r#"
d:@"debug"
first<&int32>:(value<&int32>,ignored<&string><null>){->value}
measure<int32>:(value<&int32>,ignored<&string>){->*value}
packet<{view<&int32>;count<int32>}>:(value<&int32>,ignored<&string>){->view:value;->count:*value}
owner:=5
text:="old"
missing<&string><null>:null
view:first(&owner,missing)
text="new"
d.print(*view)
owner=6
number:measure(&owner,&text)
owner=7
text="next"
d.print(number)
record:packet(&owner,&text)
owner=8
text="last"
d.print(record.count)
d.print(owner)
"#,
    )
    .runs(b"5\n6\n7\n8\n");
}

#[test]
pub fn recursive_function_borrows_use_declared_contracts() {
    Case::new(
        r#"
d:@"debug"
walk<&int32>:(value<&int32>,depth<int32>) 'result {
    |depth<=0|{'result->value;'result.leave()}
    ->walk(value,depth-1)
}
owner:=9
view:walk(&owner,4)
d.print(*view)
owner=10
d.print(owner)
first<(&int32,int32)->&int32>;
second<(&int32,int32)->&int32>;
first<&int32>:(value<&int32>,depth<int32>) 'done {
    |depth<=0|{'done->value;'done.leave()}
    ->second(value,depth-1)
}
second<&int32>:(value<&int32>,depth<int32>) 'done {
    |depth<=0|{'done->value;'done.leave()}
    ->first(value,depth-1)
}
d.print(*first(&owner,3))
owner=11
d.print(owner)
"#,
    )
    .runs(b"9\n10\n10\n11\n");
}

#[test]
pub fn returned_views_retain_every_active_input_loan() {
    for source in [
        "first<&int32>:(a<&int32>,b<&int32>){->a};a:=1;b:=2;r:first(&a,&b);b=3;value:*r",
        "first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:first(&a,&b);b=\"new\";value:*r",
        "identity<&int32>:(a<&int32>){->a};first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:identity(first(&a,&b));b=\"new\";value:*r",
        "head<&int32>:(p<{left<&int32>;right<&int32>}>){->p.left};a:=1;b:=2;r:head({->left:&a;->right:&b});b=3;value:*r",
        "pair<{left<&int32>;right<&int32>}>:(a<&int32>,b<&int32>){->left:a;->right:b};a:=1;b:=2;r:pair(&a,&b);b=3;value:*r.left",
        "identity<&int32>:(a<&int32>){->a};a:=1;r:identity(&a);a=2;value:*r",
        "read<int32>:(a<&int32>,b<int32>){->*a+b};a:=1;x:read(&a,{a=2;->3})",
        "identity<&int32>:(a<&int32>){->a};a:=1;r:{->identity(&a);a=2};value:*r",
        "identity<&int32>:(a<&int32>){->a};a:=1;r:identity(&a);i:=0;'loop{value:*r;a=2;i=i+1;|i<2|'loop.restart()}",
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
pub fn function_return_contracts_reject_local_and_ignored_input_escapes() {
    for source in [
        "bad<&int32>:(){owner:1;->&owner}",
        "bad<&int32>:(value<&int32>){owner:1;->&owner}",
        "identity<&int32>:(value<&int32>){->value};bad<&int32>:(){owner:1;->identity(&owner)}",
        "first<&int32>:(a<&int32>,b<&string>){->a};owner:1;r:{short:\"local\";->first(&owner,&short)}",
        "pair<{view<&int32>}>:(a<&int32>,b<&string>){->view:a};owner:1;r:{short:\"local\";->pair(&owner,&short)}",
        "first<&int32>:(a<&int32>,b<&int32>){->a};wrapper<&int32>:(a<&int32>){short:1;->first(a,&short)}",
    ] {
        let result = Case::new(source).command("check", &["--json"]);
        assert_eq!(result.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("\"code\":\"E303\""), "{source}: {error}");
    }
}

#[test]
pub fn nullable_function_results_keep_active_bounds_and_empty_returns() {
    Case::new(
        r#"
d:@"debug"
<Maybe>:<&int32><null>
copy<Maybe>:(value<Maybe>){->value}
optional<Maybe>:(flag<boolean>,value<&int32>){|flag|->value;|!flag|->null}
empty<Maybe>:(){->null}
inspect<null>:(flag<boolean>){
    owner:=11
    view:copy(optional(flag,&owner))
    |view<null>|{owner=12;d.print("absent")}
    |view<&int32>|d.print(*view<&int32>)
    owner=13
    d.print(owner)
}
inspect(true)
inspect(false)
none:empty()
|none<null>|d.print("empty")
number:7
text:"text"
<AnyRef>:<&int32><&string>
keep<AnyRef>:(value<AnyRef>){->value}
a:keep(&number)
b:keep(&text)
|a<&int32>|d.print(*a<&int32>)
|b<&string>|d.print(*b<&string>)
"#,
    )
    .runs(b"11\n13\nabsent\n13\nempty\n7\ntext\n");
}

#[test]
pub fn separate_calls_keep_independent_bounds_and_local_scalar_use() {
    Case::new(
        r#"
d:@"debug"
identity<&int32>:(value<&int32>){->value}
first<&int32>:(a<&int32>,b<&string>){->a}
read<int32>:(a<&int32>){local:"short";->*first(a,&local)}
left:=5
right:=6
pair:{->left:identity(&left);->right:identity(&right)}
right=7
d.print(*pair.left)
left=8
d.print(read(&left))
"#,
    )
    .runs(b"5\n8\n");
}

#[test]
pub fn shared_reborrows_preserve_identity_and_end_after_final_use() {
    Case::new(
        r#"
d:@"debug"
owner:=1
parent:&owner
view:&*parent
nested:&*(&*view)
d.print(view==parent)
d.print(nested==&owner)
d.print(*nested)
owner=2
d.print(owner)
direct:&*(&owner)
d.print(*direct)
owner=3
d.print(owner)
"#,
    )
    .runs(b"true\ntrue\n1\n2\n2\n3\n");
}

#[test]
pub fn field_reborrows_use_original_nested_storage() {
    Case::new(
        r#"
d:@"debug"
<R>:<{small<uint8>;value<int64>;nested<{count<int32>}>}>
owner<R>:={->small:7;->value:41;->nested:{->count:9}}
parent:&owner
small:&parent.small
wide:&(*parent).value
leaf:&(parent.nested).count
d.print(small==&owner.small)
d.print(wide==&owner.value)
d.print(leaf==&owner.nested.count)
d.print(*small)
d.print(*wide)
d.print(*leaf)
owner={->small:8;->value:42;->nested:{->count:10}}
d.print(owner.value)
"#,
    )
    .runs(b"true\ntrue\ntrue\n7\n41\n9\n42\n");
}

#[test]
pub fn reborrowed_function_fields_preserve_call_bounds_and_evaluate_once() {
    Case::new(
        r#"
d:@"debug"
<R>:<{value<int32>;nested<{other<int32>}>}>
get<&R>:(value<&R>){d.print("get");->value}
field<&int32>:(value<&R>){->&value.nested.other}
relay<&int32>:(value<&R>){->&*field(value)}
owner<R>:={->value:11;->nested:{->other:22}}
view:&get(&owner).value
d.print(*view)
leaf:&(*get(&owner)).nested.other
d.print(*leaf)
d.print(*relay(&owner))
d.print((&get(&owner).value)==&owner.value)
d.print((&*get(&owner))==&owner)
owner={->value:33;->nested:{->other:44}}
d.print(owner.value)
"#,
    )
    .runs(b"get\n11\nget\n22\n22\nget\ntrue\nget\ntrue\n33\n");
}

#[test]
pub fn guarded_reborrow_parents_keep_branch_and_iteration_origins() {
    Case::new(
        r#"
d:@"debug"
inspect<null>:(flag<boolean>){
    left:=11
    right:=22
    parent:{|flag|->&left;|!flag|->&right}
    view:&*parent
    |flag|right=33
    |!flag|left=44
    d.print(*view)
}
inspect(true)
inspect(false)
owner:=7
optional<&int32><null>:&owner
|optional<&int32>|{view:&*optional<&int32>;d.print(*view)}
owner=8
count:=0
'loop {
    parent:&owner
    view:&*parent
    d.print(*view)
    owner=owner+1
    count=count+1
    |count<2|'loop.restart()
}
d.print(owner)
"#,
    )
    .runs(b"11\n22\n7\n8\n9\n10\n");
}

#[test]
pub fn shared_reborrow_uses_and_inherited_bounds_reject_live_writes() {
    for source in [
        "owner:=1;parent:&owner;view:&*parent;owner=2;value:*view",
        "owner:={->value:1};parent:&owner;view:&parent.value;owner={->value:2};value:*view",
        "<R>:<{value<int32>}>;field<&int32>:(value<&R>){->&value.value};owner<R>:={->value:1};view:field(&owner);owner={->value:2};value:*view",
        "first<&int32>:(a<&int32>,b<&string>){->a};owner:=1;other:=\"old\";view:&*first(&owner,&other);other=\"new\";value:*view",
        "owner:=1;parent:&owner;same:(&*parent)=={owner=2;->&owner}",
        "owner:=1;parent:&owner;result:{->&*parent;owner=2};value:*result",
        "owner:=1;parent:&owner;view:&*parent;i:=0;'loop{value:*view;owner=2;i=i+1;|i<2|'loop.restart()}",
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
pub fn shared_reborrow_escapes_and_remaining_storage_boundaries_are_explicit() {
    for (source, code) in [
        ("view:{owner:1;parent:&owner;->&*parent}", "E303"),
        (
            "view:{owner:{->value:1};parent:&owner;->&parent.value}",
            "E303",
        ),
        (
            "<R>:<{value<int32>}>;field<&int32>:(value<&R>){->&value.value};bad<&int32>:(){owner<R>:{->value:1};->field(&owner)}",
            "E303",
        ),
        (
            "first<&int32>:(a<&int32>,b<&string>){->a};owner:1;view:{short:\"local\";->&*first(&owner,&short)}",
            "E303",
        ),
        ("owner:1;parent:&owner;pointer:&parent", "B001"),
        ("owner:=1;parent:&owner;view:&!*parent", "B001"),
        (
            "make<{value<int32>}>:(){->value:1};view:&make().value",
            "B001",
        ),
        ("view:&({->value:1}).value", "B001"),
        (
            "owner:1;carrier:{->view:&owner};pointer:&carrier.view",
            "B001",
        ),
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
pub fn reborrow_hints_preserve_argument_leave_and_operand_effects() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
get<&R>:(p<&R>){d.print("get");->p}
inspect<int32>:(flag<boolean>) 'out {
    owner<R>:{->n:1}
    same:(&get({|flag|{'out->7;'out.leave()};->&owner}).n)==&owner.n
    ->8
}
d.print(inspect(true))
d.print(inspect(false))
"#,
    )
    .runs(b"7\nget\n8\n");
}

#[test]
pub fn parameter_borrows_address_local_copies_and_preserve_argument_order() {
    Case::new(
        r#"
d:@"debug"
read<int32>:(value<int32>){view:{->&value};->*view}
same<boolean>:(value<int32>,original<&int32>){->&value==original}
first<int32>:(value<int32>,ignored<int32>){view:&value;->*view}
<R>:<{n<int32>;nested<{other<int64>}>}>
field<int64>:(value<R>){view:&value.nested.other;->*view}
owner:=7
d.print(same(owner,&owner))
d.print(read(owner))
d.print(first(owner,{owner=8;->0}))
d.print(owner)
d.print(field({->n:1;->nested:{->other:9}}))
"#,
    )
    .runs(b"false\n7\n7\n8\n9\n");
}

#[test]
pub fn value_dispatch_borrows_its_copy_and_shared_dispatch_keeps_the_owner() {
    Case::new(
        r#"
d:@"debug"
owner:=4
copy:owner.{owner=5;view:&self;->*view}
d.print(copy)
d.print(owner)
<R>:<{n<int32>;nested<{other<int32>}>}>
record<R>:={->n:6;->nested:{->other:7}}
count:record.{view:&self.n;->*view}
d.print(count)
view:(&record).{->&self.nested.other}
d.print(view==&record.nested.other)
d.print(*view)
record={->n:8;->nested:{->other:9}}
d.print(record.n)
"#,
    )
    .runs(b"4\n5\n6\ntrue\n7\n8\n");
}

#[test]
pub fn reference_carrier_dispatch_preserves_components_and_nullable_activity() {
    Case::new(
        r#"
d:@"debug"
a:=11
b:=22
pair:{->left:&a;->right:&b}
view:pair.{->self.left}
b=23
d.print(*view)
a=12
inspect<null>:(flag<boolean>){
    owner:=7
    value<&int32><null>:{|flag|->&owner}
    result:value.{|self<&int32>|->self<&int32>}
    |result<null>|{owner=8;d.print("none")}
    |result<&int32>|d.print(*result<&int32>)
    owner=9
    d.print(owner)
}
inspect(true)
inspect(false)
"#,
    )
    .runs(b"11\n7\n9\nnone\n9\n");
}

#[test]
pub fn borrowed_dispatch_operands_run_once_and_preserve_exits() {
    Case::new(
        r#"
d:@"debug"
get<&int32>:(value<&int32>){d.print("get");->value}
owner:=7
view:get(&owner).{->&*self}
d.print(*view)
owner=8
inspect<int32>:(flag<boolean>) 'out {
    value:3
    ->get({|flag|{'out->9;'out.leave()};->&value}).{view:&*self;->*view}
}
d.print(inspect(true))
d.print(inspect(false))
"#,
    )
    .runs(b"get\n7\n9\nget\n3\n");
}

#[test]
pub fn parameter_and_receiver_copy_addresses_cannot_escape() {
    for source in [
        "bad<&int32>:(value<int32>){->&value}",
        "bad:(value<int32>){view:{->&value};->view}",
        "<R>:<{n<int32>}>;bad<&int32>:(value<R>){->&value.n}",
        "owner:1;view:owner.{->&self}",
        "owner:{->n:1};view:owner.{->&self.n}",
        "bad<&int32>:(value<int32>){->(&value).{->&*self}}",
        "first<&int32>:(a<&int32>,b<&string>){->a};owner:1;view:{short:\"x\";->first(&owner,&short).{->self}}",
    ] {
        let result = Case::new(source).command("check", &["--json"]);
        assert_eq!(result.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("\"code\":\"E303\""), "{source}: {error}");
    }
}

#[test]
pub fn shared_dispatch_keeps_active_and_inherited_loans() {
    for source in [
        "owner:=1;value:(&owner).{owner=2;->*self}",
        "owner:=1;view:(&owner).{->self};owner=2;value:*view",
        "first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"x\";view:first(&a,&b).{->&*self};b=\"y\";value:*view",
        "owner:=1;view:(&owner).{->&*self;owner=2};value:*view",
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
pub fn carrier_field_reborrows_copy_the_reference_prefix_once() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
<H>:<{view<&R>;count<int32>}>
make<H>:(p<&R>){d.print("make");->view:p;->count:3}
owner<R>:={->n:7}
holder<H>:{->view:&owner;->count:4}
view:holder.{->&self.view.n}
d.print(view==&owner.n)
d.print(*view)
other:&make(&owner).view.n
d.print(*other)
maybe<{view<&R><null>}>:{->view:&owner}
|maybe.view<&R>|{field:&maybe.view.n;d.print(*field)}
owner={->n:8}
d.print(holder.count)
d.print(owner.n)
"#,
    )
    .runs(b"true\n7\nmake\n7\n7\n4\n8\n");
    for (source, code) in [
        (
            "owner:{->n:1};holder:{->view:&owner};view:&holder.view",
            "B001",
        ),
        (
            "owner:{->n:1};holder:{->view:&owner;->count:2};view:&holder.count",
            "B001",
        ),
        (
            "view:{owner:{->n:1};holder:{->view:&owner};->&holder.view.n}",
            "E303",
        ),
        (
            "owner:={->n:1};holder:{->view:&owner};view:&holder.view.n;owner={->n:2};read:*view",
            "E302",
        ),
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
pub fn bounded_lists_preserve_element_types_capacity_and_value_copies() {
    Case::new(
        r#"
d:@"debug"
byte<uint8>:7
values:[byte,8]
accept<uint8>:(value<uint8>){->value}
d.print(accept(values[2]))
capacity:2+2
original<int32[capacity]>:[10]
changed:original.add(20)
d.print(original.size())
d.print(changed.size())
d.print(changed[2])
copy<int32[4]>:=changed
copy=[]
d.print(copy.size())
d.print(changed[1])
empty<null[0]>:[]
d.print(empty.size())
growing<int32[3]>:=[]
next:=1
'fill {
    growing=growing.add(next)
    next=next+1
    |next<=3|'fill.restart()
}
d.print(growing.size())
d.print(growing[3])
"#,
    )
    .runs(b"8\n1\n2\n20\n0\n10\n0\n3\n3\n");
}

#[test]
pub fn bounded_lists_keep_records_unions_and_nested_values() {
    Case::new(
        r#"
d:@"debug"
<Row>:<{name<string>;value<int32>}>
rows<Row[3]>:[{->name:"first";->value:11},{->name:"second";->value:22}]
d.print(rows[2].name)
d.print(rows[1].value)
<Value>:<int32><string>
member<Value>:3
mixed:[member,"text"]
first:mixed[1]
second:mixed[2]
|first<int32>|d.print(first<int32>)
|second<string>|d.print(second<string>)
nested<int32[3][2]>:[[1],[2,3]]
d.print(nested[2][2])
d.print(nested[1].size())
<Numbers>:<int32[2]>
<Words>:<string[2]>
numbers<Numbers>:[5]
choice<Numbers><Words>:numbers
|choice<Numbers>|d.print(choice<Numbers>[1])
d.print(choice==choice)
"#,
    )
    .runs(b"second\n11\n3\ntext\n3\n1\n5\ntrue\n");
}

#[test]
pub fn list_equality_uses_initialized_elements_and_element_semantics() {
    Case::new(
        r#"
d:@"debug"
a<int32[4]>:=[1,2,3]
a=[]
b<int32[4]>:[]
d.print(a==b)
d.print(a!=b.add(1))
x<float64[3]>:[-0.0]
y<float64[3]>:[0.0]
d.print(x==y)
nan:0.0/0.0
n:[nan]
d.print(n==n)
s<string[3]>:["same"]
t<string[3]>:["same"]
d.print(s==t)
left<int32[3][2]>:[[1],[2,3]]
right<int32[3][2]>:[[1],[2,3]]
d.print(left==right)
<Row>:<{value<int32>;name<string>}>
r<Row[2]>:[{->value:1;->name:"one"}]
q<Row[2]>:[{->value:1;->name:"one"}]
d.print(r==q)
<Value>:<int32><string>
u<Value[2]>:[1,"two"]
v<Value[2]>:[1,"two"]
d.print(u==v)
"#,
    )
    .runs(b"true\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
pub fn list_receivers_are_copied_before_argument_effects() {
    Case::new(
        r#"
d:@"debug"
values<int32[3]>:=[1,2]
item:values[{values=[];->1}]
d.print(item)
d.print(values.size())
values=[1]
result:values.add({values=[2,3];->4})
d.print(result[1])
d.print(result[2])
d.print(values[1])
d.print(result.size())
make<int32[3]>:(){d.print("receiver");->[9]}
index<usize>:(){d.print("index");->1}
d.print(make()[index()])
'out {unused:values[{'out.leave();->1}];d.print("unreachable")}
'out {unused:values.add({'out.leave();->1});d.print("unreachable")}
d.print("done")
"#,
    )
    .runs(b"1\n0\n1\n4\n2\n2\nreceiver\nindex\n9\ndone\n");
}

#[test]
pub fn lists_reject_static_bounds_and_incompatible_elements() {
    for (source, code) in [
        ("values:[1,2];item:values[0]", "E101"),
        ("values<int32[4]>:[1];item:values[2]", "E101"),
        ("values<int32[0]>:[];item:values[1]", "E101"),
        ("values<int32[1]>:[1,2]", "E103"),
        ("values:[1];other:values.add(2)", "E103"),
        ("values<int32[0]>:[];other:values.add(1)", "E103"),
        ("values<int32[-1]>:[]", "E104"),
        (
            "f<null>:(capacity<usize>){values<int32[capacity]>:[]}",
            "E104",
        ),
        ("capacity<uint8>:255;values<int32[capacity+1]>:[]", "E107"),
        ("a<uint8>:1;b<uint16>:2;values<int32[a+b]>:[]", "E213"),
        ("values:[1,\"two\"]", "E207"),
        ("a<uint8>:1;b<uint16>:2;values:[a,b]", "E207"),
        ("a:{->1;->field:2};values:[a,3]", "E207"),
        ("values<int32[2]>:[1,\"two\"]", "E207"),
        ("a:[1];b<int32[2]>:a", "E207"),
        ("values:[1];size:values.size(1)", "E212"),
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
pub fn list_dynamic_failures_follow_argument_effects_in_both_profiles() {
    for (source, code) in [
        (
            "d:@\"debug\";read<int32>:(values<int32[3]>,position<usize>){->values[{d.print(\"argument\");->position}]};read([1],2)",
            "P001",
        ),
        (
            "d:@\"debug\";read<int32>:(values<int32[3]>,position<int32>){->values[{d.print(\"argument\");->position}]};read([1],-1)",
            "P001",
        ),
        (
            "d:@\"debug\";read<int32>:(values<int32[3]>,position<uint64>){->values[{d.print(\"argument\");->position}]};read([1],18446744073709551615)",
            "P001",
        ),
        (
            "d:@\"debug\";append<int32[1]>:(values<int32[1]>){->values.add({d.print(\"argument\");->2})};append([1])",
            "P003",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let result = case.command("run", &["--profile", profile]);
            assert_eq!(result.status.code(), Some(1), "{source}");
            assert_eq!(result.stdout, b"argument\n", "{source}");
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(error.contains(code), "{source}: {error}");
        }
    }
}

#[test]
pub fn whole_list_borrows_keep_owner_identity_and_end_at_final_use() {
    Case::new(
        r#"
d:@"debug"
values<int32[3]>:=[10]
view:&values
same<&int32[3]>:(value<&int32[3]>){->value}
alias:same(view)
d.print(alias==view)
d.print((*alias)[1])
d.print(view[1])
d.print(view.size())
values=[20,30]
d.print(values.size())
<Row>:<{values<int32[3]>}>
row<Row>:={->values:[4]}
field:&row.values
d.print((*field)[1])
row={->values:[5,6]}
d.print(row.values[2])
owner:=7
borrow:&owner
items:[*borrow,{owner=8;->9}]
d.print(items[1])
d.print(owner)
last:&values
d.print((*last)[{values=[];->1}])
d.print(values.size())
"#,
    )
    .runs(b"true\n10\n10\n1\n2\n4\n6\n7\n8\n20\n0\n");
}

#[test]
pub fn list_borrow_conflicts_and_unavailable_operations_stay_explicit() {
    for (source, code) in [
        (
            "values<int32[2]>:=[1];view:&values;values=[];copy:*view",
            "E302",
        ),
        ("owner:=1;view:&owner;values:[{owner=2;->3},*view]", "E302"),
        ("view:{values:[1];->&values}", "E303"),
        (
            "keep<&int32[2]>:(first<&int32[2]>,other<&int32>){->first};values<int32[2]>:=[1];other:=2;view:keep(&values,&other);other=3;copy:*view",
            "E302",
        ),
        ("values<int32[]>:[]", "B001"),
        ("values<uint8[65537]>:[]", "B001"),
        ("values:[\"name\":1]", "B001"),
        ("owner:1;values:[&owner]", "B001"),
        ("values:[1];slice:values.slice()", "B001"),
        ("values:[1];removed:values.remove(1)", "B001"),
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
pub fn list_length_proofs_follow_completing_paths() {
    Case::new(
        r#"
d:@"debug"
flag:=true
values<int32[2]>:'choose {
    |flag|{'choose->[10,20];'choose.leave()}
    ->[30]
}
d.print(values[2])
short<int32[2]>:'choose {
    |flag|{'choose->[40];'choose.leave()}
    ->[50,60]
}
longer:short.add(70)
d.print(longer[2])
"#,
    )
    .runs(b"20\n70\n");
    let source = "values<int32[3]>:'out{->[];'out.leave();->[1,2]};item:values[1]";
    let result = Case::new(source).command("check", &["--json"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("\"code\":\"E101\""));
}

#[test]
pub fn list_union_literals_select_unique_types_capacities_and_widths() {
    Case::new(
        r#"
d:@"debug"
values<int32[1]><string[1]>:[1]
words<int32[1]><string[1]>:["one"]
|values<int32[1]>|d.print(values[1])
|words<string[1]>|d.print(words[1])
two<int32[1]><int32[2]>:[11,22]
|two<int32[2]>|{d.print(two.size());d.print(two[2])}
wide<uint8[1]><uint16[1]>:[300]
|wide<uint16[1]>|d.print(wide[1])
byte<uint8>:7
typed<uint8[2]><uint16[2]>:[8,byte]
|typed<uint8[2]>|{d.print(typed[1]);d.print(typed[2])}
large<uint64>:9
late<int32[2]><uint64[2]>:[4294967295,large]
|late<uint64[2]>|{d.print(late[1]);d.print(late[2])}
optional<int32[1]><string[1]><null>:[4]
|optional<int32[1]>|d.print(optional[1])
<Value>:<int32><string>
member<Value>:5
existing<Value[1]><int32[1]>:[member]
|existing<Value[1]>|{element:existing[1];|element<int32>|d.print(element<int32>)}
record:{->23;->tag:"record"}
projected<int32[1]><string[1]>:[record]
|projected<int32[1]>|d.print(projected[1])
empty<uint8[0]><uint8[0]>:[]
d.print(empty.size())
"#,
    )
    .runs(b"1\none\n2\n22\n300\n8\n7\n4294967295\n9\n4\n5\n23\n0\n");
}

#[test]
pub fn list_union_literals_preserve_nested_and_record_contexts() {
    Case::new(
        r#"
d:@"debug"
nested<int32[2][2]><string[2][2]>:[[1],[2,3]]
|nested<int32[2][2]>|{d.print(nested[2][2]);d.print(nested[1].size())}
inner<int32[2]>:[7]
typed<int32[1][1]><int32[2][1]>:[inner]
|typed<int32[2][1]>|d.print(typed[1][1])
numeric<uint8[2][1]><uint16[2][1]>:[[300]]
|numeric<uint16[2][1]>|d.print(numeric[1][1])
<Row>:<{value<int32>;name<string>}>
<Other>:<{message<string>}>
row<Row>:{->value:17;->name:"row"}
rows<Row[2]><Other[2]>:[row]
|rows<Row[2]>|{d.print(rows[1].name);d.print(rows[1].value)}
records<Row[1]><Row[2]>:[{->value:1;->name:"first"},{->value:2;->name:"second"}]
|records<Row[2]>|d.print(records[2].name)
<A>:<{value<int64>;tag<string><null>}>
<B>:<{value<string>}>
fresh<A[1]><B[1]>:[{->value:7}]
accept64<int64>:(value<int64>){->value}
|fresh<A[1]>|{d.print(accept64(fresh[1].value));|fresh[1].tag<null>|d.print("missing tag")}
"#,
    )
    .runs(b"3\n1\n7\n300\nrow\n17\nsecond\n7\nmissing tag\n");
}

#[test]
pub fn list_union_literals_reject_ambiguous_or_impossible_contexts() {
    for (source, code) in [
        ("values<uint8[1]><uint16[1]>:[1]", "E207"),
        ("values<int32[2]><int32[3]>:[1]", "E207"),
        ("values<int32[0]><string[2]>:[]", "E207"),
        ("values<int32[2]><string[2]>:[true]", "E207"),
        ("byte<uint8>:7;values<uint16[1]><string[1]>:[byte]", "E207"),
        ("values<int32[1]><string[2]>:[1,2,3]", "E103"),
        ("values<int32[1]><string[2]>:[1,2]", "E207"),
        ("values<uint8[1]><int8[1]>:[256]", "E207"),
        ("values<uint8[1]>:[256]", "E216"),
        ("values<int32[1][1]><int32[2][1]>:[[1]]", "E207"),
        (
            "<Value>:<int32><string>;values<Value[1]><int32[1]>:[1]",
            "E207",
        ),
        ("values<int32[2]><string[2]>:[1,missing]", "E201"),
        ("values<int32[2]><string[2]>:[1,1/0]", "E107"),
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
pub fn list_union_candidate_checks_preserve_effect_order_and_single_evaluation() {
    Case::new(
        r#"
d:@"debug"
mark<int32>:(value<int32>){d.print(value);->value}
selected<int32[3]><string[3]>:[mark(1),mark(2)]
|selected<int32[3]>|d.print(selected[1]+selected[2])
wide<uint64>:(){d.print("wide");->7}
large<uint64[2]><string[2]>:[4294967295,wide()]
|large<uint64[2]>|{d.print(large[1]);d.print(large[2])}
count:=0
ordered<int32[1]><int32[2]>:[{d.print("first");count=count+1;->count},{d.print("second");count=count+1;->count}]
|ordered<int32[2]>|{d.print(ordered[1]);d.print(ordered[2])}
d.print(count)
"#,
    )
    .runs(b"1\n2\n3\nwide\n4294967295\n7\nfirst\nsecond\n1\n2\n2\n");
}

#[test]
pub fn list_union_candidate_checks_preserve_early_exits_and_panics() {
    Case::new(
        r#"
d:@"debug"
'out {
    unused<int32[1]><int32[2]>:[{d.print("before");'out.leave();->1},2]
    d.print("unreachable")
}
d.print("done")
"#,
    )
    .runs(b"before\ndone\n");
    let source =
        "d:@\"debug\";values<int32[1]><int32[2]>:[1,{d.print(\"element\");d.panic(\"stop\")}]";
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let result = case.command("run", &["--profile", profile]);
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(result.stdout, b"element\n");
        let start = source.find("d.panic(\"stop\")").unwrap();
        let end = start + "d.panic(\"stop\")".len();
        assert_eq!(
            result.stderr,
            format!("panic[P006]: stop at bytes {start}..{end}\n").as_bytes()
        );
    }
}

#[test]
pub fn unary_results_enter_expected_unions_after_operand_evaluation() {
    Case::new(
        r#"
d:@"debug"
<Logic>:<boolean><null>
logic<Logic[1]>:[!true]
choice<int32[1]><Logic[1]>:[!true]
first:logic[1]
|first<boolean>|d.print(first<boolean>)
|choice<Logic[1]>|{value:choice[1];|value<boolean>|d.print(value<boolean>)}
<Number>:<int8><null>
number<int8>:5
single<Number[2]>:[-number,~number]
selected<Number[2]><string[2]>:[-number,~number]
negative:single[1]
|negative<int8>|d.print(negative<int8>)
|selected<Number[2]>|{value:selected[2];|value<int8>|d.print(value<int8>)}
<Float>:<float32><null>
value<Float>:-({->1.5})
|value<float32>|d.print(value<float32>)
"#,
    )
    .runs(b"false\nfalse\n-5\n-6\n-1.5\n");
}

#[test]
pub fn compound_list_candidates_preserve_intermediate_widths_and_typed_leaves() {
    Case::new(
        r#"
d:@"debug"
grouped<int8[1]><int16[1]>:[-(128)]
|grouped<int16[1]>|d.print(grouped[1])
intermediate<int8[1]><int16[1]>:[(127+1)-1]
|intermediate<int16[1]>|d.print(intermediate[1])
negative<int8[1]><int16[1]>:[-(-128)]
|negative<int16[1]>|d.print(negative[1])
bits<int8[1]><uint8[1]>:[~128]
|bits<uint8[1]>|d.print(bits[1])
byte<uint8>:1
typed<uint8[1]><int16[1]>:[byte+1]
|typed<uint8[1]>|d.print(typed[1])
wide<uint64>:18446744073709551615
exact<uint64[1]><int64[1]>:[wide-1]
|exact<uint64[1]>|d.print(exact[1])
"#,
    )
    .runs(b"-128\n127\n128\n127\n2\n18446744073709551614\n");
}

#[test]
pub fn compound_list_candidates_keep_short_circuit_and_float_rules() {
    Case::new(
        r#"
d:@"debug"
flags<boolean[2]><int32[2]>:[false&&(1/0==0),true||(1/0==0)]
|flags<boolean[2]>|{d.print(flags[1]);d.print(flags[2])}
core:@"core"
yes:core.true
logic<boolean[1]><string[1]>:[yes&&("a"<"b")]
|logic<boolean[1]>|d.print(logic[1])
wide<float32[1]><float64[1]>:[1e39-1e39]
|wide<float64[1]>|d.print(wide[1]==0.0)
seed<float32>:1.0
rounded<float32[2]><float64[2]>:[1.0000000596046447753906250000000001+0.0,seed]
expected<float32>:1.00000011920928955078125
|rounded<float32[2]>|d.print(rounded[1]==expected)
"#,
    )
    .runs(b"false\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
pub fn compound_list_candidates_resolve_nested_literals_and_record_fields() {
    Case::new(
        r#"
d:@"debug"
values<uint8[1][1]><uint16[1][1]>:[[250+10]]
|values<uint16[1][1]>|d.print(values[1][1])
<Small>:<{value<int8>}>
<Wide>:<{value<int16>}>
rows<Small[1]><Wide[1]>:[{->value:(127+1)-1}]
|rows<Wide[1]>|d.print(rows[1].value)
"#,
    )
    .runs(b"260\n127\n");
}

#[test]
pub fn compound_list_resolution_does_not_replay_effectful_elements() {
    Case::new(
        r#"
d:@"debug"
count:=0
values<int8[2]><uint8[2]>:[127+1,{d.print("after");count=count+1;->1}]
|values<uint8[2]>|{d.print(values[1]);d.print(values[2])}
d.print(count)
mark<uint8>:(){d.print("typed");->7}
later<uint8[2]><uint16[2]>:[1+1,mark()]
|later<uint8[2]>|{d.print(later[1]);d.print(later[2])}
"#,
    )
    .runs(b"after\n128\n1\n1\ntyped\n2\n7\n");
}

#[test]
pub fn compound_list_candidates_preserve_ambiguity_and_source_errors() {
    for (source, code) in [
        ("values<int8[1]><int16[1]>:[1+1]", "E207"),
        ("values<int8[1]><uint8[1]>:[~1]", "E207"),
        ("values<int8[1]><uint8[1]>:[-(128)]", "E207"),
        ("values<int8[1]><int16[1]>:[1/0]", "E207"),
        ("values<float32[1]><float64[1]>:[3e38+3e38]", "E207"),
        ("byte<int8>:127;values<int8[1]><int16[1]>:[byte+1]", "E107"),
        ("values<int8[1]>:[(127+1)-1]", "E107"),
        ("values<int8[1]>:[-(128)]", "E216"),
        ("values<int8[2]><string[2]>:[1,missing+1]", "E201"),
        (
            "value<uint8>:1;values<uint8[1]><uint16[1]>:[{d:@\"debug\";d.print(1);->1}]",
            "B001",
        ),
        (
            "outer<int8>:1;f<null>:(){values<int8[1]><int16[1]>:[outer+1]}",
            "B001",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(&format!("\"code\":\"{code}\"")),
                "{source}: {error}"
            );
            assert!(!case.path.join("build").exists());
        }
    }
}

#[test]
pub fn compound_list_probes_respect_saved_reach_and_runtime_inputs() {
    let prefix = r#"d:@"debug";stop<never>:(){d.print("stop");d.panic("end")};"#;
    let source = format!("{prefix}values<int8[2]><int16[2]>:[stop(),(127+1)-1]");
    let output = Case::new(&source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E207\""));
    let source = format!("{prefix}values<int8[2]><int16[2]>:[(127+1)-1,stop()]");
    let start = source.find("d.panic(\"end\")").unwrap();
    let end = start + "d.panic(\"end\")".len();
    let case = Case::new(&source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"stop\n");
        assert_eq!(
            output.stderr,
            format!("panic[P006]: end at bytes {start}..{end}\n").as_bytes()
        );
    }
    let source = "number<int8>:=127;values<int8[1]><int16[1]>:[number+1]";
    let start = source.find("number+1").unwrap();
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, format!("panic[P002]: int8 + overflow (left 127, right 1; range -128..127) at bytes {start}..{}\n", start + "number+1".len()).as_bytes());
    }
}

#[test]
pub fn element_borrows_keep_original_storage_and_end_at_final_use() {
    Case::new(
        r#"
d:@"debug"
values<int32[3]>:=[7,7]
copy:values
parent:&values
index:=1
first:&values[1]
same:&parent[index]
second:&values[2]
other:&copy[1]
position:=2
index_view:&position
selected:&values[*index_view]
position=1
d.print(selected==second)
d.print(first==same)
d.print(first==second)
d.print(first==other)
d.print(&*same==same)
d.print(*second)
values=[30]
d.print(values[1])
text:{words:["inside"];view:&words[1];->*view}
d.print(text)
"#,
    )
    .runs(b"true\ntrue\nfalse\nfalse\ntrue\n7\n30\ninside\n");
}

#[test]
pub fn element_borrows_compose_nested_lists_records_and_reborrows() {
    Case::new(
        r#"
d:@"debug"
<Point>:<{x<int32>;y<int32>}>
<Row>:<{point<Point>;name<string>}>
rows<Row[3]>:=[{->point:{->x:11;->y:12};->name:"first"},{->point:{->x:21;->y:22};->name:"second"}]
row:&rows[2]
x:&rows[2].point.x
same:&row.point.x
d.print(x==same)
d.print(*same)
name:&row.name
d.print(*name)
rows=[]
<Holder>:<{values<int32[3][2]>}>
holder<Holder>:={->values:[[3],[4,5]]}
view:&holder.values[2][2]
d.print(*view)
holder={->values:[[6],[7]]}
d.print(holder.values[1][1])
"#,
    )
    .runs(b"true\n21\nsecond\n5\n6\n");
}

#[test]
pub fn element_borrow_functions_and_dispatch_preserve_input_lifetimes() {
    Case::new(
        r#"
d:@"debug"
head<&int32>:(items<&int32[3]>,position<usize>){->&items[position]}
forward<&int32>:(items<&int32[3]>,position<usize>){->head(items,position)}
copy<null>:(items<int32[3]>){view:&items[1];d.print(*view)}
values<int32[3]>:=[10,20]
view:forward(&values,2)
d.print(view==&values[2])
d.print(*view)
copy(values)
values.{view:&self[1];d.print(*view)}
shared:(&values).{->&self[1]}
d.print(*shared)
values=[30]
d.print(values[1])
optional<null>:(present<boolean>){
    owner<int32[3]>:=[10]
    parent<&int32[3]><null>:{|present|->&owner}
    |parent<&int32[3]>|{element:&parent[1];d.print(*element)}
    |parent<null>|owner=[99]
    owner=[30]
    d.print(owner[1])
}
optional(true)
optional(false)
"#,
    )
    .runs(b"true\n20\n10\n10\n10\n30\n10\n30\n30\n");
}

#[test]
pub fn element_borrow_parents_indices_and_early_exits_evaluate_once() {
    Case::new(
        r#"
d:@"debug"
parent<&int32[3]>:(items<&int32[3]>){d.print("parent");->items}
position<usize>:(){d.print("index");->1}
values<int32[3]>:=[9]
d.print(&parent(&values)[position()]==&values[1])
'out {unused:&parent(&values)[{d.print("leave");'out.leave();->1}];d.print("unreachable")}
empty<int32[0]>:[]
'out {unused:&empty[{'out.leave()}];d.print("unreachable")}
'out {unused:&values[{values=[];'out.leave();->1}]}
d.print(values.size())
|false|{unused:&values[0]}
skipped:true||{unused:&values[0];->true}
d.print(skipped)
"#,
    )
    .runs(b"parent\nindex\ntrue\nparent\nleave\n0\ntrue\n");
}

#[test]
pub fn element_borrows_enforce_bounds_lifetimes_and_remaining_boundaries() {
    for (source, code) in [
        ("values:[1];view:&values[0]", "E101"),
        ("values:[1];view:&values[-1]", "E101"),
        ("values<int32[3]>:[1];view:&values[2]", "E101"),
        ("values<int32[0]>:[];view:&values[1]", "E101"),
        ("view:{values:[1];->&values[1]}", "E303"),
        ("bad<&int32>:(values<int32[2]>){->&values[1]}", "E303"),
        ("values:[1];view:values.{->&self[1]}", "E303"),
        ("view:&[1,2][1]", "B001"),
        ("make<int32[2]>:(){->[1]};view:&make()[1]", "B001"),
        ("values:=[1];view:&!values[1]", "B001"),
        ("owner:1;values:[&owner];view:&values[1]", "B001"),
        ("values:[{->1;->name:2}];view:&values[1]<int32>", "B001"),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("build", &["--json", "--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(&format!("\"code\":\"{code}\"")),
                "{source}: {error}"
            );
            assert!(!case.path.join("build").exists());
        }
    }
}

#[test]
pub fn element_borrows_protect_parent_storage_and_ignored_input_bounds() {
    for source in [
        "values<int32[2]>:=[1];view:&values[1];values=[];copy:*view",
        "values<int32[2]>:=[1];unused:&values[{values=[];->1}]",
        "values<int32[2]>:=[1];same:&values[1]=={values=[];->&values[1]}",
        "head<&int32>:(values<&int32[2]>,other<&string>){->&values[1]};values<int32[2]>:=[1];other:=\"before\";view:head(&values,&other);other=\"after\";copy:*view",
        "pick<&int32[2]>:(values<&int32[2]>,other<&string>){->values};values<int32[2]>:=[1];other:=\"before\";unused:&pick(&values,&other)[{other=\"after\";->1}]",
        "head<&int32>:(values<&int32[2]>,index<&int32>){->&values[*index]};values<int32[2]>:=[1];index:=1;view:head(&values,&index);index=2;copy:*view",
        "values<int32[2]>:=[1];holder:{->view:&values[1]};values=[];copy:holder.view",
        "values<int32[2]>:=[1];flag:=true;|flag|{view:&values[1];values=[];copy:*view}",
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1), "{source}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E302\""), "{source}: {error}");
    }
    let source = "head<&int32>:(values<&int32[2]>,other<&string>){->&values[1]};values<int32[2]>:[1];view:{other:\"local\";->head(&values,&other)}";
    let output = Case::new(source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E303\""));
    let source = "head<&int32>:(values<&int32[2]>){->&values[1]};values:[1,2];bad:{head(&values);local:3;->&local}";
    let output = Case::new(source).command("check", &["--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"code\":\"E303\""));
}

#[test]
pub fn element_borrow_dynamic_bounds_report_original_index_and_length() {
    for (capacity, initial, ty, index, shown, length) in [
        (3, "[10]", "int8", "-1", "-1", 1),
        (3, "[10]", "usize", "2", "2", 1),
        (
            3,
            "[10]",
            "uint64",
            "18446744073709551615",
            "18446744073709551615",
            1,
        ),
        (0, "[]", "int64", "1", "1", 0),
    ] {
        let access = "&items[{d.print(\"index\");->position}]";
        let source = format!(
            "#é🙂#\nd:@\"debug\";get<&int32>:(items<&int32[{capacity}]>,position<{ty}>){{->{access}}};values<int32[{capacity}]>:{initial};view:get(&values,{index});d.print(*view)"
        );
        let start = source.find(access).unwrap();
        let end = start + access.len();
        let case = Case::new(&source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert_eq!(output.stdout, b"index\n");
            assert_eq!(output.stderr, format!("panic[P001]: index {shown} is outside initialized length {length} at bytes {start}..{end}\n").as_bytes());
        }
    }
}
