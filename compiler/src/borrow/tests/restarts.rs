use super::{accepts, rejects};

#[test]
pub(crate) fn restart_headers_reach_all_carried_sources_before_validation() {
    for source in [
        "a:1;b:2;p:=&a;count:=0;'loop{p=&b;count=count+1;|count<2|'loop.restart()};value:*p",
        "a:1;b:2;p:=&a;q:=&a;r:=&a;count:=0;'loop{p=q;q=r;r=&b;count=count+1;|count<4|'loop.restart()};value:*p",
        "a:1;b:2;p:=&a;outer:=0;'outer{inner:=0;'inner{p=&b;inner=inner+1;|inner<2|'inner.restart()};outer=outer+1;|outer<2|'outer.restart()};value:*p",
    ] {
        accepts(source);
    }
    rejects(
        "a:1;b:=2;p:=&a;q:=&a;r:=&a;count:=0;'loop{p=q;q=r;r=&b;count=count+1;|count<4|'loop.restart()};b=3;value:*p",
        "E302",
    );
    rejects(
        "a:=1;b:2;p:=&b;later:=false;count:=0;'loop{|later|{a=3;value:*p};|!later|p=&a;later=true;count=count+1;|count<2|'loop.restart()}",
        "E302",
    );
}

#[test]
pub(crate) fn restart_domains_require_surviving_direct_sources_and_bounds() {
    for source in [
        "a:1;p:=&a;'loop{local:2;p=&local;'loop.restart()}",
        "a:1;p:=&a;'loop{->n:2;p=&n;'loop.restart()}",
        "a:1;p:=&a;'loop{p=&1;'loop.restart()}",
        "first<&int32>:(p<&int32>,other<&string>){->p};a:1;p:=&a;'loop{local:\"short\";p=first(&a,&local);'loop.restart()}",
    ] {
        rejects(source, "B001");
    }
    accepts(
        "a:1;p:=&a;count:=0;'loop{local:2;p=&local;value:*p;p=&a;count=count+1;|count<2|'loop.restart()};value:*p",
    );
}

#[test]
pub(crate) fn restarted_copies_keep_initial_storage_and_scoped_exits() {
    for source in [
        "a:=1;b:=2;p:=&a;old:p;count:=0;'loop{p=&b;count=count+1;|count<2|'loop.restart()};value:*old;a=3;other:*p",
        "a:1;b:2;c:3;p:=&a;count:=0;'loop{count=count+1;p={p=&b;|count<2|'loop.restart();->&c}};value:*p",
        "a:1;b:2;p:=&a;count:=0;'loop{p=&b;count=count+1;|count<2|'loop.restart();'loop.leave()};value:*p",
    ] {
        accepts(source);
    }
    rejects(
        "a:=1;b:2;p:=&a;old:p;count:=0;'loop{p=&b;count=count+1;|count<2|'loop.restart()};a=3;value:*old",
        "E302",
    );
}

#[test]
pub(crate) fn restart_replay_has_a_bounded_public_source_fixed_point() {
    let mut source = "a:1;b:2;".to_owned();
    for index in 0..80 {
        source.push_str(&format!("p{index}:=&a;"));
    }
    source.push_str("'loop{");
    for index in 0..79 {
        source.push_str(&format!("p{index}=p{};", index + 1));
    }
    source.push_str("p79=&b;'loop.restart()}");
    let errors = crate::compile(&source).unwrap_err();
    assert_eq!(errors[0].code, "B001", "{errors:?}");
    assert!(errors[0].message.contains("budget"));
}
