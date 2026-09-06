use super::{accepts, rejects};

#[test]
pub(crate) fn named_outer_emissions_preserve_origin_and_leave_target() {
    accepts("a:1;r:'result {'inner {'result->&a;'result.leave()}};v:*r");
    rejects(
        "r:'result {'inner {a:1;'result->&a;'result.leave()}}",
        "E303",
    );
    rejects(
        "a:1;r:'outer {'inner {'outer->&a;'inner.restart()}}",
        "B001",
    );
}

#[test]
pub(crate) fn completing_local_results_never_extend_storage_lifetimes() {
    for source in [
        "value:{a:1;->&a}",
        "make:(){a:1;view:&a;alias:view;->alias}",
        "a:1;value:{a:2;view:{->&a};->view}",
        "a:1;r:'scope {b:2;|true|->&b}",
        "a:1;r:'scope {b:2;flag:true;|flag|->&b}",
        "condition:(x<boolean>){->x};|condition(false)|{x:1;->&x}",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub(crate) fn discarded_reference_emissions_do_not_escape() {
    for source in [
        "'loop {x:1;->&x;'loop.restart()}",
        "debug:@\"debug\";{x:1;->&x;debug.panic(\"stop\")}",
        "'outer {{x:1;->&x;'outer.leave()}}",
        "flag:false;'scope {x:1;|flag|->&x}",
        "'scope {x:1;|false|->&x}",
        "owner:42;again:=true;r:'result {|again|{local:1;'result->&local;again=false;'result.restart()};->&owner};value:*r",
        "owner:42;again:=true;r:'result {|again|{local:1;'result->&local;again=false;'result.restart()};'nested {'result->&owner;'result.leave()}};value:*r",
        "debug:@\"debug\";f<int32>:(flag<boolean>){a:1;r:'result {|flag|{b:2;'result->&b;debug.panic(\"stop\")};->&a};->*r}",
    ] {
        accepts(source);
    }
}

#[test]
pub(crate) fn scoped_transfers_skip_unreachable_reference_uses() {
    accepts("'outer {{'outer.leave()};x:1;->&x}");
    accepts("'outer {{'outer.restart()};x:1;->&x}");
    accepts("debug:@\"debug\";'scope {debug.panic(\"stop\");x:1;->&x}");
    accepts("a:1;r:'scope {|true|{'scope->&a;'scope.leave()};x:2;->&x};value:*r");
}
