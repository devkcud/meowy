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
    let case = Case::new("values:[1,2,3]");
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
        ("owner:1;view:&owner;view.{->*self}", "B001"),
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
