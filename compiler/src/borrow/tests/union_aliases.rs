use super::{accepts, rejects};

#[test]
pub(crate) fn union_alias_writes_remap_reference_members_and_clear_old_sources() {
    accepts(
        "x:=1;y:2;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=&x;p=&y};|!flag|{'out->p:=\"x\"}};x=3;|r.p<&int32>|{v:*r.p}",
    );
    accepts(
        "x:=1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=&x;p=null};|!flag|{'out->p:=\"x\"}};x=3;copy:r",
    );
    rejects(
        "x:=1;y:2;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=&x;old:p;p=&y;x=3;copy:old};|!flag|{'out->p:=\"x\"}}",
        "E302",
    );
    rejects(
        "x:1;y:=2;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=&x;p=&y};|!flag|{'out->p:=\"x\"}};y=3;copy:r",
        "E302",
    );
}

#[test]
pub(crate) fn union_alias_branches_and_leave_publish_the_last_member() {
    accepts(
        "x:1;flag:=true;again:=true;r:'out{|flag|{'out->p<&int32><null>:=null;|again|p=&x};|!flag|{'out->p:=false}};copy:r",
    );
    accepts(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;p={p=&x;'out.leave();->null}};|!flag|{'out->p:=\"x\"}};|r.p<&int32>|{v:*r.p}",
    );
    rejects(
        "x:=1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;p={p=&x;'out.leave();->null}};|!flag|{'out->p:=\"x\"}};x=2;copy:r",
        "E302",
    );
}

#[test]
pub(crate) fn union_alias_current_predicates_keep_nested_reference_activity() {
    let types = "<A>:<{p<&int32><null>}>;<B>:<{n<int32>}>;";
    accepts(&format!(
        "{types}x:1;flag:=true;r:'out{{|flag|{{'out->c<A><B>:={{->n:0}};c={{->p:&x}};|c<A>|{{|c.p<&int32>|{{v:*c.p}}}};c={{->n:2}}}};|!flag|{{'out->c:=null}}}};copy:r"
    ));
    rejects(
        &format!(
            "{types}x:=1;flag:=true;r:'out{{|flag|{{'out->c<A><B>:={{->n:0}};c={{->p:&x}};x=2;|c<A>|{{|c.p<&int32>|{{v:*c.p}}}}}};|!flag|{{'out->c:=null}}}}"
        ),
        "E302",
    );
}

#[test]
pub(crate) fn union_alias_lifetimes_survive_only_with_the_final_sources() {
    accepts(
        "outer:1;flag:=true;result:{local:2;r:'out{|flag|{'out->p<&int32><null>:=&local;p=&outer};|!flag|{'out->p:=\"x\"}};->r};copy:result",
    );
    rejects(
        "outer:1;flag:=true;result:{local:2;r:'out{|flag|{'out->p<&int32><null>:=&outer;p=&local};|!flag|{'out->p:=\"x\"}};->r}",
        "E303",
    );
    rejects(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;local:2;p=&local};|!flag|{'out->p:=false}}",
        "E303",
    );
}

#[test]
pub(crate) fn union_alias_restarts_and_discarded_effects_keep_versions() {
    accepts(
        "x:1;flag:=true;n:=0;r:'loop{|flag|{'loop->p<&int32><null>:=null;p=&x};|!flag|{'loop->p:=\"x\"}};'other{n=n+1;|n<2|'other.restart()};copy:r",
    );
    accepts(
        "x:1;flag:=true;n:=0;r:'loop{|flag|{'loop->p<&int32><null>:=null;p=&x};|!flag|{'loop->p:=\"x\"};n=n+1;|n<2|'loop.restart()};copy:r",
    );
    accepts("x:1;'out{r:{->p<&int32><null>:=null;p=&x;'out.leave()}}");
    accepts(
        "<R>:<{p<&int32><null><string>:=}>;x:1;n:=2;r<R>:{->p<&int32><null>:=null;'inner{p=&x;n=n-1;|n>0|'inner.restart()}}",
    );
}

#[test]
pub(crate) fn union_alias_addresses_and_lexical_type_boundaries_stay_explicit() {
    rejects(
        "<A>:<{p<&int32>:=}>;<B>:<{n<int32>}>;x:1;flag:=true;r:'out{|flag|{'out->c<A><B>:={->p:=&x};|c<A>|c.p=&x};|!flag|{'out->c:=null}}",
        "B001",
    );
    rejects(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;cell:&p;p=&x};|!flag|{'out->p:=\"x\"}}",
        "B001",
    );
    rejects(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:=null;p=\"x\"};|!flag|{'out->p:=\"x\"}}",
        "E207",
    );
    rejects(
        "x:1;flag:=true;r:'out{|flag|{'out->p<&int32><null>:null;p=&x};|!flag|{'out->p:\"x\"}}",
        "E305",
    );
}
