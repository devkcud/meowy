use crate::ast::Span;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }

    pub fn unsupported(feature: impl Into<String>, span: Span) -> Self {
        Self::new(
            "B001",
            format!(
                "{} is not supported by this bootstrap compiler",
                feature.into()
            ),
            span,
        )
    }
}
