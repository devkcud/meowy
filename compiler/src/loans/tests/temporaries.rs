use super::{accepts, rejects};

#[test]
pub(crate) fn temporary_argument_effects_keep_existing_owner_loans_live() {
    accepts("read<int32>:(p<&int32>,q<&int32>){->*p+*q};owner:=1;v:read(&owner,&2);owner=3");
    rejects(
        "read<int32>:(p<&int32>,q<&int32>){->*p+*q};owner:=1;v:read(&owner,&{owner=3;->2})",
        "E302",
    );
    rejects(
        "first<&int32>:(p<&int32>,q<&string>){->p};owner:1;view:first(&owner,&\"tmp\");v:*view",
        "E303",
    );
    accepts("read<int32>:(p<&int32>,q<int32>){->*p+q};'out{v:read(&1,{'out.leave()})}");
}

#[test]
pub(crate) fn temporary_statement_wrappers_preserve_local_scope_and_loop_liveness() {
    accepts("owner:=1;copy:*(&2);view:&owner;v:*view;owner=3");
    rejects("owner:=1;copy:*(&2);view:&owner;owner=3;v:*view", "E302");
    accepts("i:=0;'again{copy:*(&{i=i+1;->i});|i<2|'again.restart()}");
    rejects("p:&1;v:*p", "E303");
    rejects("p:*({->&1})", "E303");
}
