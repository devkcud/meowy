use super::Case;
use super::exclusive_references::rejects;

#[test]
pub fn prefix_borrows_select_values_and_grouped_borrows_select_storage() {
    Case::new(
        r#"
d:@"debug"
items<int32[2]>:=[10,20]
copy<int32>:&items[1]
view<&int32>:&(items[1])
d.print(copy);d.print(*view);d.print(view==&(items[1]))
items[1]=30;d.print(copy);d.print(items[1])
object:={->field:=40}
value<int32>:&object.field
field<&int32>:object.&field
group<&int32>:&(object.field)
d.print(value);d.print(field==group);d.print(*field)
object.field=50;d.print(value);d.print(object.field)
holder:{->items:[60,70]}
element<int32>:holder.&items[2]
selected<&int32>:&(holder.items[2])
d.print(element);d.print(*selected)
list:&items
deref<int32>:*list[2]
d.print(deref)
"#,
    )
    .runs(b"10\n10\ntrue\n10\n30\n40\ntrue\n40\n40\n50\n70\n70\n20\n");
    rejects("items:[1];view<&int32>:&items[1]", "E207");
    rejects("object:{->field:1};view<&int32>:&object.field", "E207");
    rejects("items:[1];view:&items;value:*(view[1])", "E222");
}

#[test]
pub fn dotted_unary_operators_apply_to_the_immediately_selected_member() {
    Case::new(
        r#"
d:@"debug"
object:={->inner:={->field:=7;->other:=8};->tail:=9}
copy<int32>:object.&inner.field
view<&int32>:object.inner.&field
parent:object.&inner
same<&int32>:parent.&field
d.print(copy);d.print(view==same);d.print(*same)
left:object.inner.&!field;right:object.inner.&!other
*left=10;*right=11;object.tail=12
d.print(*left);d.print(*right);d.print(object.tail)
value:13
record:{->field:&value}
pointer:&record
plain<int32>:pointer.*field
copied<&int32>:*pointer.field
shared<&int32>:&*pointer.field
grouped<& &int32>:&((*pointer).field)
inner:{->field:&value}
nested:{->inner:&inner}
d.print(plain);d.print(copied==&value);d.print(shared==&value)
d.print(**grouped);d.print(*(pointer.field));d.print(nested.*inner.*field)
"#,
    )
    .runs(b"7\ntrue\n7\n10\n11\n12\n13\ntrue\ntrue\n13\n13\n13\n");
    rejects(
        "value:1;object:{->field:&value};pointer:&object;copy<int32>:*pointer.field",
        "E207",
    );
}

#[test]
pub fn dotted_field_loans_preserve_conflicts_and_mutability() {
    Case::new("object:{->field:=1};p:object.&!field").runs(b"");
    Case::new("object:={->inner:{->field:=1}};p:object.inner.&!field").runs(b"");
    Case::new(
        r#"
d:@"debug"
object:={->left:=1;->right:=2}
shared:object.&left;exclusive:object.&!right
*exclusive=3;d.print(*shared);d.print(*exclusive)
object.left=4;d.print(object.left)
"#,
    )
    .runs(b"1\n3\n4\n");
    for source in [
        "object:={->field:=1};p:object.&field;q:object.&!field;v:*p",
        "object:={->field:=1};p:object.&!field;q:object.&field;v:*p",
        "object:={->field:=1};p:object.&!field;q:object.&!field;v:*p",
        "object:={->field:=1};p:object.&!field;object.field=2;v:*p",
    ] {
        rejects(source, "E302");
    }
    rejects("object:={->field:1};p:object.&!field", "E305");
}

#[test]
pub fn grouped_unary_call_and_index_operands_evaluate_in_order_once() {
    Case::new(
        r#"
d:@"debug"
parent<&int32[2]>:(items<&int32[2]>){d.print("parent");->items}
index<usize>:(){d.print("index");->2}
read<int32>:(view<&int32>){d.print("read");->*view}
<R>:<{field<int32>}>
make<R>:(){d.print("make");->field:9}
<Ref>:<{field<&int32>}>
wrap<Ref>:(view<&int32>){d.print("wrap");->field:view}
items<int32[2]>:=[4,5]
d.print(read(&(parent(&items)[index()])))
d.print(*(parent(&items))[index()])
d.print(wrap(&(items[1])).*field)
d.print(read(make().&field))
d.print(read(&(make().field)))
calls:=0
slot:&!(items[{calls=calls+1;d.print("exclusive index");->1}])
*slot=6;d.print(calls);d.print(*slot);d.print(items[1])
'out{unused:&(parent(&items)[{d.print("leave");'out.leave();->1}]);d.print("unreachable")}
d.print("done")
"#,
    )
    .runs(b"parent\nindex\nread\n5\nparent\nindex\n5\nwrap\n4\nmake\nread\n9\nmake\nread\n9\nexclusive index\n1\n6\n6\nparent\nleave\ndone\n");
}

#[test]
pub fn dotted_and_grouped_borrows_preserve_lifetimes_and_capability_gates() {
    for source in [
        "p:{object:{->field:1};->object.&field}",
        "p:{object:={->field:=1};->object.&!field}",
        "p:({->field:1}).&field;v:*p",
        "make<int32>:(){->1};p:&(make());v:*p",
        "p:&([1,2][1]);v:*p",
    ] {
        rejects(source, "E303");
    }
    for source in [
        "object:={->inner:={->field:=1}};p:object.&!inner",
        "object:={->field:=\"text\"};p:object.&!field",
        "object:={->field:=1};p:&!object.field",
        "items<int32[1]>:=[1];p:&!items[1]",
        "p:({->field:=1}).&!field",
        "value:1;items:[&value]",
    ] {
        rejects(source, "B001");
    }
}

#[test]
pub fn borrowing_exclusive_field_references_keeps_existing_rejection_boundaries() {
    for source in [
        "object:={->field:=1};p:&object.&!field",
        "object:={->field:=1};p:&(object.&!field)",
    ] {
        rejects(source, "B001");
    }
}
