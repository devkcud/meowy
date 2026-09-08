use super::Case;

#[test]
pub fn carried_borrows_retain_the_slot_across_inner_restarts() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
x:1
p:=&x
old:p
first:=true
i:=0
r<R>:'out{
    'loop{
        |first|{'out->n:7;p=&n;first=false}
        d.print(*old)
        d.print(*p)
        i=i+1
        |i<3|'loop.restart()
    }
    d.print(*p)
}
d.print(r.n)
p=&x
d.print(*p)
"#,
    )
    .runs(b"1\n7\n1\n7\n1\n7\n7\n7\n1\n");
}

#[test]
pub fn carried_borrows_reacquire_fresh_storage_after_owner_resets() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
x:1
p:=&x
i:=0
r<R>:'out{
    p=&x
    first:=true
    'inner{
        |first|{'out->n:i;p=&n;first=false;'inner.restart()}
        d.print(*p)
    }
    i=i+1
    |i<2|'out.restart()
}
d.print(r.n)
p=&x
d.print(*p)
"#,
    )
    .runs(b"0\n1\n1\n1\n");
}

#[test]
pub fn carried_borrows_preserve_addresses_widths_and_final_use() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<uint8>:=;ok<boolean>;name<string>;ratio<float32>}>
first:=true
i:=0
r<R>:'out{
    'loop{
        |first|{
            'out->n:=1
            p:&n
            q:&n
            d.print(p==q)
            d.print(*p)
            n=255
            'out->ok:true
            b:&ok
            d.print(*b)
            'out->name:"hello"
            s:&name
            d.print(*s)
            'out->ratio:1.5
            f:&ratio
            d.print(*f)
            first=false
        }
        i=i+1
        |i<2|'loop.restart()
    }
}
d.print(r.n)
"#,
    )
    .runs(b"true\n1\ntrue\nhello\n1.5\n255\n");
}

#[test]
pub fn carried_borrows_keep_function_results_live_until_leave() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
keep<&int32>:(value<&int32>){->value}
x:1
p:=&x
first:=true
r<R>:'out{
    'loop{
        |first|{'out->n:7;p=keep(&n);first=false;'loop.restart()}
        d.print(*p)
        'out.leave()
    }
}
p=&x
d.print(r.n)
"#,
    )
    .runs(b"7\n7\n");
}

#[test]
pub fn carried_borrows_reject_expired_sources_and_conflicting_writes() {
    for (source, code) in [
        (
            "<R>:<{n<int32>}>;x:1;p:=&x;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;p=&n;first=false};i=i+1;|i<2|'loop.restart()}};v:*p",
            "E303",
        ),
        (
            "<R>:<{n<int32>}>;x:1;p:=&x;i:=0;r<R>:'out{v:*p;first:=true;'loop{|first|{'out->n:i;p=&n;first=false;'loop.restart()}};i=i+1;|i<2|'out.restart()}",
            "E303",
        ),
        (
            "<R>:<{n<int32>:=}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:=7;p:&n;n=8;v:*p;first=false};i=i+1;|i<2|'loop.restart()}}",
            "E302",
        ),
    ] {
        let case = Case::new(source);
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                "{source}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
