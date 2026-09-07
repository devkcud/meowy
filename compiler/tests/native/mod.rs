pub(crate) mod cli;
pub(crate) mod control;
pub(crate) mod dispatch;
pub(crate) mod dynamic_lists;
pub(crate) mod effectful_lists;
pub(crate) mod element_borrows;
pub(crate) mod element_writes;
pub(crate) mod emitted_borrows;
pub(crate) mod emitted_slots;
pub(crate) mod examples;
pub(crate) mod fields;
pub(crate) mod function_borrows;
pub(crate) mod immutable_slots;
pub(crate) mod list_contexts;
pub(crate) mod lists;
pub(crate) mod mixed_writes;
pub(crate) mod nested_writes;
pub(crate) mod panics;
pub(crate) mod reborrows;
pub(crate) mod reference_records;
pub(crate) mod reference_slots;
pub(crate) mod reference_unions;
pub(crate) mod references;
pub(crate) mod scalars;
pub(crate) mod transitive_borrows;
pub(crate) mod unions;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static NEXT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Case {
    pub(crate) path: PathBuf,
    pub(crate) source: PathBuf,
}

impl Case {
    pub(crate) fn new(source: &str) -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("meowy-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        let entry = path.join("main.mwy");
        fs::write(&entry, source).unwrap();
        Self {
            path,
            source: entry,
        }
    }

    pub(crate) fn command(&self, action: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_meowy"))
            .arg(action)
            .arg(&self.source)
            .args(["--standalone", "--quiet", "--color", "never"])
            .args(args)
            .current_dir(&self.path)
            .output()
            .unwrap()
    }

    pub(crate) fn runs(&self, expected: &[u8]) {
        for profile in ["debug", "release"] {
            let result = self.command("run", &["--profile", profile]);
            assert!(
                result.status.success(),
                "{profile}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(result.stdout, expected, "{profile}");
            assert!(
                result.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
