use super::{accepts, rejects};

#[test]
pub(crate) fn field_writes_preserve_disjoint_loans_and_final_rhs_reads() {
    for source in [
        "r:={->left:=1;->right:=2};p:&(r.left);r.right=3;v:*p",
        "r:={->left:=1;->right:=2};p:&(r.left);r.left=*p+1",
        "r:={->child:={->left:=1;->right:=2}};p:&(r.child.left);r.child.right=3;v:*p",
        "a:=1;b:=2;r:={->flag:=false;->other:=false};|!r.flag|{r.other=true;p:{|r.flag|->&a;|!r.flag|->&b};a=3;v:*p}",
        "d:@\"debug\";r:={->n:=1};p:&(r.n);r.n=d.panic(\"stop\");v:*p",
    ] {
        accepts(source);
    }
    for source in [
        "r:={->left:=1;->right:=2};p:&(r.left);r.left=3;v:*p",
        "r:={->left:=1;->right:=2};p:&r;r.right=3;v:p.left",
        "r:={->child:={->n:=1}};p:&(r.child.n);r.child={->n:=2};v:*p",
        "a:=1;b:=2;r:={->flag:=false;->other:=false};|!r.flag|{r.flag=true;p:{|r.flag|->&a;|!r.flag|->&b};a=3;v:*p}",
    ] {
        rejects(source, "E302");
    }
}
