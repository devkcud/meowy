pub mod ast;
pub mod backend;
pub mod check;
pub mod diagnostic;
pub mod driver;
pub mod hir;
pub mod lexer;
pub mod parser;

pub fn compile(source: &str) -> Result<hir::Program, Vec<diagnostic::Diagnostic>> {
    let tree = parser::parse(source)?;
    check::check(&tree)
}
