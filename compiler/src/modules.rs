mod load;

use crate::ast::{Block, Expr, ExprKind, Span, Stmt, StmtKind};
use crate::diagnostic::Diagnostic;
use crate::{hir, parser};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) const MAX_FILES: usize = 64;
pub(crate) const MAX_DEPTH: usize = 32;
pub(crate) const MAX_SOURCE: usize = 4 * 1024 * 1024;
pub(crate) const MAX_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_EDGES: usize = 4096;

pub(crate) struct File {
    pub(crate) path: PathBuf,
    pub(crate) source: String,
    pub(crate) base: usize,
    pub(crate) parsed: parser::Parsed,
}

#[derive(Debug)]
pub(crate) struct Failure {
    pub(crate) path: PathBuf,
    pub(crate) source: String,
    pub(crate) errors: Vec<Diagnostic>,
}

pub(crate) struct Graph {
    pub(crate) files: Vec<File>,
    pub(crate) order: Vec<usize>,
    pub(crate) imports: BTreeMap<usize, usize>,
}

impl File {
    pub(crate) fn local(&self, error: &Diagnostic) -> Diagnostic {
        Diagnostic::new(
            error.code,
            error.message.clone(),
            Span::new(
                error.span.start.saturating_sub(self.base),
                error.span.end.saturating_sub(self.base),
            ),
        )
    }

    pub(crate) fn failure(&self, error: Diagnostic) -> Failure {
        Failure {
            path: self.path.clone(),
            source: self.source.clone(),
            errors: vec![self.local(&error)],
        }
    }
}

impl Graph {
    pub(crate) fn load(entry: &Path, source: &str) -> Result<Self, Failure> {
        let entry = std::fs::canonicalize(entry).map_err(|error| Failure {
            path: entry.to_owned(),
            source: source.into(),
            errors: vec![Diagnostic::new(
                "E501",
                format!("cannot resolve entry: {error}"),
                Span::default(),
            )],
        })?;
        load::Loader::new(&entry).load(&entry, source)
    }

    pub(crate) fn file(&self, span: Span) -> &File {
        self.files
            .iter()
            .rev()
            .find(|file| span.start >= file.base && span.start <= file.base + file.source.len())
            .unwrap_or(&self.files[0])
    }

    pub(crate) fn compile(&self) -> Result<hir::Program, Vec<Diagnostic>> {
        if self.files.len() == 1 {
            return crate::compile(&self.files[0].source);
        }
        for file in &self.files {
            if let Some(doc) = file.parsed.docs.first() {
                return Err(vec![Diagnostic::unsupported(
                    "documentation in file-module graphs",
                    doc.open,
                )]);
            }
        }
        let mut statements = Vec::new();
        let mut imports = BTreeMap::new();
        for (site, id) in &self.imports {
            imports.insert(*site, format!("\0module{id}"));
        }
        for id in &self.order {
            let file = &self.files[*id];
            let value = Expr {
                kind: ExprKind::Block(file.parsed.block.clone()),
                span: file.parsed.block.span,
            };
            let kind = StmtKind::Bind {
                name: format!("\0module{id}"),
                ty: None,
                mutable: false,
                value,
            };
            statements.push(Stmt {
                kind,
                span: file.parsed.block.span,
            });
        }
        let block = Block {
            label: None,
            stmts: statements,
            span: self.files[0].parsed.block.span,
        };
        crate::check::check_imports(&block, None, imports).map(|(program, _)| program)
    }
}

#[cfg(test)]
mod tests;
