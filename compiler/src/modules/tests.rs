use super::{Graph, MAX_DEPTH, MAX_EDGES, MAX_FILES, MAX_SOURCE};
use crate::driver::Scratch;
use std::fs;

pub(crate) fn graph(files: &[(&str, &str)]) -> (Scratch, Graph) {
    let temp = Scratch::new(&std::env::temp_dir()).unwrap();
    for (name, source) in files {
        let path = temp.path.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    let entry = temp.path.join("main.mwy");
    let source = fs::read_to_string(&entry).unwrap();
    let loaded = Graph::load(&entry, &source).unwrap();
    (temp, loaded)
}

#[test]
pub(crate) fn file_modules_keep_canonical_diamond_order_and_file_spans() {
    let (_temp, graph) = graph(&[
        (
            "main.mwy",
            "a:@\"./a.mwy\";b:@\"./b.mwy\";d:@\"debug\";d.print(a.n+b.n)",
        ),
        ("a.mwy", "s:@\"./shared.mwy\";->n:s.n+1"),
        ("b.mwy", "s:(@\"./sub/../shared.mwy\");->n:s.n+2"),
        ("shared.mwy", "->n:7"),
        ("sub/unused.mwy", ""),
    ]);
    assert_eq!(graph.files.len(), 4);
    assert_eq!(
        graph
            .order
            .iter()
            .map(|id| graph.files[*id].path.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<_>>(),
        ["shared.mwy", "a.mwy", "b.mwy", "main.mwy"]
    );
    assert_eq!(graph.imports.len(), 4);
    for file in &graph.files {
        assert_eq!(graph.file(file.parsed.block.span).path, file.path);
        assert_eq!(file.parsed.block.span.start, file.base);
        assert!(file.parsed.block.span.end <= file.base + file.source.len());
    }
    graph.compile().unwrap();
}

#[test]
pub(crate) fn file_modules_report_imported_type_errors_at_local_utf8_spans() {
    let (_temp, graph) = graph(&[
        ("main.mwy", "m:@\"./value.mwy\""),
        ("value.mwy", "#é🙂#\n->n:missing"),
    ]);
    let error = &graph.compile().unwrap_err()[0];
    assert_eq!(error.code, "E201");
    let file = graph.file(error.span);
    assert!(file.path.ends_with("value.mwy"));
    let local = file.local(error);
    assert_eq!(&file.source[local.span.start..local.span.end], "missing");
}

#[test]
pub(crate) fn file_modules_reject_cycles_missing_paths_and_different_manifest_contexts() {
    let temp = Scratch::new(&std::env::temp_dir()).unwrap();
    let entry = temp.path.join("main.mwy");
    let source = "m:@\"./other.mwy\"";
    fs::write(&entry, source).unwrap();
    let missing = Graph::load(&entry, source).err().unwrap();
    assert_eq!(missing.errors[0].code, "E501");
    assert!(missing.errors[0].message.contains("./other.mwy"));
    fs::write(temp.path.join("other.mwy"), "m:@\"./main.mwy\"").unwrap();
    let cycle = Graph::load(&entry, source).err().unwrap();
    assert_eq!(cycle.errors[0].code, "E502");
    assert!(cycle.path.ends_with("other.mwy"));
    assert!(cycle.errors[0].message.contains("main.mwy ->"));
    fs::create_dir(temp.path.join("pkg")).unwrap();
    fs::write(temp.path.join("pkg/mod.mwy"), "").unwrap();
    fs::write(temp.path.join("pkg/value.mwy"), "->n:1").unwrap();
    let failure = Graph::load(&entry, "m:@\"./pkg/value.mwy\"").err().unwrap();
    assert_eq!(failure.errors[0].code, "B001");
}

#[test]
pub(crate) fn file_modules_bound_depth_files_edges_and_source_bytes() {
    let temp = Scratch::new(&std::env::temp_dir()).unwrap();
    let entry = temp.path.join("main.mwy");
    fs::write(&entry, "").unwrap();
    for id in 0..MAX_DEPTH {
        fs::write(
            temp.path.join(format!("{id}.mwy")),
            format!("m:@\"./{}.mwy\"", id + 1),
        )
        .unwrap();
    }
    fs::write(temp.path.join(format!("{MAX_DEPTH}.mwy")), "->n:1").unwrap();
    let failure = Graph::load(&entry, "m:@\"./0.mwy\"").err().unwrap();
    assert_eq!(failure.errors[0].code, "B001");
    let mut source = String::new();
    for id in 0..MAX_FILES {
        fs::write(temp.path.join(format!("leaf{id}.mwy")), "->n:1").unwrap();
        source.push_str(&format!("m{id}:@\"./leaf{id}.mwy\";"));
    }
    assert_eq!(
        Graph::load(&entry, &source).err().unwrap().errors[0].code,
        "B001"
    );
    let source = (0..=MAX_EDGES)
        .map(|id| format!("m{id}:@\"./leaf0.mwy\";"))
        .collect::<String>();
    assert_eq!(
        Graph::load(&entry, &source).err().unwrap().errors[0].code,
        "B001"
    );
    assert_eq!(
        Graph::load(&entry, &" ".repeat(MAX_SOURCE + 1))
            .err()
            .unwrap()
            .errors[0]
            .code,
        "B001"
    );
}

#[test]
pub(crate) fn file_modules_preserve_export_and_execution_gates() {
    for (source, module, code) in [
        ("m:@\"./value.mwy\";v:m.private", "private:1;->n:2", "E201"),
        ("m:@\"./value.mwy\"", "->n:=2", "B001"),
        ("m:@\"./value.mwy\"", "f<int32>:(){->1};->f:f", "E214"),
        ("m:@\"./value.mwy\"", "->v:{->n:=2}", "B001"),
        ("m:@\"./value.mwy\"", "x:1;->v:&x", "B001"),
        ("m:@\"./value.mwy\";p:&(m.n)", "->n:2", "B001"),
        ("m:@\"./value.mwy\";f<int32>:(){->m.n}", "->n:2", "B001"),
        ("f<null>:(){m:@\"./value.mwy\";v:m.n}", "->n:2", "B001"),
        ("m:@\"./value.mwy\"", "#| value |#\n->n:2", "B001"),
    ] {
        let (_temp, graph) = graph(&[("main.mwy", source), ("value.mwy", module)]);
        assert_eq!(
            graph.compile().unwrap_err()[0].code,
            code,
            "{source}: {module}"
        );
    }
}

#[test]
pub(crate) fn file_modules_reject_invalid_import_contents_and_exact_paths() {
    let temp = Scratch::new(&std::env::temp_dir()).unwrap();
    let entry = temp.path.join("main.mwy");
    fs::write(&entry, "").unwrap();
    fs::write(temp.path.join("value.mwy"), [0xff]).unwrap();
    let invalid = Graph::load(&entry, "m:@\"./value.mwy\"").err().unwrap();
    assert_eq!(invalid.errors[0].code, "E001");
    assert!(invalid.path.ends_with("value.mwy"));
    fs::create_dir(temp.path.join("folder.mwy")).unwrap();
    for source in ["m:@\"./folder.mwy\"", "m:@\"./value\""] {
        assert_eq!(
            Graph::load(&entry, source).err().unwrap().errors[0].code,
            "E501"
        );
    }
    fs::write(temp.path.join("value.mwy"), "-><Point>:<int32>").unwrap();
    Graph::load(&entry, "m:@\"./value.mwy\"")
        .unwrap()
        .compile()
        .unwrap();
    fs::write(temp.path.join("value.mwy"), "->n:missing\n").unwrap();
    let graph = Graph::load(&entry, "m:@\"./value.mwy\"").unwrap();
    fs::write(temp.path.join("value.mwy"), "->n:7").unwrap();
    assert_eq!(graph.compile().unwrap_err()[0].code, "E201");
}

#[cfg(unix)]
#[test]
pub(crate) fn file_modules_share_symlink_identity_and_reject_symlink_cycles() {
    let (temp, _) = graph(&[("main.mwy", "m:@\"./value.mwy\""), ("value.mwy", "->n:7")]);
    std::os::unix::fs::symlink("value.mwy", temp.path.join("alias.mwy")).unwrap();
    let entry = temp.path.join("main.mwy");
    let loaded = Graph::load(&entry, "a:@\"./value.mwy\";b:@\"./alias.mwy\"").unwrap();
    assert_eq!(loaded.files.len(), 2);
    assert_eq!(loaded.imports.values().collect::<Vec<_>>(), [&1, &1]);
    std::os::unix::fs::symlink("main.mwy", temp.path.join("entry.mwy")).unwrap();
    let cycle = Graph::load(&temp.path.join("entry.mwy"), "m:@\"./entry.mwy\"")
        .err()
        .unwrap();
    assert_eq!(cycle.errors[0].code, "E502");
}

#[test]
pub(crate) fn file_modules_bound_total_snapshot_bytes() {
    let temp = Scratch::new(&std::env::temp_dir()).unwrap();
    let entry = temp.path.join("main.mwy");
    fs::write(&entry, "").unwrap();
    let mut source = String::new();
    let padding = " ".repeat(MAX_SOURCE);
    for id in 0..4 {
        fs::write(temp.path.join(format!("{id}.mwy")), &padding).unwrap();
        source.push_str(&format!("m{id}:@\"./{id}.mwy\";"));
    }
    assert_eq!(
        Graph::load(&entry, &source).err().unwrap().errors[0].code,
        "B001"
    );
}

#[test]
pub(crate) fn file_function_reexports_share_call_ids_without_merging_distinct_functions() {
    let (_temp, graph) = graph(&[
        (
            "main.mwy",
            "ops:@\"./ops.mwy\";api:@\"./api.mwy\";other:@\"./other.mwy\";ops.inc(1);api.bump(2);other.inc(3)",
        ),
        ("ops.mwy", "->inc<int32>:(x<int32>){->x+1}"),
        (
            "api.mwy",
            "ops:@\"./ops.mwy\";->bump<(int32)->int32>:ops.inc",
        ),
        ("other.mwy", "->inc<int32>:(x<int32>){->x+1}"),
    ]);
    let program = graph.compile().unwrap();
    assert_eq!(program.functions.len(), 2);
    let Some(crate::hir::Stmt::Bind { value, .. }) = program.body.stmts.last() else {
        panic!("entry initializer")
    };
    let crate::hir::ExprKind::Block(block) = &value.kind else {
        panic!("entry block")
    };
    let calls = block
        .stmts
        .iter()
        .filter_map(|stmt| match stmt {
            crate::hir::Stmt::Expr(crate::hir::Expr {
                kind: crate::hir::ExprKind::Call { id, .. },
                ..
            }) => Some(*id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls,
        [
            program.functions[0].id,
            program.functions[0].id,
            program.functions[1].id
        ]
    );
}

#[test]
pub(crate) fn exported_types_retain_underlying_nominal_and_structural_types() {
    let (_temp, graph) = graph(&[
        (
            "main.mwy",
            "a:@\"./types.mwy\";b:@\"./api.mwy\";f<a.Handle>:(p<b.Handle>){->p};g<a.Number>:(n<b.Number>){->n}",
        ),
        (
            "types.mwy",
            "memory:@\"memory\";-><Handle>:<memory.Allocator>;-><Number>:<uint64>",
        ),
        (
            "api.mwy",
            "a:@\"./types.mwy\";-><Handle>:<a.Handle>;-><Number>:<a.Number>",
        ),
    ]);
    let program = graph.compile().unwrap();
    let allocator = crate::hir::Type::Foundation(crate::hir::FoundationType::Allocator);
    assert_eq!(program.functions[0].result, allocator);
    assert_eq!(program.locals[program.functions[0].params[0]], allocator);
    let number = crate::hir::Type::Int {
        bits: 64,
        signed: false,
    };
    assert_eq!(program.functions[1].result, number);
    assert_eq!(program.locals[program.functions[1].params[0]], number);
}

#[test]
pub(crate) fn discovered_imports_include_nested_expressions_in_source_order() {
    let source = "f<null>:(){m:@\"./first.mwy\"};|false|{m:@\"./second.mwy\"};d:@\"debug\";d.print(\"value {(@\"./third.mwy\").n}\");x:[(@\"./fourth.mwy\").n];# @\"./ignored.mwy\" #";
    let block = crate::parser::parse_documented_at(source, 100)
        .unwrap()
        .block;
    let imports = super::discover::imports(&block).unwrap();
    assert_eq!(
        imports
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["./first.mwy", "./second.mwy", "./third.mwy", "./fourth.mwy"]
    );
    for (name, span) in imports {
        assert_eq!(
            &source[span.start - 100..span.end - 100],
            format!("@{name:?}")
        );
    }
}

#[test]
pub(crate) fn discovered_imports_traverse_type_operands_and_bound_work() {
    let source = "<A>:<int32[(@\"./one.mwy\").n]>;f<(&int32[(@\"./two.mwy\").n])->int32[(@\"./three.mwy\").n]>;-><B>:@\"./four.mwy\"";
    let block = crate::parser::parse(source).unwrap();
    let imports = super::discover::imports(&block).unwrap();
    assert_eq!(
        imports
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["./one.mwy", "./two.mwy", "./three.mwy", "./four.mwy"]
    );
    let mut scan = super::discover::Scan {
        pending: Vec::new(),
        work: super::discover::MAX_WORK,
        span: block.span,
    };
    assert_eq!(
        scan.push(super::discover::Node::Block(&block))
            .unwrap_err()
            .code,
        "B001"
    );
    let (_temp, graph) = graph(&[
        ("main.mwy", "|false|{m:@\"./value.mwy\"}"),
        ("value.mwy", "->n:7"),
    ]);
    assert_eq!(graph.files.len(), 2);
    graph.compile().unwrap();
}
