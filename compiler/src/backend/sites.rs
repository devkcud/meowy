use super::Generator;
use crate::ast::Span;

#[derive(Clone, Debug)]
pub struct Source {
    pub path: String,
    pub span: Span,
}

pub(crate) fn validate(sources: &[Source]) -> Result<(), String> {
    if sources.len() > 64 {
        return Err("runtime source map exceeds 64 files".into());
    }
    let mut end = None;
    let mut bytes = 0usize;
    for source in sources {
        if source.path.is_empty() || source.path.len() > 1024 * 1024 - bytes {
            return Err("runtime source labels are empty or exceed 1 MiB".into());
        }
        bytes += source.path.len();
        if source.span.start > source.span.end || end.is_some_and(|end| source.span.start <= end) {
            return Err("runtime source ranges must be ordered and disjoint".into());
        }
        end = Some(source.span.end);
    }
    Ok(())
}

impl Generator<'_> {
    pub(crate) fn capture(
        &mut self,
        name: &str,
        panic: &str,
        args: &str,
        span: Span,
    ) -> Result<String, String> {
        let args = if args.is_empty() {
            String::new()
        } else {
            format!(", {args}")
        };
        if self.sources.is_empty() {
            return Ok(format!(
                "call void @meowy_{name}_v0(ptr {panic}{args}, i64 {}, i64 {})",
                span.start, span.end
            ));
        }
        let (id, source) = self
            .sources
            .iter()
            .enumerate()
            .find(|(_, source)| {
                span.start <= span.end
                    && span.start >= source.span.start
                    && span.end <= source.span.end
            })
            .ok_or("runtime site is outside the source map")?;
        let start = span.start - source.span.start;
        let end = span.end - source.span.start;
        let file = if let Some(file) = self.paths.get(&id) {
            file.clone()
        } else {
            let path = source.path.clone();
            let file = self.string(&path);
            self.paths.insert(id, file.clone());
            file
        };
        let path = self.value(format!("extractvalue {{ ptr, i64 }} {file}, 0"));
        let size = self.value(format!("extractvalue {{ ptr, i64 }} {file}, 1"));
        Ok(format!(
            "call void @meowy_{name}_file_v0(ptr {panic}{args}, i64 {start}, i64 {end}, ptr {path}, i64 {size})"
        ))
    }
}
