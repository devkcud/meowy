use super::{accepts, rejects};

#[test]
pub(crate) fn reference_assignment_expiry_tracks_the_current_value_only() {
    for source in [
        "a:1;p:=&1;p=&a;value:*p",
        "a:1;p:=&a;{b:2;p=&b};p=&a;value:*p",
        "first<&int32>:(p<&int32>,q<&string>){->p};a:1;b:2;p:=first(&a,&\"short\");p=&b;value:*p",
        "a:1;b:2;p:=&a;p={p=&b;->&a};value:*p",
        "d:@\"debug\";a:1;p:=&a;p={d.panic(\"stop\")}",
    ] {
        accepts(source);
    }
    for source in [
        "a:1;p:=&a;{b:2;p=&b};value:*p",
        "a:1;p:=&a;p=&1;value:*p",
        "first<&int32>:(p<&int32>,q<&string>){->p};a:1;p:=&a;p=first(&a,&\"short\");value:*p",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub(crate) fn reference_assignments_preserve_control_and_unread_expiry() {
    accepts("a:1;p:=&a;'loop{local:2;p=&local;'loop.restart()}");
    for source in [
        "a:1;b:2;p:=&a;'out{p=&b;'out.leave()}",
        "a:1;b:2;p:=&a;p=&b;'out{'out.leave()}",
        "a:1;b:2;p:=&a;|true|p=&b",
        "a:1;b:2;p:=&a;|false|p=&b",
        "f<null>:(flag<boolean>){a:1;b:2;p:=&a;skip:flag&&{p=&b;->true}}",
        "f<null>:(flag<boolean>){a:1;b:2;p:=&a;skip:flag||{p=&b;->false}}",
        "d:@\"debug\";a:1;b:2;p:=&a;d.print(true&&{p=&b;->true})",
        "a:1;b:2;p:=&a;items:[1];value:items[{skip:false||{p=&b;->false};->1}]",
        "a:1;|true|{p:=&a;value:*p}",
        "a:1;p:=&a;count:=0;'loop{value:*p;count=count+1;|count<2|'loop.restart()}",
        "unused<null>:()'loop{'loop.restart()};a:1;b:2;p:=&a;p=&b;value:*p",
    ] {
        accepts(source);
    }
}

#[test]
pub(crate) fn reference_bindings_keep_fixed_types_and_carrier_mutation_boundaries() {
    accepts("a:1;p<&int32><null>:=&a");
    accepts("a:1;p:={->view:&a}");
    for (source, code) in [
        ("a:1;p:=&a;p=&\"x\"", "E207"),
        (
            "a:1;result:{->view:=&a;view=&a;n:=2;'loop{n=n-1;|n>0|'loop.restart()}}",
            "B001",
        ),
        ("a:{->n:=1};p:=&a;p.n=2", "B001"),
        ("a:[1,2];p:=&a;p[1]=3", "B001"),
    ] {
        rejects(source, code);
    }
}
