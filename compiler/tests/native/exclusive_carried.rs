use super::Case;

#[test]
pub fn exclusive_carried_scalar_widths_mutate_original_result_storage() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<uint8>:=;b<boolean>:=;f<float32>:=}>
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->n:=1
            old:n
            p:&!n
            *p=255
            d.print(old)
            'out->b:=true
            q:&!b
            *q=false
            'out->f:=1.5
            s:&!f
            *s=2.5
            first=false
            'loop.restart()
        }
    }
}
d.print(r.n)
d.print(r.b)
d.print(r.f)
"#,
    )
    .runs(b"1\n255\nfalse\n2.5\n");
}

#[test]
pub fn exclusive_carried_moves_children_and_calls_finish_before_restart() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>:=}>
bump<null>:(p<&!int32>){*p=*p+1}
first:=true
r<R>:'out{
    'loop{
        |first|{
            'out->n:=7
            p:&!n
            s:&*p
            d.print(*s)
            q:&!*p
            *q=8
            d.print(*q)
            *p=9
            d.print(*p)
            moved:p
            bump(moved)
            first=false
            'loop.restart()
        }
    }
}
d.print(r.n)
"#,
    )
    .runs(b"7\n8\n9\n10\n");
}

#[test]
pub fn exclusive_carried_owner_resets_create_fresh_local_loans() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>:=}>
i:=0
r<R>:'out{
    first:=true
    'loop{
        |first|{
            'out->n:=i
            p:&!n
            *p=*p+10
            d.print(*p)
            first=false
            'loop.restart()
        }
    }
    i=i+1
    |i<2|'out.restart()
}
d.print(r.n)
"#,
    )
    .runs(b"10\n11\n11\n");
}

#[test]
pub fn exclusive_carried_leave_skips_the_captured_store() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>:=}>
run<null>:(stop<boolean>){
    first:=true
    r<R>:'out{
        'loop{
            |first|{
                'out->n:=3
                p:&!n
                *p={d.print("rhs");|stop|'out.leave();->9}
                first=false
                'loop.restart()
            }
        }
    }
    d.print(r.n)
}
run(false)
run(true)
"#,
    )
    .runs(b"rhs\n9\nrhs\n3\n");
}

#[test]
pub fn exclusive_carried_invalid_uses_and_frontiers_keep_primary_diagnostics() {
    for (body, code) in [
        ("p:&!n;q:p;v:*p;w:*q;first=false", "E301"),
        ("p:&!n;n=8;v:*p;first=false", "E302"),
        ("p:&!n;s:&*p;*p=8;v:*s;first=false", "E302"),
        ("p:&!n;first=false;*p=8", "B001"),
        (
            "p:&!n;i:=0;'again{v:*p;i=i+1;|i<2|'again.restart()};first=false",
            "B001",
        ),
        (
            "p:&!n;s:&*p;i:=0;'again{v:*s;i=i+1;|i<2|'again.restart()};first=false",
            "B001",
        ),
    ] {
        let case = Case::new(&format!(
            "<R>:<{{n<int32>:=}}>;first:=true;r<R>:'out{{'loop{{|first|{{'out->n:=7;{body};'loop.restart()}}}}}}"
        ));
        for profile in ["debug", "release"] {
            let output = case.command("run", &["--profile", profile]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).starts_with(&format!("error[{code}]:")),
                "{body}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
