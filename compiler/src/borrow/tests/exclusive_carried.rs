use super::{accepts, rejects};

#[test]
pub(crate) fn exclusive_carried_scalar_mutation_ends_before_restart() {
    accepts(
        "<R>:<{n<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;*p=8;first=false;'loop.restart()}}};v:r.n",
    );
    accepts(
        "<R>:<{n<uint8>:=;b<boolean>:=;f<float32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=1;p:&!n;*p=255;'out->b:=true;q:&!b;*q=false;'out->f:=1.5;s:&!f;*s=2.5;first=false;'loop.restart()}}}",
    );
}

#[test]
pub(crate) fn exclusive_carried_moves_and_children_preserve_authority() {
    accepts(
        "<R>:<{n<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;s:&*p;v:*s;q:&!*p;*q=8;*p=9;moved:p;*moved=10;first=false;'loop.restart()}}}",
    );
    rejects(
        "<R>:<{n<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;q:p;v:*p;w:*q;first=false;'loop.restart()}}}",
        "E301",
    );
}

#[test]
pub(crate) fn exclusive_carried_owner_and_child_conflicts_remain_rejected() {
    for body in [
        "p:&!n;n=8;v:*p",
        "p:&!n;q:&*p;*p=8;v:*q",
        "p:&!n;q:&!n;v:*p;w:*q",
    ] {
        rejects(
            &format!(
                "<R>:<{{n<int32>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->n:=7;{body};first=false;'loop.restart()}}}}}}"
            ),
            "E302",
        );
    }
    accepts(
        "<R>:<{n<int32>:=;other<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=7;'out->other:=1;p:&!n;other=2;v:*p;n=8;first=false;'loop.restart()}}}",
    );
}

#[test]
pub(crate) fn exclusive_carried_loans_and_shared_descendants_cannot_cross_backedges() {
    for view in ["p:&!n;", "p:&!n;q:&*p;"] {
        let name = if view.contains("q:") { "q" } else { "p" };
        rejects(
            &format!(
                "<R>:<{{n<int32>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->n:=7;{view}i:=0;'again{{v:*{name};i=i+1;|i<2|'again.restart()}};first=false;'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    for value in ["&*p", "keep(p)"] {
        rejects(
            &format!(
                "<R>:<{{n<int32>:=}}>;keep<&int32>:(p<&!int32>){{->&*p}};x:1;s:=&x;first:=true;r<R>:'out{{'loop{{|first|{{'out->n:=7;p:&!n;s={value};first=false;'loop.restart()}};v:*s}}}}"
            ),
            "B001",
        );
    }
}

#[test]
pub(crate) fn exclusive_carried_owner_resets_and_leave_keep_completed_effects() {
    accepts(
        "<R>:<{n<int32>:=}>;i:=0;r<R>:'out{first:=true;'loop{|first|{'out->n:=i;p:&!n;*p=*p+1;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
    );
    accepts(
        "<R>:<{n<int32>:=}>;run<null>:(stop<boolean>){first:=true;r<R>:'out{'loop{|first|{'out->n:=3;p:&!n;*p={|stop|'out.leave();->9};first=false;'loop.restart()}}}}",
    );
}

#[test]
pub(crate) fn exclusive_carried_indirect_effects_forget_boolean_proofs() {
    for body in ["first=false;*p=8", "first=false;set(p)"] {
        rejects(
            &format!(
                "<R>:<{{n<int32>:=}}>;set<null>:(p<&!int32>){{*p=8}};first:=true;r<R>:'out{{'loop{{|first|{{'out->n:=7;p:&!n;{body};'loop.restart()}}}}}}"
            ),
            "B001",
        );
    }
    accepts(
        "<R>:<{n<int32>:=}>;set<null>:(p<&!int32>){*p=8};first:=true;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;set(p);first=false;'loop.restart()}}}",
    );
}

#[test]
pub(crate) fn exclusive_carried_proof_does_not_enable_unrelated_restart_borrows() {
    rejects(
        "<R>:<{n<int32>}>;run<null>:(){first:=true;r<R>:'out{'loop{|first|{'out->n:7;first=false;'loop.restart()}}}};x:=1;'loop{p:&!x;v:*p;'loop.restart()}",
        "B001",
    );
    rejects(
        "<R>:<{n<int32>:=}>;run<null>:(p<&!int32>){first:=true;r<R>:'out{'loop{|first|{'out->n:=7;first=false;'loop.restart()}}};v:*p}",
        "B001",
    );
}
