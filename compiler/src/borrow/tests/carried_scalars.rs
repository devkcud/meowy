use super::{accepts, rejects};

#[test]
pub(crate) fn carried_scalars_initialize_once_and_remain_available_after_restart() {
    accepts(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=false};i=i+1;|i<3|'loop.restart()}};v:r.n",
    );
    accepts(
        "<R>:<{n<int32>}>;first:=true;r<R>:'out{'loop{|first|{'out->n:7;first=false;'loop.restart()}}};v:r.n",
    );
    accepts(
        "first:=true;i:=0;r<int32>:'out{'loop{|first|{'out->7;first=false};i=i+1;|i<2|'loop.restart()}};v:r+1",
    );
}

#[test]
pub(crate) fn carried_scalars_track_boolean_updates_and_keep_copied_flags() {
    accepts(
        "<R>:<{n<int32>}>;first:=false;i:=0;r<R>:'out{'loop{|!first|{'out->n:7;first=true};i=i+1;|i<2|'loop.restart()}}",
    );
    accepts(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=first&&false};i=i+1;|i<2|'loop.restart()}}",
    );
    rejects(
        "<R>:<{n<int32>}>;first:=true;old:first;i:=0;r<R>:'out{'loop{|old|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=false};first=true;i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_scalars_reject_uninitialized_completions_and_repeated_emissions() {
    rejects(
        "<R>:<{n<int32>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32>}>;i:=0;r<R>:'out{'loop{'out->n:7;i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;'out->n:8;first=false};i=i+1;|i<2|'loop.restart()}}",
        "E205",
    );
    rejects("<R>:<{n<int32>}>;r<R>:{}", "E204");
}

#[test]
pub(crate) fn carried_scalar_owner_restarts_clear_initialization_state() {
    accepts(
        "<R>:<{n<int32>}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->n:i;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()};v:r.n",
    );
    rejects(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:i;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()};v:r.n",
        "B001",
    );
}

#[test]
pub(crate) fn carried_scalars_keep_leave_and_partial_panic_paths() {
    accepts(
        "<R>:<{n<int32>}>;first:=true;r<R>:'out{'loop{|first|{'out->n:7;first=false;'loop.restart()};'out.leave()}};v:r.n",
    );
    accepts(
        "<R>:<{n<int32>}>;f<null>:(stop<boolean>){d:@\"debug\";first:=true;i:=0;r<R>:'out{'loop{|stop|d.panic(\"stop\");|first|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}}",
    );
}

#[test]
pub(crate) fn carried_scalars_require_pure_boolean_proof_without_replaying_effects() {
    rejects(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=first&&{first=false;->true}};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32>}>;stop<boolean>:(){->false};first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=stop()};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
}

#[test]
pub(crate) fn carried_scalar_widths_strings_and_boolean_fields_keep_declared_types() {
    accepts(
        "<R>:<{n<uint8>;ok<boolean>;name<string>;ratio<float32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:255;'out->ok:true;'out->name:\"hello\";'out->ratio:1.5;first=false};i=i+1;|i<2|'loop.restart()}};v:r.n",
    );
}

#[test]
pub(crate) fn carried_scalar_exclusive_borrows_and_wider_types_remain_gated() {
    rejects(
        "first:=true;i:=0;r:'out{'loop{|first|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32><null>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{p<&int32>}>;x:1;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->p:&x;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32>:=}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;v:*p;first=false};i=i+1;|i<2|'loop.restart()}}",
        "B001",
    );
    accepts(
        "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}};p:&r.n;v:*p",
    );
}
