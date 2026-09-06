use super::rejects;

#[test]
pub(crate) fn effect_prefixes_choose_context_after_once_only_checking() {
    for source in [
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->300}]",
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->255+1}]",
        "d:@\"debug\";values<uint8[1]><int32[1]>:[{x:1;d.print(1);->x}]",
        "d:@\"debug\";x<uint8>:1;values<uint8[1]><string[1]>:[{x:\"x\";d.print(1);->x}]",
        "d:@\"debug\";values<uint8[1][1]><uint16[1][1]>:[{d.print(1);->[300]}]",
        "d:@\"debug\";<A>:<{n<uint8>}>;<B>:<{n<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->n:300}]",
        "d:@\"debug\";<A>:<{n<uint8>}>;<B>:<{n<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->{->n:300}}]",
        "d:@\"debug\";<R>:<{n<int32>}>;<S>:<{-><R><int32>;n<int32>}>;<U>:<R><S>;values<R[1]><U[1]>:[{d.print(1);->{->n:1};->n:2}]",
        "d:@\"debug\";byte<uint8>:1;values<uint8[2]><uint16[2]>:[{d.print(1);->1},byte]",
    ] {
        let result = crate::compile(source);
        assert!(result.is_ok(), "{source}: {result:?}");
    }
    rejects(
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1}]",
        "E207",
    );
    rejects(
        "d:@\"debug\";values<uint8[2]><uint16[2]>:[{d.print(1);->1},{byte<uint8>:2;->byte}]",
        "B001",
    );
    rejects(
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.print(1);->1;d.print(2)}]",
        "B001",
    );
    rejects(
        "d:@\"debug\";values<int32[1]><string[1]>:[{x:{->n:1};d.print(1);->x}]",
        "B001",
    );
    rejects(
        "d:@\"debug\";x<uint8>:1;<A>:<{x<uint16>;y<uint8>}>;<B>:<{x<uint16>;y<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->x:300;->y:x}]",
        "B001",
    );
}

#[test]
pub(crate) fn effectful_suffix_unknown_locals_keep_their_primitive_types() {
    for source in [
        "f<null>:(x<uint8>,flag<boolean>,text<string>,real<float32>,empty<null>){a<uint8[1]><uint16[1]>:[{0;->x}];b<boolean[1]><int32[1]>:[{0;->flag}];c<string[1]><int32[1]>:[{0;->text}];d<float32[1]><float64[1]>:[{0;->real}];e<null[1]><int32[1]>:[{0;->empty}]}",
        "read<uint8>:(){->1};values<uint8[1]><uint16[1]>:[{x:read();->x+1}]",
        "d:@\"debug\";values<int32[1]><string[1]>:[{x:=1;d.print(1);->x}]",
        "text:=\"old\";values<string[1]><int32[1]>:[{text=\"new\";->text}]",
        "<A>:<{n<uint8>}>;<B>:<{n<uint16>}>;read<uint8>:(){->1};values<A[1]><B[1]>:[{x:read();->n:x}]",
    ] {
        let result = crate::compile(source);
        assert!(result.is_ok(), "{source}: {result:?}");
    }
    for (source, code) in [
        ("x<uint8>:=1;values<uint16[1]><string[1]>:[{0;->x}]", "E207"),
        (
            "x:=1;f<null>:(){values<int32[1]><string[1]>:[{0;->x}]}",
            "B001",
        ),
        (
            "<U>:<int32><string>;x<U>:=1;values<int32[1]><string[1]>:[{0;->x}]",
            "B001",
        ),
        (
            "x:=1;r:&x;values<int32[1]><string[1]>:[{x=2;->x}];v:*r",
            "E302",
        ),
    ] {
        rejects(source, code);
    }
}

#[test]
pub(crate) fn unknown_suffix_control_does_not_discard_live_proof_possibilities() {
    for source in [
        "f<null>:(flag<boolean>){|!flag|{v<boolean[1]><int32[1]>:[{0;->flag&&(1/0==0)}]}}",
        "f<null>:(flag<boolean>){|!flag|{v<boolean[1]><int32[1]>:[{0;->(flag)&&(1/0==0)}]}}",
        "f<null>:(flag<boolean>){|flag|{v<boolean[1]><int32[1]>:[{0;->flag||(1/0==0)}]}}",
        "f<null>:(flag<boolean>){|flag|{v<boolean[1]><int32[1]>:[{0;->((flag))||(1/0==0)}]}}",
        "f<null>:(flag<boolean>){|!flag|{v<boolean[1]><int32[1]>:[{0;->flag&&{->true;->false}}]}}",
        "f<null>:(flag<boolean>){|!flag|{v<boolean[1]><int32[1]>:[{0;->(flag)&&{->true;->false}}]}}",
        "f<null>:(flag<boolean>){|flag|{v<boolean[1]><int32[1]>:[{0;->flag||{}}]}}",
        "d:@\"debug\";flag:=false;v<boolean[1]><string[1]>:[{d.panic(\"stop\");->flag&&(1/0==0)}]",
    ] {
        let result = crate::compile(source);
        assert!(result.is_ok(), "{source}: {result:?}");
    }
    for (source, code) in [
        (
            "flag:=false;|!flag|{v<boolean[1]><int32[1]>:[{flag=true;->flag&&(1/0==0)}]}",
            "E107",
        ),
        (
            "f<null>:(flag<boolean>){v<boolean[1]><int32[1]>:[{0;->flag&&(1/0==0)}]}",
            "E107",
        ),
        (
            "<A>:<{state<boolean>;n<int8>}>;<B>:<{state<boolean>;n<int16>}>;f<null>:(flag<boolean>){v<A[1]><B[1]>:[{0;->state:true||flag;->n:1;->n:2}]}",
            "E205",
        ),
        (
            "<A>:<{state<boolean>;n<int8>}>;<B>:<{state<boolean>;n<int16>}>;f<null>:(flag<boolean>){|!flag|{v<A[1]><B[1]>:[{0;->state:flag&&(1/0==0);->n:1}]}}",
            "B001",
        ),
    ] {
        rejects(source, code);
    }
}

#[test]
pub(crate) fn ordinary_unknown_reads_are_never_deferred_as_constants() {
    let program = crate::compile("x:=1;v<int32[2]><string[2]>:[x+1,{x=9;->3}]").unwrap();
    let crate::hir::Stmt::Bind { value, .. } = &program.body.stmts[1] else {
        panic!("list binding")
    };
    let mut value = value;
    while let crate::hir::ExprKind::Coerce { value: inner } = &value.kind {
        value = inner;
    }
    let crate::hir::ExprKind::List { values, .. } = &value.kind else {
        panic!("list value")
    };
    assert!(matches!(
        values[0].kind,
        crate::hir::ExprKind::Binary { .. }
    ));
    assert!(matches!(values[1].kind, crate::hir::ExprKind::Block(_)));
    let tree = crate::parser::parse("x:=1;x+1").unwrap();
    let mut checker = crate::check::Checker::new();
    checker.stmt(&tree.stmts[0]).unwrap();
    let crate::ast::StmtKind::Expr(value) = &tree.stmts[1].kind else {
        panic!("scalar expression")
    };
    assert!(checker.list_scalar(value).unwrap().is_none());
    assert!(!checker.list_deferred(value).unwrap());
}

#[test]
pub(crate) fn effect_prefix_reach_and_source_errors_precede_suffix_selection() {
    for (source, code) in [
        (
            "d:@\"debug\";values<int8[1]><uint8[1]>:[{d.panic(\"stop\");->127+1}]",
            "E207",
        ),
        (
            "d:@\"debug\";values<uint8[1]><uint16[1]>:[{unknown();->300}]",
            "E201",
        ),
        (
            "d:@\"debug\";<A>:<{x<uint8>}>;<B>:<{x<uint16>}>;values<A[1]><B[1]>:[{x:1;d.print(1);->x:2}]",
            "E203",
        ),
        (
            "d:@\"debug\";values<int8[1]><int16[1]>:[{d.print(1);->1;->2}]",
            "E205",
        ),
        (
            "d:@\"debug\";values<int8[1]><uint8[1]>:[{d.panic(\"stop\");a<int32[1/0]>:[];->1}]",
            "E107",
        ),
        (
            "d:@\"debug\";a:=1;r:&a;values<uint8[1]><uint16[1]>:[{a=2;->300}];x:*r",
            "E302",
        ),
    ] {
        rejects(source, code);
    }
    for source in [
        "d:@\"debug\";values<uint8[1]><uint16[1]>:[{d.panic(\"stop\");->300}]",
        "d:@\"debug\";<A>:<{n<uint8>;missing<string>}>;<B>:<{n<uint16>}>;values<A[1]><B[1]>:[{d.print(1);->n:1}]",
        "d:@\"debug\";values<int8[1]><uint8[1]>:[{d.panic(\"stop\");->-128;->-128}]",
    ] {
        let result = crate::compile(source);
        assert!(result.is_ok(), "{source}: {result:?}");
    }
}
