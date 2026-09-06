pub mod ast;
pub mod backend;
pub(crate) mod borrow;
pub(crate) mod borrow_value;
pub mod check;
pub mod diagnostic;
pub mod driver;
pub(crate) mod flow;
pub mod hir;
pub mod lexer;
pub(crate) mod loans;
pub mod parser;

pub fn compile(source: &str) -> Result<hir::Program, Vec<diagnostic::Diagnostic>> {
    let tree = parser::parse(source)?;
    check::check(&tree)
}
