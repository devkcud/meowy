use super::{accepts, rejects};

#[test]
pub(crate) fn aliases_preserve_origins_through_nested_block_results() {
    accepts("a:1;view:&a;{copy:view;value:*copy};value:*view");
    accepts("a:1;view:{alias:{->&a};->alias};same:view==&a;value:*view");
    accepts("a:{->x:1;->nested:{->y:2}};view:{->{->&a.nested.y}};value:*view");
    accepts("f<int32>:(){a:9;view:{->&a};->*view};value:f()");
}

#[test]
pub(crate) fn complementary_conditions_preserve_every_origin() {
    accepts("f<int32>:(flag<boolean>){a:11;b:22;r:{|flag|->&a;|!flag|->&b};->*r}");
    accepts(
        "f<int32>:(flag<boolean>){a:11;b:22;r:'result {|flag|{'result->&a;'result.leave()};->&b};->*r}",
    );
    rejects(
        "f<int32>:(flag<boolean>){a:11;r:'result {|flag|{b:22;'result->&b;'result.leave()};->&a};->*r}",
        "E303",
    );
    rejects(
        "a:11;flag:=false;r:'outer {b:22;view:{|flag|->&a;|!flag|->&b};->view}",
        "E303",
    );
    rejects(
        "a:11;flag:=false;r:'outer {b:22;view:{|flag|->&b;|!flag|->&a};->view}",
        "E303",
    );
}

#[test]
pub(crate) fn separate_function_frames_keep_local_borrows_independent() {
    accepts("f<int32>:(){a:1;r:{->&a};->*r};g<int32>:(){a:2;r:{->&a};->*r};x:f()+g()");
    rejects(
        "f<int32>:(){a:1;r:{->&a};->*r};g:(){a:2;r:{->&a};->r}",
        "E303",
    );
}
