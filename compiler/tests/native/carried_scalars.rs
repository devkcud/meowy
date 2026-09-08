use super::Case;

#[test]
pub fn carried_scalar_results_initialize_once_and_support_primaries() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
first:=true
i:=0
builds:=0
r<R>:'out{
    'loop{
        |first|{'out->n:{builds=builds+1;->7};first=false}
        d.print(i)
        i=i+1
        |i<3|'loop.restart()
    }
}
d.print(r.n)
d.print(builds)
first=true
value<int32>:'out{'loop{|first|{'out->9;first=false;'loop.restart()};'out.leave()}}
d.print(value)
"#,
    )
    .runs(b"0\n1\n2\n7\n1\n9\n");
}

#[test]
pub fn carried_scalar_owner_resets_start_fresh_initialization() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<int32>}>
i:=0
r<R>:'out{
    first:=true
    'inner{|first|{d.print("init");'out->n:i;first=false;'inner.restart()}}
    i=i+1
    |i<2|'out.restart()
}
d.print(r.n)
d.print(i)
"#,
    )
    .runs(b"init\ninit\n1\n2\n");
}

#[test]
pub fn carried_scalar_widths_and_mutation_preserve_runtime_values() {
    Case::new(
        r#"
d:@"debug"
<R>:<{n<uint8>:=;ok<boolean>:=;name<string>;ratio<float32>}>
first:=true
i:=0
r<R>:'out{
    'loop{
        |first|{
            'out->n:=1
            old:n
            n=255
            d.print(old)
            'out->ok:=true
            ok=false
            'out->name:"hello"
            'out->ratio:1.5
            first=false
        }
        i=i+1
        |i<2|'loop.restart()
    }
}
d.print(r.n)
d.print(r.ok)
d.print(r.name)
d.print(r.ratio)
p:&r.n
d.print(*p)
"#,
    )
    .runs(b"1\n255\nfalse\nhello\n1.5\n255\n");
}

#[test]
pub fn carried_scalar_boolean_flags_keep_copies_and_pure_operations() {
    Case::new(
        r#"
d:@"debug"
<R>:<{ok<boolean>}>
first:=true
old:first
r<R>:'out{'loop{|first|{'out->ok:old;first=first&&false;'loop.restart()}}}
d.print(r.ok)
first=false
i:=0
s<R>:'out{'loop{|!first|{'out->ok:true;first=!first};i=i+1;|i<2|'loop.restart()}}
d.print(s.ok)
"#,
    )
    .runs(b"true\ntrue\n");
}

#[test]
pub fn carried_scalar_partial_panics_do_not_require_a_completed_value() {
    let code = r#"
d:@"debug"
<R>:<{n<int32>}>
run<int32>:(stop<boolean>){
    first:=true
    r<R>:'out{'loop{|stop|d.panic("halt");|first|{'out->n:7;first=false;'loop.restart()}}}
    ->r.n
}
"#;
    Case::new(&format!("{code}d.print(run(false))")).runs(b"7\n");
    let case = Case::new(&format!("{code}d.print(run(true))"));
    for profile in ["debug", "release"] {
        let output = case.command("run", &["--profile", profile]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("P006"));
    }
}

#[test]
pub fn carried_scalar_incomplete_repeated_and_exclusive_storage_stay_rejected() {
    for (source, code) in [
        (
            "<R>:<{n<int32>}>;first:=false;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<R>:<{n<int32>}>;first:=true;old:first;i:=0;r<R>:'out{'loop{|old|{'out->n:7;first=false};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<R>:<{n<int32>}>;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->n:7;first=first&&{first=false;->true}};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<R>:<{n<int32>:=}>;first:=true;r<R>:'out{'loop{|first|{'out->n:=7;p:&!n;i:=0;'again{v:*p;i=i+1;|i<2|'again.restart()};first=false;'loop.restart()}}}",
            "B001",
        ),
        (
            "<R>:<{p<&int32>}>;x:1;first:=true;i:=0;r<R>:'out{'loop{|first|{'out->p:&x;first=false};i=i+1;|i<2|'loop.restart()}}",
            "B001",
        ),
        (
            "<R>:<{n<int32>}>;r<R>:'out{'loop{'out->n:7;'out->n:8;'loop.restart()}}",
            "E205",
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
