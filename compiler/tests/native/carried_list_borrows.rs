use super::Case;
use super::exclusive_references::rejects;

#[test]
pub(crate) fn carried_list_borrows_retain_whole_storage_and_old_copies() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[3]>}>
seed<int32[3]>:[0]
p:=&seed
old:p
first:=true
i:=0
r<R>:'out{
    'loop{
        |first|{
            'out->items:{d.print("init");->[7,8]}
            p=&items
            d.print(p==&items)
            first=false
        }
        d.print(p.size())
        d.print(p[2])
        i=i+1
        |i<3|'loop.restart()
    }
    d.print(p[1])
}
p=&seed
d.print(old[1])
d.print(r.items[2])
"#,
    )
    .runs(b"init\ntrue\n2\n8\n2\n8\n2\n8\n7\n0\n8\n");
}

#[test]
pub(crate) fn carried_list_borrows_keep_nested_elements_parent_views_and_siblings() {
    Case::new(
        r#"
d:@"debug"
<Item>:<{n<int32>;name<string>}>
<Row>:<{items<Item[2][2]>;other<int32>:=}>
<R>:<{row<Row>}>
x:0
p:=&x
q:=&x
first:=true
i:=0
r<R>:'out{'loop{
    |first|{
        'out->row:{->items:[[{->n:7;->name:"ready"}]];->other:=1}
        whole:row.&items
        inner:&(whole[1])
        p=&(row.items[1][1].n)
        q=&(inner[1].n)
        other:row.&other
        d.print(*other)
        row.other=2
        d.print(p==q)
        d.print(inner[1].name)
        first=false
    }
    d.print(*p)
    d.print(*q)
    i=i+1
    |i<2|'loop.restart()
}}
p=&x
q=&x
d.print(r.row.other)
"#,
    )
    .runs(b"1\ntrue\nready\n7\n7\n7\n7\n2\n");
}

#[test]
pub(crate) fn carried_list_borrows_use_current_length_for_dynamic_bounds() {
    let source = r#"
d:@"debug"
<R>:<{items<int32[3]>:=}>
index<int32>:(at<int32>){d.print("index");->at}
run<null>:(long<boolean>,at<int32>){
    first:=true
    r<R>:'out{'loop{|first|{
        'out->items:=[1]
        |long|items=[7,8]
        p:&(items[index(at)])
        d.print(*p)
        items=[9]
        q:&items
        d.print(q.size())
        first=false
        'loop.restart()
    }}}
    d.print(r.items[1])
}
"#;
    Case::new(&format!("{source}run(false,1);run(true,2)"))
        .runs(b"index\n1\n1\n9\nindex\n8\n1\n9\n");
    let case = Case::new(&format!("{source}run(false,2)"));
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, b"index\n");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .starts_with("panic[P001]: index 2 is outside initialized length 1")
        );
    }
}

#[test]
pub(crate) fn carried_list_borrows_preserve_canceled_index_effects() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2]>:=}>
run<null>:(stop<boolean>){
    first:=true
    r<R>:'out{'loop{|first|{
        'out->items:=[7]
        p:&(items[{d.print("index");|stop|{items=[9];'out.leave()};->1}])
        d.print(*p)
        first=false
        'loop.restart()
    }}}
    d.print(r.items[1])
}
run(false)
run(true)
"#,
    )
    .runs(b"index\n7\n7\nindex\n9\n");
    rejects(
        "<R>:<{items<int32[2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];p:&(items[{items=[9];->1}]);v:*p;first=false;'loop.restart()}}}",
        "E302",
    );
}

#[test]
pub(crate) fn carried_list_borrows_reacquire_after_owner_resets_and_return_through_calls() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2]>}>
keep<&int32>:(p<&int32[2]>){->&(p[1])}
seed<int32[2]>:[0]
x:0
p:=&seed
q:=&x
old:q
i:=0
r<R>:'out{
    p=&seed
    q=&x
    first:=true
    'loop{
        |first|{
            'out->items:[i+7]
            p=&items
            q=keep(&items)
            first=false
            'loop.restart()
        }
        d.print(p[1])
        d.print(*q)
    }
    i=i+1
    |i<2|'out.restart()
}
p=&seed
q=&x
d.print(*old)
d.print(r.items[1])
"#,
    )
    .runs(b"7\n7\n8\n8\n0\n8\n");
}

#[test]
pub(crate) fn carried_list_borrows_keep_empty_lists_copies_and_exclusive_sibling_headers() {
    Case::new(
        r#"
d:@"debug"
<R>:<{items<int32[2]>;empty<int32[0]>;n<int32>:=}>
seed<int32[2]>:[0]
p:=&seed
first:=true
r<R>:'out{'loop{
    |first|{
        'out->items:[7]
        p=&items
        'out->empty:[]
        e:&empty
        d.print(e.size())
        'out->n:=1
        q:&!n
        *q=2
        first=false
        'loop.restart()
    }
    d.print(p[1])
}}
p=&seed
d.print(r.n)
<Row>:<{items<int32[2]>:=}>
<S>:<{row<Row>}>
first=true
s<S>:'out{'loop{|first|{
    'out->row:{->items:=[3]}
    old:row.items
    q:&(old[1])
    copy:&row.items[1]
    row.items=[4]
    d.print(*q)
    d.print(copy)
    first=false
    'loop.restart()
}}}
d.print(s.row.items[1])
"#,
    )
    .runs(b"0\n7\n2\n3\n3\n4\n");
}

#[test]
pub(crate) fn carried_list_borrows_preserve_expiry_conflicts_initialization_and_gates() {
    for source in [
        "<R>:<{items<int32[2]>}>;seed:[0,1];p:=&seed;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=&items;first=false;'loop.restart()}}};v:p[1]",
        "<R>:<{items<int32[2]>}>;x:0;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=&(items[1]);first=false;'loop.restart()};'out.leave()}};v:*p",
        "<R>:<{items<int32[2]>}>;x:0;p:=&x;i:=0;r<R>:'out{v:*p;first:=true;'loop{|first|{'out->items:[i];p=&(items[1]);first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
    ] {
        rejects(source, "E303");
    }
    for body in ["p:&items;items=[8];v:p[1]", "p:&(items[1]);items=[8];v:*p"] {
        rejects(
            &format!(
                "<R>:<{{items<int32[2]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[7];{body};first=false;'loop.restart()}}}}}}"
            ),
            "E302",
        );
    }
    for body in ["p:&!(items[1])", "items[1]=9"] {
        rejects(
            &format!(
                "<R>:<{{items<int32[2]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[7];{body};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    rejects(
        "<Row>:<{items<int32[2]>;n<int32>:=}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:[7];->n:=1};p:row.&!n;first=false;'loop.restart()}}}",
        "B001",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->items:[7];p:&items;v:p[1];first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{items<int32[3]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p:&(items[2]);first=false;'loop.restart()}}}",
        "E101",
    );
    rejects(
        "<R>:<{items<int32[2]>;n<int32>:=}>;keep<&int32[2]>:(p<&int32[2]>,extra<string>){->p};seed:[0,1];p:=keep(&seed,\"\");first:=true;r<R>:'out{'loop{|first|{'out->items:[7];'out->n:=1;q:&!n;*q=2;first=false;'loop.restart()};v:p[1]}}",
        "B001",
    );
}
