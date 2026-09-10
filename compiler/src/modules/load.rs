use super::{Failure, File, Graph, MAX_BYTES, MAX_DEPTH, MAX_EDGES, MAX_FILES, MAX_SOURCE};
use crate::ast::{Expr, ExprKind, Span, StmtKind};
use crate::diagnostic::Diagnostic;
use std::collections::BTreeMap;
use std::fs::{self, File as Input};
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) struct Loader {
    pub(crate) graph: Graph,
    pub(crate) ids: BTreeMap<PathBuf, usize>,
    pub(crate) active: Vec<usize>,
    pub(crate) context: Option<PathBuf>,
    pub(crate) bytes: usize,
    pub(crate) edges: usize,
}

impl Loader {
    pub(crate) fn new(entry: &Path) -> Self {
        Self {
            graph: Graph {
                files: Vec::new(),
                order: Vec::new(),
                imports: BTreeMap::new(),
            },
            ids: BTreeMap::new(),
            active: Vec::new(),
            context: manifest(entry),
            bytes: 0,
            edges: 0,
        }
    }

    pub(crate) fn load(mut self, entry: &Path, source: &str) -> Result<Graph, Failure> {
        self.visit(entry.to_path_buf(), source.to_owned())?;
        Ok(self.graph)
    }

    pub(crate) fn visit(&mut self, path: PathBuf, source: String) -> Result<(), Failure> {
        let base = self.bytes;
        if source.len() > MAX_SOURCE || self.bytes + source.len() + 1 > MAX_BYTES {
            return Err(Failure {
                path,
                source,
                errors: vec![Diagnostic::unsupported(
                    "file-module source budget exhausted",
                    Span::default(),
                )],
            });
        }
        self.bytes += source.len() + 1;
        let parsed = crate::parser::parse_documented_at(&source, base).map_err(|mut errors| {
            for error in &mut errors {
                error.span.start = error.span.start.saturating_sub(base);
                error.span.end = error.span.end.saturating_sub(base);
            }
            Failure {
                path: path.clone(),
                source: source.clone(),
                errors,
            }
        })?;
        let id = self.graph.files.len();
        let imports = parsed
            .block
            .stmts
            .iter()
            .filter_map(|stmt| {
                let StmtKind::Bind {
                    mutable: false,
                    ty: None,
                    value,
                    ..
                } = &stmt.kind
                else {
                    return None;
                };
                let value = ungroup(value);
                let ExprKind::Import(name) = &value.kind else {
                    return None;
                };
                (name.starts_with("./") || name.starts_with("../"))
                    .then(|| (name.clone(), value.span))
            })
            .collect::<Vec<_>>();
        self.ids.insert(path.clone(), id);
        self.graph.files.push(File {
            path,
            source,
            base,
            parsed,
        });
        self.active.push(id);
        for (name, span) in imports {
            if self.edges >= MAX_EDGES {
                return Err(self.graph.files[id].failure(Diagnostic::unsupported(
                    "file-module import budget exhausted",
                    span,
                )));
            }
            self.edges += 1;
            let target = self.resolve(id, &name, span)?;
            let target_id = if let Some(target_id) = self.ids.get(&target).copied() {
                if self.active.contains(&target_id) {
                    let mut chain = self
                        .active
                        .iter()
                        .map(|id| self.graph.files[*id].path.display().to_string())
                        .collect::<Vec<_>>();
                    chain.push(target.display().to_string());
                    return Err(self.graph.files[id].failure(Diagnostic::new(
                        "E502",
                        format!("import cycle through {name:?}: {}", chain.join(" -> ")),
                        span,
                    )));
                }
                target_id
            } else {
                if self.graph.files.len() >= MAX_FILES || self.active.len() >= MAX_DEPTH {
                    return Err(self.graph.files[id].failure(Diagnostic::unsupported(
                        "file-module graph budget exhausted",
                        span,
                    )));
                }
                let source = self.read(id, &target, span)?;
                let target_id = self.graph.files.len();
                self.visit(target, source)?;
                target_id
            };
            self.graph.imports.insert(span.start, target_id);
        }
        self.active.pop();
        self.graph.order.push(id);
        Ok(())
    }

    pub(crate) fn resolve(&self, id: usize, name: &str, span: Span) -> Result<PathBuf, Failure> {
        let file = &self.graph.files[id];
        let base = file.path.parent().unwrap_or(Path::new("."));
        let error = |message: String| {
            file.failure(Diagnostic::new(
                "E501",
                format!(
                    "cannot resolve import {name:?} from {}: {message}",
                    base.display()
                ),
                span,
            ))
        };
        if name.contains('\\') || Path::new(name).extension().is_none_or(|ext| ext != "mwy") {
            return Err(error(
                "relative imports require an exact .mwy file with / separators".into(),
            ));
        }
        let target = fs::canonicalize(base.join(name)).map_err(|err| error(err.to_string()))?;
        if !target.is_file() {
            return Err(error("target is not a regular file".into()));
        }
        if target.file_name().is_some_and(|name| name == "mod.mwy")
            || manifest(&target) != self.context
        {
            return Err(file.failure(Diagnostic::unsupported(
                "file imports across package manifest contexts",
                span,
            )));
        }
        Ok(target)
    }

    pub(crate) fn read(&self, id: usize, path: &Path, span: Span) -> Result<String, Failure> {
        let mut bytes = Vec::new();
        Input::open(path)
            .and_then(|file| file.take((MAX_SOURCE + 1) as u64).read_to_end(&mut bytes))
            .map_err(|err| {
                self.graph.files[id].failure(Diagnostic::new(
                    "E501",
                    format!("cannot read imported file {}: {err}", path.display()),
                    span,
                ))
            })?;
        if bytes.len() > MAX_SOURCE {
            return Err(self.graph.files[id].failure(Diagnostic::unsupported(
                "file-module source budget exhausted",
                span,
            )));
        }
        String::from_utf8(bytes).map_err(|error| {
            let utf8 = error.utf8_error();
            Failure {
                path: path.to_owned(),
                source: String::new(),
                errors: vec![Diagnostic::new(
                    "E001",
                    "source contains invalid UTF-8",
                    Span::new(
                        utf8.valid_up_to(),
                        utf8.valid_up_to() + utf8.error_len().unwrap_or(1),
                    ),
                )],
            }
        })
    }
}

pub(crate) fn manifest(path: &Path) -> Option<PathBuf> {
    path.parent()
        .into_iter()
        .flat_map(Path::ancestors)
        .map(|dir| dir.join("mod.mwy"))
        .find(|path| path.exists())
}

pub(crate) fn ungroup(mut expr: &Expr) -> &Expr {
    while let ExprKind::Group(value) = &expr.kind {
        expr = value;
    }
    expr
}
