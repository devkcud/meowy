use super::Case;
use super::file_modules::case;

#[test]
pub(crate) fn panic_sites_map_imported_failures_to_local_files_and_bytes() {
    for (source, fault, code, message) in [
        (
            "d:@\"debug\";d.panic(\"stop\")",
            "d.panic(\"stop\")",
            "P006",
            "stop",
        ),
        (
            "f<int8>:(x<int8>){->x+1};->n:f(127)",
            "x+1",
            "P002",
            "int8 + overflow (left 127, right 1; range -128..127)",
        ),
        (
            "f<int32>:(i<int32>){xs:[7];->xs[i]};->n:f(2)",
            "xs[i]",
            "P001",
            "index 2 is outside initialized length 1",
        ),
        (
            "f<int32[1]>:(xs<int32[1]>){->xs.add(2)};->xs:f([1])",
            "xs.add(2)",
            "P003",
            "bounded list is full (length 1, capacity 1)",
        ),
    ] {
        let source = format!("#é🙂#\n{source}");
        let case = case(
            "d:@\"debug\";d.print(\"never\");m:@\"./sub/value.mwy\"",
            &[("sub/value.mwy", &source)],
        );
        let path = case.path.join("sub/value.mwy");
        let start = source.find(fault).unwrap();
        let expected = format!(
            "panic[{code}]: {message} at \"{}\" bytes {start}..{}\n",
            path.display(),
            start + fault.len()
        );
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes());
        }
    }
}

#[test]
pub(crate) fn panic_sites_keep_entry_offsets_and_skip_later_effects() {
    let source = "d:@\"debug\";m:@\"./value.mwy\";d.panic(\"entry\");d.print(\"never\")";
    let case = case(
        source,
        &[("value.mwy", "d:@\"debug\";d.print(\"init\");->n:7")],
    );
    let start = source.find("d.panic").unwrap();
    let expected = format!(
        "panic[P006]: entry at \"{}\" bytes {start}..{}\n",
        case.source.display(),
        start + "d.panic(\"entry\")".len()
    );
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"init\n");
        assert_eq!(output.stderr, expected.as_bytes());
    }
}

#[test]
pub(crate) fn panic_sites_preserve_single_file_diagnostics() {
    let source = "d:@\"debug\";d.panic(\"stop\")";
    let case = Case::new(source);
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!("panic[P006]: stop at bytes 11..{}\n", source.len()).as_bytes()
        );
    }
}

#[cfg(unix)]
#[test]
pub(crate) fn panic_sites_use_canonical_escaped_labels_for_aliases() {
    let name = "é\"\n.mwy";
    let source = "d:@\"debug\";d.panic(\"stop\")";
    let case = case("a:@\"./alias.mwy\"", &[(name, source)]);
    std::os::unix::fs::symlink(name, case.path.join("alias.mwy")).unwrap();
    let path = format!("{}/é\\\"\\u000a.mwy", case.path.display());
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: stop at \"{path}\" bytes 11..{}\n",
                source.len()
            )
            .as_bytes()
        );
    }
}

#[test]
pub(crate) fn panic_sites_name_nested_dependency_instead_of_importer() {
    let source = "d:@\"debug\";d.panic(\"shared\")";
    let case = case(
        "a:@\"./left.mwy\";b:@\"./right.mwy\"",
        &[
            ("left.mwy", "s:@\"./shared.mwy\";->n:1"),
            ("right.mwy", "s:@\"./shared.mwy\";->n:2"),
            ("shared.mwy", source),
        ],
    );
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: shared at \"{}\" bytes 11..{}\n",
                case.path.join("shared.mwy").display(),
                source.len()
            )
            .as_bytes()
        );
    }
}
