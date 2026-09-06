use super::{accepts, rejects};

#[test]
pub(crate) fn emitted_slot_ownership_outlives_alias_names_but_not_publication() {
    for source in [
        "r:{->n:=1;p:&n;v:*p}",
        "r:'out{p:{'out->n:=1;->&n};->seen:*p}",
        "r:'out{p:{'out->row:={->n:=1};->&row.n};->seen:*p}",
        "r:'out{p:{'out->items:=[1,2];->&items[2]};->seen:*p}",
        "d:@\"debug\";'done{r:'out{p:{'out->lost:=1;->&lost};d.print(*p);'done.leave()}}",
        "f<null>:(flag<boolean>){r:'out{|flag|{'out->n:=1;p:&n;v:*p};|!flag|{'out->n:=2;p:&n;v:*p}}}",
    ] {
        accepts(source);
    }
    for source in [
        "r:{->n:=1;->view:&n}",
        "p:{->n:=1;->&n}",
        "p:'out{r:{->n:=1;'out->&n}}",
        "f<{n<int32>:=;view<&int32>}>:(){->n:=1;->view:&n}",
    ] {
        rejects(source, "E303");
    }
}

#[test]
pub(crate) fn slot_borrow_representation_matches_the_lexical_view() {
    accepts("f<null>:(flag<boolean>){r:'out{|flag|{'out->n:=1;p:&n;v:*p}}}");
    accepts("f<null>:(flag<boolean>){r:'out{|flag|{'out->row:={->n:=1};p:&row.n;v:*p}}}");
    accepts("r:{->n<int32><null>:=1;p:&n;v:*p}");
    rejects(
        "f<null>:(flag<boolean>){r:'out{|flag|{'out->n<int32><null>:=1;p:&n;v:*p};|!flag|{'out->n:=\"x\"}}}",
        "B001",
    );
    accepts("r:{->n:1;p:&n}");
    rejects("r:{->n:=1;p:&!n}", "B001");
}

#[test]
pub(crate) fn immutable_slot_borrows_share_target_lifetime_and_representation_gates() {
    for source in [
        "r:'out{p:{'out->n:1;->&n};->seen:*p}",
        "d:@\"debug\";'done{r:'out{p:{'out->lost:1;->&lost};d.print(*p);'done.leave()}}",
        "f<null>:(flag<boolean>){r:'out{|flag|{'out->n:1;p:&n;v:*p}}}",
        "r:{->n<int32><null>:null;p:&n;v:*p}",
    ] {
        accepts(source);
    }
    for source in ["r:{->n:1;->view:&n}", "p:{->n:1;->&n}"] {
        rejects(source, "E303");
    }
    rejects(
        "f<null>:(flag<boolean>){r:'out{|flag|{'out->n<int32><null>:1;p:&n};|!flag|{'out->n:\"x\"}}}",
        "B001",
    );
}
