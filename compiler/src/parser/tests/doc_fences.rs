use crate::ast::Span;
use crate::parser::{parse, parse_documented};

#[test]
pub(crate) fn doc_fences_attach_in_supported_declaration_positions() {
    for source in [
        "#| summary |#x:1",
        "#!|| module ||!#\nx:1",
        "f<int32>:(#| input |#x<int32>){->x}",
        "<R>:<{#|| field ||#n<int32>}>",
    ] {
        assert!(parse(source).is_ok(), "{source}");
    }
    assert_eq!(parse("|false|{#| unused |#}").unwrap_err()[0].code, "E801");
}

#[test]
pub(crate) fn doc_fences_in_interpolation_keep_original_byte_offsets() {
    let source = "#\u{e9}#\r\ns:\"before {{#|| } \" #| inner |# ||# x:7;->x}} after\"";
    let parsed = parse_documented(source).unwrap();
    let start = source.find("#||").unwrap();
    assert_eq!(parsed.docs[0].open, Span::new(start, start + 3));
    assert!(parse(source).is_ok());
}

#[test]
pub(crate) fn doc_fences_distinguish_attachment_and_lexical_errors() {
    assert!(parse("# | ordinary | #x:1").is_ok());
    assert!(parse("##x:1").is_ok());
    assert!(parse("x:\"#| literal |#\"").is_ok());
    assert!(parse("#| |#x:1").is_ok());
    assert_eq!(parse("#|| unfinished |#").unwrap_err()[0].code, "E002");
    assert_eq!(parse("#| orphan |#").unwrap_err()[0].code, "E801");
}
