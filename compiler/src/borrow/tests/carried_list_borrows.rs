use super::{accepts, rejects};

#[test]
pub(crate) fn carried_list_borrows_retain_whole_list_storage_across_restarts() {
    accepts(
        "<R>:<{items<int32[3]>}>;seed<int32[3]>:[0];p:=&seed;old:p;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->items:[7,8];p=&items;first=false};v:p[2];w:old[1];i=i+1;|i<3|'loop.restart()};v:p[1]};p=&seed;v:p[1]",
    );
}

#[test]
pub(crate) fn carried_list_borrows_retain_elements_and_nested_field_reborrows() {
    accepts(
        "<Row>:<{n<int32>;name<string>}>;<R>:<{items<Row[2][2]>}>;x:0;p:=&x;q:=&x;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->items:[[{->n:7;->name:\"ready\"}]];whole:&items;inner:&(whole[1]);p=&(items[1][1].n);q=&(inner[1].n);first=false};v:*p;w:*q;same:p==q;i=i+1;|i<3|'loop.restart()};v:*q};p=&x;q=&x",
    );
    accepts(
        "<Row>:<{items<int32[2]>;other<int32>}>;<R>:<{row<Row>}>;x:0;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:[7];->other:8};whole:&row;p=&(whole.items[1]);first=false;'loop.restart()};v:*p}};p=&x",
    );
}

#[test]
pub(crate) fn carried_list_borrows_expire_on_owner_completion_and_reset() {
    let prefix = "<R>:<{items<int32[2]>}>;seed<int32[2]>:[0];p:=&seed;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=&items;first=false;'loop.restart()};v:p[1]}};";
    rejects(&format!("{prefix}v:p[1]"), "E303");
    accepts(&format!("{prefix}p=&seed;v:p[1]"));
    rejects(
        "<R>:<{items<int32[2]>}>;x:0;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=&(items[1]);first=false;'loop.restart()};v:*p;'out.leave()}};v:*p",
        "E303",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;x:0;p:=&x;i:=0;r<R>:'out{v:*p;first:=true;'loop{|first|{'out->items:[i];p=&(items[1]);first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
        "E303",
    );
    accepts(
        "<R>:<{items<int32[2]>}>;x:0;p:=&x;i:=0;r<R>:'out{p=&x;first:=true;'loop{|first|{'out->items:[i];p=&(items[1]);first=false;'loop.restart()};v:*p};i=i+1;|i<2|'out.restart()};p=&x",
    );
}

#[test]
pub(crate) fn carried_list_borrows_preserve_conflicts_copies_and_final_use() {
    let prefix = "<Row>:<{items<int32[2]>:=;other<int32>:=}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:=[7];->other:=1};";
    let suffix = "first=false;'loop.restart()}}}";
    for body in [
        "p:row.&items;row.items=[8];v:p[1]",
        "p:&(row.items[1]);row.items=[8];v:*p",
        "p:&row;row.other=2;v:p.other",
    ] {
        rejects(&format!("{prefix}{body};{suffix}"), "E302");
    }
    accepts(&format!(
        "{prefix}p:&(row.items[1]);row.other=2;v:*p;row.items=[8];q:row.&items;w:q[1];row.items=[9];{suffix}"
    ));
    accepts(&format!(
        "{prefix}old:row.items;p:&(old[1]);copy:&row.items[1];row.items=[9];v:*p;w:copy;{suffix}"
    ));
}

#[test]
pub(crate) fn carried_list_borrows_check_index_effects_before_completed_acquisition() {
    accepts(
        "<R>:<{items<int32[2]>:=}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];p:&(items[{|stop|{items=[9];'out.leave()};->1}]);v:*p;first=false;'loop.restart()}}}}",
    );
    rejects(
        "<R>:<{items<int32[2]>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->items:=[7];p:&(items[{items=[9];->1}]);v:*p;first=false;'loop.restart()}}}",
        "E302",
    );
}

#[test]
pub(crate) fn carried_list_borrows_keep_call_bounds_and_old_reference_copies() {
    accepts(
        "<R>:<{items<int32[2]>}>;keep<&int32>:(p<&int32[2]>){->&(p[1])};x:0;p:=&x;old:p;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=keep(&items);first=false;'loop.restart()};v:*p;w:*old;'out.leave()}};p=&x;v:*p",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;keep<&int32>:(p<&int32[2]>){->&(p[1])};x:0;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=keep(&items);first=false;'loop.restart()}}};v:*p",
        "E303",
    );
}

#[test]
pub(crate) fn carried_list_borrows_keep_empty_initialization_and_static_bounds() {
    accepts(
        "<R>:<{items<int32[0]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[];p:&items;n:p.size();first=false;'loop.restart()}}}",
    );
    rejects(
        "<R>:<{items<int32[2]>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->items:[7];p:&items;v:p[1];first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{items<int32[3]>}>;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p:&(items[2]);first=false;'loop.restart()}}}",
        "E101",
    );
}

#[test]
pub(crate) fn carried_list_borrows_preserve_exclusive_frontiers_and_collection_gates() {
    accepts(
        "<R>:<{items<int32[2]>;n<int32>:=}>;seed<int32[2]>:[0];p:=&seed;first:=true;r<R>:'out{'loop{|first|{'out->items:[7];p=&items;'out->n:=1;q:&!n;*q=2;first=false;'loop.restart()};v:p[1]}};p=&seed",
    );
    for body in ["p:&!(items[1])", "items[1]=9"] {
        rejects(
            &format!(
                "<R>:<{{items<int32[2]>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->items:=[7];{body};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    rejects(
        "<Row>:<{items<int32[2]>;n<int32>:=}>;<R>:<{row<Row>}>;first:=true;r<R>:'out{'loop{|first|{'out->row:{->items:[7];->n:=1};p:row.&!n;*p=2;first=false;'loop.restart()}}}",
        "B001",
    );
    rejects(
        "<R>:<{items<int32[2]>;n<int32>:=}>;keep<&int32[2]>:(p<&int32[2]>,extra<string>){->p};seed:[0,1];p:=keep(&seed,\"\");first:=true;r<R>:'out{'loop{|first|{'out->items:[7];'out->n:=1;q:&!n;*q=2;first=false;'loop.restart()};v:p[1]}}",
        "B001",
    );
}
