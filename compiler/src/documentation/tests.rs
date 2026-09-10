use super::*;

#[test]
pub(crate) fn documentation_attaches_and_derives_checked_signatures() {
    let source = "#!| Module links to [[add]]. |!#\n#| Add [[left]] and [[right]] using [[<int32>]]. |#\nadd<int32>:(#| Left. |#left<int32>,#| Right. |#right<int32>){->left+right}";
    let (_, model) = checked(source, true).unwrap();
    let model = model.unwrap();
    assert_eq!(model.entries.len(), 4);
    assert_eq!(model.entries[1].signature, "(int32,int32)->int32");
    assert_eq!(model.entries[2].signature, "int32");
    assert!(model.entries.iter().all(|entry| entry.checked));
    model.require_public().unwrap();
}

#[test]
pub(crate) fn documentation_diagnoses_orphan_duplicate_trailing_and_module_placement() {
    for source in [
        "#| orphan |#->1",
        "#| first |# #| second |#x:1",
        "x:1 #| trailing |#\ny:2",
        "x:1\n#!| late |!#",
        "#!| first |!#\n#!| second |!#",
        "|false|{#| orphan |#}",
    ] {
        assert_eq!(
            checked(source, true).unwrap_err()[0].code,
            "E801",
            "{source}"
        );
    }
}

#[test]
pub(crate) fn documentation_resolves_scoped_links_and_record_members_without_reads() {
    let source = "<R>:<{#| Field of [[<R>]]. |#n<int32>}>;r<R>:{->n:7};#| Read [[r.n]] through its actual type. |#v:r.n";
    let (_, model) = checked(source, true).unwrap();
    assert!(
        model
            .unwrap()
            .entries
            .iter()
            .any(|entry| entry.kind == Kind::Field && entry.signature == "int32")
    );
    for source in [
        "#| [[missing]] |#x:1",
        "x:1;#| [[x.nope]] |#y:2",
        "#| [[<Missing>]] |#x:1",
        "#| [[later]] |#x:1;later:2",
        "#| [[inside]] |#f<int32>:(){inside:1;->inside}",
    ] {
        assert_eq!(
            checked(source, true).unwrap_err()[0].code,
            "E802",
            "{source}"
        );
    }
    checked("d:@\"debug\";#| [[d.print|printing]] |#alias:d.print", true).unwrap();
}

#[test]
pub(crate) fn documentation_ignores_links_in_code_and_rejects_malformed_links() {
    checked(
        "#||\nLiteral `[[unknown]]`.\n```text\n[[also_unknown]]\n```\n||#x:1",
        true,
    )
    .unwrap();
    for source in [
        "#| [[x()]] |#x:1",
        "#| [[missing |#x:1",
        "#| [[x|bad|label]] |#x:1",
    ] {
        assert_eq!(checked(source, true).unwrap_err()[0].code, "E802");
    }
}

#[test]
pub(crate) fn documentation_checks_example_metadata_without_executing_examples() {
    let source = "#||\n```meowy run\nd:@\"debug\";d.panic(\"not run\")\n```\n```output\nanything\n```\n||#x:1";
    let (_, model) = checked(source, true).unwrap();
    let model = model.unwrap();
    assert_eq!(model.entries[1].examples.len(), 1);
    assert!(!model.entries[1].examples[0].ran);
    for body in [
        "```meowy mystery\nx:1\n```",
        "```meowy reject=B001\nx:1\n```",
        "```output\nx\n```",
    ] {
        assert_eq!(
            checked(&format!("#||\n{body}\n||#x:1"), true).unwrap_err()[0].code,
            "E803"
        );
    }
}

#[test]
pub(crate) fn documentation_preserves_normalized_source_mapping_and_interpolation() {
    let source = "#||\r\n    Unicode \u{e9} [[missing]].\r\n||#x:1";
    let error = checked(source, true).unwrap_err().remove(0);
    assert_eq!(error.code, "E802");
    assert_eq!(error.span.start, source.find("[[missing]]").unwrap());
    checked(
        "d:@\"debug\";d.print(\"value {{#| Inner. |#x:7;->x}}\")",
        true,
    )
    .unwrap();
}

#[test]
pub(crate) fn documentation_renderer_escapes_active_content_and_keeps_source_private() {
    let source = "#||\n<script>alert(1)</script>\n\n[bad](javascript:alert(1))\n![image](https://example.test/secret)\n||#x:1";
    let (_, model) = checked(source, true).unwrap();
    let page = render::page(&model.unwrap(), "<entry>");
    assert!(!page.contains("<script>"));
    assert!(!page.contains("href=\"javascript:"));
    assert!(!page.contains("<img"));
    assert!(page.contains("&lt;entry&gt;"));
    assert!(page.contains("@media"));
}

#[test]
pub(crate) fn documentation_public_policy_and_budgets_fail_explicitly() {
    let (_, model) = checked("x:1", true).unwrap();
    assert_eq!(model.unwrap().require_public().unwrap_err().code, "E803");
    let source = format!("#|{}|#x:1", "x".repeat(MAX_SOURCE));
    assert_eq!(checked(&source, true).unwrap_err()[0].code, "B001");
}

#[test]
pub(crate) fn exported_types_documentation_preserves_type_roles_and_declaration_spans() {
    let source = "#| Public counter. |#\n-><Count>:<int32>";
    let (_, model) = checked(source, true).unwrap();
    let model = model.unwrap();
    let entry = model
        .entries
        .iter()
        .find(|entry| entry.name == "Count")
        .unwrap();
    assert_eq!(entry.kind, Kind::Type);
    assert!(entry.public);
    assert!(entry.checked);
    assert_eq!(entry.signature, "int32");
    assert_eq!(entry.span.start, source.find("->").unwrap());
    model.require_public().unwrap();
}
