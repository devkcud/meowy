use crate::ast::Span;
use crate::parser::parse;

#[test]
pub(crate) fn doc_fences_require_attachment_support_in_every_declaration_position() {
    for (source, opener) in [
        ("#| summary |#x:1", "#|"),
        ("#!|| module ||!#\nx:1", "#!||"),
        ("f<int32>:(#| input |#x<int32>){->x}", "#|"),
        ("<R>:<{#|| field ||#n<int32>}>", "#||"),
        ("|false|{#| unused |#}", "#|"),
    ] {
        let errors = parse(source).unwrap_err();
        assert_eq!(errors.len(), 1, "{source}");
        assert_eq!(errors[0].code, "B001");
        assert!(
            errors[0]
                .message
                .contains("documentation comment attachment")
        );
        let start = source.find(opener).unwrap();
        assert_eq!(errors[0].span, Span::new(start, start + opener.len()));
    }
}

#[test]
pub(crate) fn doc_fences_in_interpolation_keep_original_byte_offsets() {
    let source = "#\u{e9}#\r\ns:\"before {{#|| } \" #| inner |# ||# x:7;->x}} after\"";
    let errors = parse(source).unwrap_err();
    assert_eq!(errors[0].code, "B001");
    let start = source.find("#||").unwrap();
    assert_eq!(errors[0].span, Span::new(start, start + 3));
}

#[test]
pub(crate) fn doc_fences_do_not_silently_become_ordinary_trivia() {
    assert!(parse("# | ordinary | #x:1").is_ok());
    assert!(parse("##x:1").is_ok());
    assert!(parse("x:\"#| literal |#\"").is_ok());
    assert_eq!(parse("#| |#x:1").unwrap_err()[0].code, "B001");
    assert_eq!(parse("#|| unfinished |#").unwrap_err()[0].code, "E002");
}
