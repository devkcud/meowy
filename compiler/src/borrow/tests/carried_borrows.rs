use super::{accepts, rejects};

#[test]
pub(crate) fn carried_borrows_acquire_initialized_scalar_storage() {
    accepts(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;p:&n;q:&n;same:p==q;v:*p;first=false};i=i+1;|i<2|'loop.restart()}}",
    );
    accepts(
        "<R>:<{n<uint8>;ok<boolean>;name<string>;ratio<float32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:255;p:&n;v:*p;'out->ok:true;q:&ok;b:*q;'out->name:\"hello\";s:&name;t:*s;'out->ratio:1.5;f:&ratio;x:*f;first=false};i=i+1;|i<2|'loop.restart()}}",
    );
}

#[test]
pub(crate) fn carried_borrows_survive_inner_restarts_and_alias_scope_exit() {
    accepts(
        "<R>:<{n<int32>}>;x:1;p:=&x;old:p;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;p=&n;first=false};v:*p;w:*old;i=i+1;|i<3|'loop.restart()};v:*p};p=&x;v:*p",
    );
}

#[test]
pub(crate) fn carried_borrows_expire_with_their_owner() {
    rejects(
        "<R>:<{n<int32>}>;x:1;p:=&x;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;p=&n;first=false};i=i+1;|i<2|'loop.restart()}};v:*p",
        "E303",
    );
    rejects(
        "<R>:<{n<int32>}>;x:1;p:=&x;i:=0;r<R>:'out{v:*p;first:=true;'loop{|first|{'out->n:i;p=&n;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
        "E303",
    );
}

#[test]
pub(crate) fn carried_borrows_can_be_replaced_before_use_after_owner_reset() {
    accepts(
        "<R>:<{n<int32>}>;x:1;p:=&x;i:=0;r<R>:'out{p=&x;first:=true;'loop{|first|{'out->n:i;p=&n;first=false;'loop.restart()};v:*p};i=i+1;|i<2|'out.restart()};p=&x;v:*p",
    );
}

#[test]
pub(crate) fn carried_borrows_preserve_last_use_and_write_conflicts() {
    accepts(
        "<R>:<{n<int32>:=}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:=7;p:&n;v:*p;n=8;first=false};i=i+1;|i<2|'loop.restart()}}",
    );
    rejects(
        "<R>:<{n<int32>:=}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:=7;p:&n;n=8;v:*p;first=false};i=i+1;|i<2|'loop.restart()}}",
        "E302",
    );
}

#[test]
pub(crate) fn carried_borrows_keep_call_bounds_and_leave_lifetimes() {
    let code = "<R>:<{n<int32>}>;keep<&int32>:(value<&int32>){->value};x:1;p:=&x;first:=true;r<R>:'out{'loop{|first|{'out->n:7;p=keep(&n);first=false;'loop.restart()};v:*p;'out.leave()}};";
    accepts(&format!("{code}p=&x;v:*p"));
    rejects(&format!("{code}v:*p"), "E303");
}
