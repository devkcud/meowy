use super::Case;

#[test]
pub fn doc_fences_preserve_literal_and_ordinary_comment_execution() {
    Case::new("d:@\"debug\";# | ordinary | #d.print(\"#|literal|#\");d.print(\"x{# ordinary } \" # 7}y\")")
        .runs(b"#|literal|#\nx7y\n");
}

#[test]
pub fn doc_fences_keep_unsupported_attachment_distinct_from_lexical_failure() {
    for source in [
        "d:@\"debug\";d.print(\"must not run\");#| # Heading |#x:7",
        "#!|| module |!# shorter ||!#\nx:7",
        "f<int32>:(#| input |#x<int32>){->x};v:f(7)",
        "s:\"before {{#|| } \" #| inner |# ||# x:7;->x}} after\"",
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(error.starts_with("error[B001]:"), "{source}: {error}");
            assert!(error.contains("documentation comment attachment"));
        }
    }
}

#[test]
pub fn doc_fences_report_exact_unclosed_opener_spans() {
    for (source, start, end) in [
        ("#\u{e9}#\r\n#|| unclosed", 6, 9),
        ("#!|| wrong |!#", 0, 4),
        ("#| wrong |!#", 0, 2),
    ] {
        let output = Case::new(source).command("check", &["--json"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("\"code\":\"E002\""), "{error}");
        assert!(error.contains(&format!("\"start\":{start}")), "{error}");
        assert!(error.contains(&format!("\"end\":{end}")), "{error}");
        assert!(error.contains("unclosed documentation comment"));
    }
}
