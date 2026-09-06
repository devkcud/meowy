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
