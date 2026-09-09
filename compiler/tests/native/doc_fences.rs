use super::Case;

#[test]
pub fn doc_fences_preserve_literal_and_ordinary_comment_execution() {
    Case::new("d:@\"debug\";# | ordinary | #d.print(\"#|literal|#\");d.print(\"x{# ordinary } \" # 7}y\")")
        .runs(b"#|literal|#\nx7y\n");
}

#[test]
pub fn doc_fences_compile_supported_attachment_in_both_profiles() {
    for (source, output) in [
        (
            "d:@\"debug\";#| # Heading |#x:7;d.print(x)",
            b"7\n".as_slice(),
        ),
        ("#!|| module |!# shorter ||!#\nx:7", b"".as_slice()),
        (
            "d:@\"debug\";f<int32>:(#| input |#x<int32>){->x};d.print(f(7))",
            b"7\n".as_slice(),
        ),
        (
            "d:@\"debug\";d.print(\"before {{#|| } \" #| inner |# ||# x:7;->x}} after\")",
            b"before 7 after\n".as_slice(),
        ),
    ] {
        Case::new(source).runs(output);
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
    }
}
