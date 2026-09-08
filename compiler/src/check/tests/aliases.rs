use super::{accepts, rejects};

#[test]
pub(crate) fn immutable_emitted_aliases_keep_constants_lengths_and_write_protection() {
    for source in [
        "r:{->n:1;p:&n;v:*p}",
        "r:{->flag:false;v:flag&&(1/0==0)}",
        "r:{->items<int32[3]>:[1];n:items.size();v<int32[n]>:[];p:&items[1]}",
        "r:{->row:{->n:=1};p:&row.n;v:*p}",
        "a:1;r:{->view:&a;p:&view}",
    ] {
        accepts(source);
    }
    for (source, code) in [
        ("r:{->byte<uint8>:255;v:byte+1}", "E107"),
        ("r:{->items<int32[3]>:[1];p:&items[2]}", "E101"),
        ("r:{->n:1;n=2}", "E305"),
        ("r:{->row:{->n:=1};row.n=2}", "E305"),
        ("r:{->items:[1,2];items[1]=3}", "E305"),
    ] {
        rejects(source, code);
    }
}

#[test]
pub(crate) fn mutable_emitted_names_support_slot_writes_and_mixed_paths() {
    for source in [
        "r:{->n:=1;n=2}",
        "r:{->row:={->n:=1;->items:=[1,2]};row.n=2;row.items[1]=3}",
        "r:'out{'inner{'out->n:=1;n=2;'out.leave()}}",
        "i:=0;r:'again{->n:=i;n=n+1;i=i+1;|i<2|'again.restart()}",
        "f<null>:(flag<boolean>){r:{|flag|{->n:=1;n=2}}}",
        "f<null>:(flag<boolean>){r:{|flag|{->item:=1;item=2};|!flag|{->item:=\"a\";item=\"b\"}}}",
        "r:{->tag<int32><null>:=null;tag=1}",
        "r:{->items<int32[3]>:=[1];items=[2,3];items[2]=4}",
        "f<null>:(flag<boolean>){r:{|flag|{->items<int32[2]>:=[1,2];items[1]=3}}}",
    ] {
        accepts(source);
    }
}

#[test]
pub(crate) fn alias_discard_paths_preserve_lexical_storage_and_boundaries() {
    for source in [
        "d:@\"debug\";'out{r:{->lost:=1;lost=2;d.print(lost);'out.leave()}}",
        "d:@\"debug\";'out{r:{->items:=[1,2];items[1]=3;d.print(items[1]);'out.leave()}}",
        "r:{->n:=1;copy:=n;copy=2;n=3}",
    ] {
        accepts(source);
    }
    for (source, code) in [
        ("r:{->n:=1;n=\"x\"}", "E207"),
        ("r:{->n:=1;->n:=2}", "E205"),
        ("a:1;r:{->view:=&a;view=&a;'loop{'loop.restart()}}", "B001"),
    ] {
        rejects(source, code);
    }
}

#[test]
pub(crate) fn result_alias_metadata_stays_within_shared_work_bounds() {
    let fields = (0..2048)
        .map(|id| format!("->n{id}:={id};"))
        .collect::<String>();
    let errors = crate::compile(&format!("r:{{{fields}}}")).unwrap_err();
    assert_eq!(errors[0].code, "B001");
    assert!(errors[0].message.contains("budget"));
}

#[test]
pub(crate) fn exclusive_alias_backing_rejects_widening_without_restricting_shared_views() {
    accepts("f:(flag<boolean>)'out{|flag|{'out->n:=1;p:&n;v:*p};|!flag|'out->n:=true}");
    let source = "f:(flag<boolean>)'out{|flag|{'out->n:=1;p:&!n;v:*p};|!flag|'out->n:=true}";
    let errors = crate::compile(source).unwrap_err();
    assert_eq!(errors[0].code, "B001");
    assert!(errors[0].message.contains("identical backing type"));
    assert_eq!(&source[errors[0].span.start..errors[0].span.end], "&!n");
}

#[test]
pub(crate) fn exclusive_projection_requires_exact_backing_beyond_the_selected_field() {
    accepts(
        "f:(flag<boolean>)'out{|flag|{'out->row:={->n:=1;->tag:=true};p:&row.n;v:*p};|!flag|'out->row:={->n:=2;->tag:=3}}",
    );
    let source = "f:(flag<boolean>)'out{|flag|{'out->row:={->n:=1;->tag:=true};p:&!row.n;v:*p};|!flag|'out->row:={->n:=2;->tag:=3}}";
    let errors = crate::compile(source).unwrap_err();
    assert_eq!(errors[0].code, "B001");
    assert!(errors[0].message.contains("identical backing type"));
    assert_eq!(&source[errors[0].span.start..errors[0].span.end], "&!row.n");
}
