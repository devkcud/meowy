use super::{accepts, rejects};

#[test]
pub(crate) fn temporary_contents_preserve_eager_value_reads_and_lazy_pointees() {
    accepts("owner:=1;inner:&owner;outer:&inner;owner=2;copied:*(&outer);same:copied==outer");
    accepts("expired:&1;pointer:*(&(&expired));same:pointer==&expired");
    rejects("owner:=1;inner:&owner;owner=2;copied:*(&inner)", "E302");
    rejects(
        "owner:=1;holder:{->view:&owner;->n:2};owner=3;v:*(&({->holder}.n))",
        "E302",
    );
    rejects(
        "owner:=1;inner:&owner;outer:&inner;owner=2;copied:*(&outer);v:**copied",
        "E302",
    );
}

#[test]
pub(crate) fn temporary_reference_copies_drop_only_their_own_cell_lifetime() {
    accepts("owner:1;copied:*(&(&owner));v:*copied");
    accepts("owner:1;copied:*(&{->view:&owner});v:*(copied.view)");
    rejects("owner:1;cell:&(&owner);v:**cell", "E303");
    rejects("copied:*(&(&1));v:*copied", "E303");
    rejects(
        "<R>:<&int32>;load<R>:(p<&R>){->*p};owner:1;copied:load(&(&owner));v:*copied",
        "E303",
    );
}
