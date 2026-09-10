use super::{Source, Span, emit_ir, emit_ir_with_sources, native_ir};

#[test]
pub(crate) fn mapped_panic_sites_use_local_ranges_and_preserve_empty_map_output() {
    let source = "d:@\"debug\";d.panic(\"stop\")";
    let parsed = crate::parser::parse_documented_at(source, 100).unwrap();
    let program = crate::check::check(&parsed.block).unwrap();
    let sources = [Source {
        path: "folder/é\"\n.mwy".into(),
        span: Span::new(100, 100 + source.len()),
    }];
    let ir = emit_ir_with_sources(&program, &sources).unwrap();
    let start = source.find("d.panic").unwrap();
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: stop at \"folder/é\\\"\\u000a.mwy\" bytes {start}..{}\n",
                source.len()
            )
            .as_bytes()
        );
        let output = native_ir(&emit_ir(&program).unwrap(), release, false);
        assert_eq!(
            output.stderr,
            format!(
                "panic[P006]: stop at bytes {}..{}\n",
                100 + start,
                100 + source.len()
            )
            .as_bytes()
        );
    }
}

#[test]
pub(crate) fn mapped_panic_sites_reject_invalid_maps_and_unmapped_sites() {
    let source = "d:@\"debug\";d.panic(\"stop\")";
    let program = crate::compile(source).unwrap();
    for ranges in [
        vec![(9, 8)],
        vec![(0, 10), (10, 30)],
        vec![(30, 40), (0, 20)],
        vec![(1, 2)],
    ] {
        let sources = ranges
            .into_iter()
            .map(|(start, end)| Source {
                path: "file.mwy".into(),
                span: Span::new(start, end),
            })
            .collect::<Vec<_>>();
        assert!(emit_ir_with_sources(&program, &sources).is_err());
    }
    for path in [String::new(), "x".repeat(1024 * 1024 + 1)] {
        assert!(
            emit_ir_with_sources(
                &program,
                &[Source {
                    path,
                    span: Span::new(0, source.len())
                }]
            )
            .is_err()
        );
    }
    let sources = (0..65)
        .map(|id| Source {
            path: "file.mwy".into(),
            span: Span::new(id * 100, id * 100 + 50),
        })
        .collect::<Vec<_>>();
    assert!(emit_ir_with_sources(&program, &sources).is_err());
}

#[test]
pub(crate) fn mapped_panic_sites_keep_nested_failure_identity_and_deduplicate_paths() {
    let source = "d:@\"debug\";f<never>:(){d.panic(\"inner\")};d.panic(\"outer {f()} never\")";
    let program = crate::compile(source).unwrap();
    let sources = [Source {
        path: "file.mwy".into(),
        span: Span::new(0, source.len()),
    }];
    let ir = emit_ir_with_sources(&program, &sources).unwrap();
    let encoded = "\\66\\69\\6C\\65\\2E\\6D\\77\\79";
    assert_eq!(ir.matches(encoded).count(), 1);
    let start = source.find("d.panic").unwrap();
    let end = start + "d.panic(\"inner\")".len();
    for release in [false, true] {
        let output = native_ir(&ir, release, false);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            output.stderr,
            format!("panic[P006]: outer panic[P006]: inner at \"file.mwy\" bytes {start}..{end}\n")
                .as_bytes()
        );
    }
}
