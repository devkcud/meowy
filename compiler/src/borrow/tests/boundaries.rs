use super::{accepts, rejects};

#[test]
pub(crate) fn reference_carrier_bindings_and_direct_exclusive_contracts_are_supported() {
    accepts("a:=1;r:&!a");
    accepts("f<int32>:(r<&!int32>){->*r}");
    for source in ["a:1;r:={->field:&a}", "a:1;flag:=true;r:={|flag|->&a}"] {
        accepts(source);
    }
}

#[test]
pub(crate) fn fixed_carrier_bindings_do_not_enable_implicit_reference_formatting() {
    for source in [
        "a:1;r<{view<&int32>}><null>:=null",
        "a:1;flag:=true;r:={|flag|->view:&a}",
        "a:1;r:={->view:&a}",
    ] {
        accepts(source);
    }
    rejects("a:1;r:{->&a;->count:3};d:@\"debug\";d.print(r)", "B001");
}
