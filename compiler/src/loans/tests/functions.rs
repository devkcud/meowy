use super::{accepts, rejects};

#[test]
pub(crate) fn returned_function_views_keep_all_input_bounds_until_final_use() {
    accepts(
        "first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:first(&a,&b);x:*r;b=\"new\";a=2",
    );
    rejects(
        "first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:first(&a,&b);b=\"new\";x:*r",
        "E302",
    );
    rejects(
        "identity<&int32>:(a<&int32>){->a};first<&int32>:(a<&int32>,b<&string>){->a};a:=1;b:=\"old\";r:identity(first(&a,&b));b=\"new\";x:*r",
        "E302",
    );
    rejects(
        "head<&int32>:(r<{left<&int32>;right<&string>}>){->r.left};a:=1;b:=\"old\";r:head({->left:&a;->right:&b});b=\"new\";x:*r",
        "E302",
    );
}

#[test]
pub(crate) fn scalar_function_results_and_projections_end_argument_loans() {
    accepts("read<int32>:(a<&int32>){->*a};a:=1;x:read(&a);a=2");
    accepts(
        "packet<{view<&int32>;count<int32>}>:(a<&int32>,b<&string>){->view:a;->count:*a};a:=1;b:=\"old\";r:packet(&a,&b);a=2;b=\"new\";x:r.count",
    );
    accepts(
        "packet<{view<&int32>;count<int32>}>:(a<&int32>,b<&string>){->view:a;->count:*a};a:=1;b:=\"old\";x:packet(&a,&b).count;a=2;b=\"new\"",
    );
}

#[test]
pub(crate) fn call_arguments_stay_live_until_consumption_and_stop_at_exit() {
    rejects(
        "read<int32>:(a<&int32>,b<int32>){->*a+b};a:=1;x:read(&a,{a=2;->3})",
        "E302",
    );
    accepts("read<int32>:(a<&int32>,b<int32>){->*a+b};a:=1;'out {read(&a,{'out.leave()});a=2}");
    accepts(
        "identity<&int32>:(a<&int32>){->a};a:=1;r<&int32><null>:null;|r<&int32>|{view:identity(&a)};a=2",
    );
}

#[test]
pub(crate) fn symbolic_function_inputs_do_not_hide_local_conflicts() {
    accepts("read<int32>:(a<&int32>){local:=1;r:&local;x:*r;local=2;->*a}");
    rejects(
        "read<int32>:(a<&int32>){local:=1;r:&local;local=2;x:*r;->*a}",
        "E302",
    );
}
