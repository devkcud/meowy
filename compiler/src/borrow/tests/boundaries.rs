use super::rejects;

#[test]
pub(crate) fn reference_mutation_and_exclusive_contracts_remain_explicit() {
    for source in [
        "a:1;r:={->field:&a}",
        "a:1;flag:=true;r:={|flag|->&a}",
        "a:1;r:=&a",
        "a:=1;r:&!a",
        "f<int32>:(r<&!int32>){->*r}",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub(crate) fn carrier_mutation_and_implicit_formatting_remain_explicit_boundaries() {
    for source in [
        "a:1;r<{view<&int32>}><null>:=null",
        "a:1;flag:=true;r:={|flag|->view:&a}",
        "a:1;r:={->view:&a}",
        "a:1;r:{->&a;->count:3};d:@\"debug\";d.print(r)",
    ] {
        rejects(source, "B001");
    }
}
