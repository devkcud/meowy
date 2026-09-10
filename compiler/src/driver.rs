use crate::ast::Span;
use crate::diagnostic::Diagnostic;
use std::ffi::OsString;
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicU64, Ordering};

pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const HELP: &str = "meowy 0.0.0 bootstrap compiler

Usage: meowy <check|build|run> FILE [OPTIONS]
       meowy doc <check|build> FILE [OPTIONS]
       meowy help [COMMAND]
       meowy --version

Commands:
  check                 Parse and check without executing application code
  build                 Produce a Linux x86-64 native executable
  run                   Build and execute; program arguments follow --
  doc check             Check attachments, links and complete example source
  doc build             Build a local API site (--output DIR required)

Options:
  --profile debug|release   Select optimization (both check arithmetic)
  --target TRIPLE           Only x86_64-unknown-linux-gnu is available
  --output PATH            Executable destination (build only)
  --offline                Require local inputs (the bootstrap is always offline)
  --quiet                  Suppress successful build messages
  --color auto|always|never Control terminal diagnostic colors
  --help                   Display help

Documentation options:
  --require-public         Require docs on standalone top-level API declarations
  --run-examples           Execute run examples (doc check only)
  --example-timeout-ms N   Per-example limit, 1..60000 (default 5000)

Bootstrap options:
  --standalone             Check source outside ancestor manifest policy
  --json                   Emit bootstrap JSON diagnostics on stderr
  --emit-llvm PATH          Preserve LLVM IR (build only)

This compiler supports the scalar native milestone. Unsupported language features
report B001. Project manifests, packages, ownership, and the complete standard
library are not implemented. See compiler/STATUS.md for the handoff and roadmap.
";

pub(crate) static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Check,
    Build,
    Run,
    DocCheck,
    DocBuild,
}

#[derive(Debug)]
pub(crate) struct Options {
    pub(crate) action: Action,
    pub(crate) entry: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) ir: Option<PathBuf>,
    pub(crate) release: bool,
    pub(crate) target: String,
    pub(crate) standalone: bool,
    pub(crate) quiet: bool,
    pub(crate) json: bool,
    pub(crate) color: bool,
    pub(crate) args: Vec<OsString>,
    pub(crate) require_public: bool,
    pub(crate) run_examples: bool,
    pub(crate) example_timeout: u64,
}

pub fn run(args: Vec<OsString>) -> i32 {
    if args.is_empty()
        || args
            .first()
            .is_some_and(|s| s == "help" || s == "--help" || s == "-h")
    {
        if args.first().is_some_and(|s| s == "help")
            && args
                .get(1)
                .is_some_and(|s| !matches!(s.to_str(), Some("check" | "build" | "run" | "doc")))
        {
            eprintln!("meowy: help is available for check, build, and run");
            return 2;
        }
        print!("{HELP}");
        return 0;
    }
    if args.len() == 1 && args[0] == "--version" {
        println!(
            "meowy 0.0.0-bootstrap (Rust 1.98.1; LLVM {}; {TARGET}; host toolchain)",
            env!("MEOWY_LLVM_VERSION")
        );
        return 0;
    }
    if args
        .iter()
        .take_while(|s| *s != "--")
        .any(|s| s == "--help" || s == "-h")
    {
        print!("{HELP}");
        return 0;
    }
    let opts = match options(args) {
        Ok(opts) => opts,
        Err(message) => {
            eprintln!("meowy: {message}\nRun 'meowy help' for usage.");
            return 2;
        }
    };
    execute(&opts)
}

pub(crate) fn options(args: Vec<OsString>) -> Result<Options, String> {
    let mut args = args.into_iter();
    let action = match args.next().and_then(|s| s.into_string().ok()).as_deref() {
        Some("check") => Action::Check,
        Some("build") => Action::Build,
        Some("run") => Action::Run,
        Some("doc") => match args.next().and_then(|s| s.into_string().ok()).as_deref() {
            Some("check") => Action::DocCheck,
            Some("build") => Action::DocBuild,
            _ => return Err("doc requires check or build".into()),
        },
        Some(name) => return Err(format!("command '{name}' is unavailable in this bootstrap")),
        None => return Err("expected a command".into()),
    };
    let mut opts = Options {
        action,
        entry: PathBuf::new(),
        output: None,
        ir: None,
        release: false,
        target: TARGET.into(),
        standalone: false,
        quiet: false,
        json: false,
        color: io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
        args: Vec::new(),
        require_public: false,
        run_examples: false,
        example_timeout: 5000,
    };
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--") => {
                if action != Action::Run {
                    return Err("application arguments are allowed only with run".into());
                }
                opts.args = args.collect();
                break;
            }
            Some("--profile") => {
                opts.release = match args.next().and_then(|s| s.into_string().ok()).as_deref() {
                    Some("debug") => false,
                    Some("release") => true,
                    _ => return Err("--profile requires debug or release".into()),
                };
            }
            Some("--target") => {
                opts.target = args
                    .next()
                    .and_then(|s| s.into_string().ok())
                    .ok_or("--target requires a target triple")?;
            }
            Some("--output") => {
                if !matches!(action, Action::Build | Action::DocBuild) {
                    return Err("--output is allowed only with build".into());
                }
                opts.output = Some(
                    args.next()
                        .map(PathBuf::from)
                        .ok_or("--output requires a path")?,
                );
            }
            Some("--emit-llvm") => {
                if action != Action::Build {
                    return Err("--emit-llvm is allowed only with build".into());
                }
                opts.ir = Some(
                    args.next()
                        .map(PathBuf::from)
                        .ok_or("--emit-llvm requires a path")?,
                );
            }
            Some("--color") => {
                opts.color = match args.next().and_then(|s| s.into_string().ok()).as_deref() {
                    Some("always") => true,
                    Some("never") => false,
                    Some("auto") => {
                        io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
                    }
                    _ => return Err("--color requires auto, always, or never".into()),
                };
            }
            Some("--offline") => {}
            Some("--require-public") if matches!(action, Action::DocCheck | Action::DocBuild) => {
                opts.require_public = true
            }
            Some("--run-examples") if action == Action::DocCheck => opts.run_examples = true,
            Some("--example-timeout-ms") if action == Action::DocCheck => {
                opts.example_timeout = args
                    .next()
                    .and_then(|value| value.into_string().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .filter(|value| (1..=60000).contains(value))
                    .ok_or("--example-timeout-ms requires 1..60000")?;
            }
            Some("--standalone") => opts.standalone = true,
            Some("--quiet") => opts.quiet = true,
            Some("--json") => opts.json = true,
            Some(name) if name.starts_with('-') => return Err(format!("unknown option '{name}'")),
            _ => {
                if !opts.entry.as_os_str().is_empty() {
                    return Err("expected one entry file; program arguments must follow --".into());
                }
                opts.entry = arg.into();
            }
        }
    }
    if opts.entry.as_os_str().is_empty() {
        return Err(
            "an explicit .mwy entry file is required; manifest entry selection is not implemented"
                .into(),
        );
    }
    if opts.entry.extension().is_none_or(|ext| ext != "mwy") {
        return Err("the entry must be a .mwy source file".into());
    }
    if action == Action::DocBuild && opts.output.is_none() {
        return Err("doc build requires --output DIR".into());
    }
    Ok(opts)
}

pub(crate) fn execute(opts: &Options) -> i32 {
    if opts.target != TARGET || !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        report(
            opts,
            "",
            &Diagnostic::new(
                "E507",
                format!(
                    "target '{}' is unavailable; this bootstrap supports {TARGET} on Linux x86-64",
                    opts.target
                ),
                Span::default(),
            ),
        );
        return 1;
    }
    let entry = match fs::canonicalize(&opts.entry) {
        Ok(path) => path,
        Err(err) => {
            report(
                opts,
                "",
                &Diagnostic::new(
                    "E501",
                    format!("cannot read {}: {err}", opts.entry.display()),
                    Span::default(),
                ),
            );
            return 1;
        }
    };
    if !opts.standalone {
        for dir in entry.parent().into_iter().flat_map(Path::ancestors) {
            let manifest = dir.join("mod.mwy");
            if manifest.exists() {
                report(
                    opts,
                    "",
                    &Diagnostic::unsupported(
                        format!(
                            "project manifest {} (use --standalone only when intentionally checking an isolated source)",
                            manifest.display()
                        ),
                        Span::default(),
                    ),
                );
                return 1;
            }
        }
    }
    let bytes = match fs::read(&entry) {
        Ok(bytes) => bytes,
        Err(err) => {
            report(
                opts,
                "",
                &Diagnostic::new(
                    "E501",
                    format!("cannot read source: {err}"),
                    Span::default(),
                ),
            );
            return 1;
        }
    };
    let source = match std::str::from_utf8(&bytes) {
        Ok(source) => source,
        Err(err) => {
            report(
                opts,
                "",
                &Diagnostic::new(
                    "E001",
                    "source contains invalid UTF-8",
                    Span::new(
                        err.valid_up_to(),
                        err.valid_up_to() + err.error_len().unwrap_or(1),
                    ),
                ),
            );
            return 1;
        }
    };
    if matches!(opts.action, Action::DocCheck | Action::DocBuild) {
        return crate::documentation::execute(opts, source, &entry);
    }
    let graph = match crate::modules::Graph::load(&entry, source) {
        Ok(graph) => graph,
        Err(failure) => {
            for error in &failure.errors {
                report_at(opts, &failure.path, &failure.source, error);
            }
            return 1;
        }
    };
    let program = match graph.compile() {
        Ok(program) => program,
        Err(errors) => {
            for error in errors {
                let file = graph.file(error.span);
                report_at(opts, &file.path, &file.source, &file.local(&error));
            }
            return 1;
        }
    };
    if opts.action == Action::Check {
        return 0;
    }
    let output = opts.output.clone().unwrap_or_else(|| {
        entry
            .parent()
            .unwrap_or(Path::new("."))
            .join("build")
            .join(TARGET)
            .join(if opts.release { "release" } else { "debug" })
            .join(entry.file_stem().unwrap_or_default())
    });
    for path in std::iter::once(&output).chain(opts.ir.iter()) {
        for file in &graph.files {
            if let Err(err) = protect(path, &file.path) {
                eprintln!("meowy: {err}");
                return 2;
            }
        }
    }
    if opts
        .ir
        .as_ref()
        .is_some_and(|ir| matches!((absolute(ir), absolute(&output)), (Ok(a), Ok(b)) if a == b))
    {
        eprintln!("meowy: executable and LLVM IR outputs must have different paths");
        return 2;
    }
    let ir = match crate::backend::emit_ir(&program) {
        Ok(ir) => ir,
        Err(err) => {
            report(
                opts,
                source,
                &Diagnostic::new(
                    "F001",
                    format!("LLVM lowering failed: {err}"),
                    Span::default(),
                ),
            );
            return 1;
        }
    };
    match build(&ir, &output, opts) {
        Ok(()) => {}
        Err(err) => {
            report(opts, source, &Diagnostic::new("B002", err, Span::default()));
            return 1;
        }
    }
    if opts.action == Action::Build {
        if !opts.quiet && !opts.json {
            eprintln!("built {}", output.display());
        }
        return 0;
    }
    let path = match fs::canonicalize(&output) {
        Ok(path) => path,
        Err(err) => {
            report(
                opts,
                source,
                &Diagnostic::new(
                    "B002",
                    format!("cannot locate built executable: {err}"),
                    Span::default(),
                ),
            );
            return 1;
        }
    };
    match Command::new(path).args(&opts.args).status() {
        Ok(status) => exit_code(status),
        Err(err) => {
            report(
                opts,
                source,
                &Diagnostic::new(
                    "E507",
                    format!("cannot launch application: {err}"),
                    Span::default(),
                ),
            );
            1
        }
    }
}

pub(crate) fn protect(path: &Path, entry: &Path) -> Result<(), String> {
    if matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("mwy" | "replay")
    ) || path.file_name().is_some_and(|name| name == "mod.lock")
    {
        return Err(format!(
            "output would overwrite a protected input: {}",
            path.display()
        ));
    }
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() || !meta.is_file() {
            return Err(format!("output is not a regular file: {}", path.display()));
        }
        if fs::canonicalize(path).ok().as_deref() == Some(entry) {
            return Err(format!(
                "output would overwrite the source: {}",
                path.display()
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(input) = fs::metadata(entry)
                && input.dev() == meta.dev()
                && input.ino() == meta.ino()
            {
                return Err(format!(
                    "output is a hard link to the source: {}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn absolute(path: &Path) -> io::Result<PathBuf> {
    let path = std::path::absolute(path)?;
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    Ok(fs::canonicalize(parent)?.join(path.file_name().unwrap_or_default()))
}

pub(crate) struct Scratch {
    pub(crate) path: PathBuf,
}

impl Scratch {
    pub(crate) fn new(parent: &Path) -> io::Result<Self> {
        for _ in 0..100 {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".meowy-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "unable to create a unique build directory",
        ))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn build(ir: &str, output: &Path, opts: &Options) -> Result<(), String> {
    let output = absolute(output).map_err(|err| format!("cannot prepare output: {err}"))?;
    let scratch = Scratch::new(output.parent().unwrap_or(Path::new(".")))
        .map_err(|err| format!("cannot create build directory: {err}"))?;
    let object = scratch.path.join("main.o");
    let runtime = scratch.path.join("runtime.a");
    let binary = scratch.path.join("program");
    crate::backend::emit_object(ir, &object, opts.release)
        .map_err(|err| format!("LLVM object emission failed: {err}"))?;
    fs::write(&runtime, crate::backend::RUNTIME_ARCHIVE)
        .map_err(|err| format!("cannot write runtime: {err}"))?;
    let mut command = Command::new(env!("MEOWY_CLANG"));
    command
        .arg(format!("--ld-path={}", env!("MEOWY_LLD")))
        .args([
            "--target=x86_64-unknown-linux-gnu",
            "-march=x86-64",
            "-fPIE",
            "-pie",
            "-Wl,--gc-sections",
            "-Wl,--build-id=sha1",
        ])
        .arg(&object)
        .arg(&runtime)
        .arg("-o")
        .arg(&binary);
    for name in [
        "CC",
        "CXX",
        "CFLAGS",
        "CXXFLAGS",
        "CPPFLAGS",
        "LDFLAGS",
        "LIBRARY_PATH",
        "COMPILER_PATH",
        "GCC_EXEC_PREFIX",
        "CPATH",
        "C_INCLUDE_PATH",
        "CPLUS_INCLUDE_PATH",
    ] {
        command.env_remove(name);
    }
    let result = command.output().map_err(|err| {
        format!(
            "cannot run pinned linker driver {}: {err}",
            env!("MEOWY_CLANG")
        )
    })?;
    if !result.status.success() {
        return Err(format!(
            "native link failed ({})\ncommand: {command:?}\n{}{}",
            result.status,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    if let Some(path) = &opts.ir {
        let path = absolute(path).map_err(|err| format!("cannot prepare IR output: {err}"))?;
        let temp = Scratch::new(path.parent().unwrap_or(Path::new(".")))
            .map_err(|err| format!("cannot prepare IR output: {err}"))?;
        let staged = temp.path.join("module.ll");
        fs::write(&staged, ir).map_err(|err| format!("cannot write LLVM IR: {err}"))?;
        fs::rename(&staged, path).map_err(|err| format!("cannot publish LLVM IR: {err}"))?;
    }
    fs::rename(&binary, &output).map_err(|err| format!("cannot publish executable: {err}"))?;
    Ok(())
}

pub(crate) fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        128 + status.signal().unwrap_or(1)
    }
    #[cfg(not(unix))]
    {
        1
    }
}

pub(crate) fn report(opts: &Options, source: &str, error: &Diagnostic) {
    report_at(opts, &opts.entry, source, error);
}

pub(crate) fn report_at(opts: &Options, path: &Path, source: &str, error: &Diagnostic) {
    let start = floor_char(source, error.span.start.min(source.len()));
    let line = source[..start].bytes().filter(|b| *b == b'\n').count() + 1;
    let left = source[..start].rfind('\n').map_or(0, |i| i + 1);
    let column = source[left..start].chars().count() + 1;
    if opts.json {
        eprintln!(
            "{{\"schema\":\"meowy.bootstrap.diagnostic\",\"version\":1,\"code\":{},\"message\":{},\"path\":{},\"start\":{},\"end\":{},\"line\":{line},\"column\":{column}}}",
            json(error.code),
            json(&error.message),
            json(&path.to_string_lossy()),
            error.span.start,
            error.span.end
        );
        return;
    }
    let (red, reset) = if opts.color {
        ("\u{1b}[1;31m", "\u{1b}[0m")
    } else {
        ("", "")
    };
    eprintln!("{red}error[{}]{reset}: {}", error.code, error.message);
    eprintln!("  --> {}:{line}:{column}", path.display());
    if source.is_empty() {
        return;
    }
    let right = source[start..]
        .find('\n')
        .map_or(source.len(), |i| start + i);
    let text = source[left..right]
        .trim_end_matches('\r')
        .replace('\t', "    ");
    let width = line.to_string().len();
    let indent: usize = source[left..start]
        .chars()
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum();
    let end = floor_char(source, error.span.end.min(right).max(start));
    let count = source[start..end].chars().count().clamp(1, 80);
    eprintln!("{line:>width$} | {text}");
    eprintln!(
        "{:>width$} | {}{red}{}{reset}",
        "",
        " ".repeat(indent),
        "^".repeat(count)
    );
}

pub(crate) fn floor_char(source: &str, mut offset: usize) -> usize {
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

pub(crate) fn json(text: &str) -> String {
    let mut value = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => value.push_str("\\\""),
            '\\' => value.push_str("\\\\"),
            '\n' => value.push_str("\\n"),
            '\r' => value.push_str("\\r"),
            '\t' => value.push_str("\\t"),
            ch if ch < '\u{20}' => value.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => value.push(ch),
        }
    }
    value.push('"');
    value
}
