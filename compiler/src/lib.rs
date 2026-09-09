pub mod ast;
pub mod backend;
pub(crate) mod borrow;
pub(crate) mod borrow_contract;
pub(crate) mod borrow_value;
pub mod check;
pub mod diagnostic;
pub mod documentation;
pub mod driver;
pub(crate) mod flow;
pub(crate) mod foundation;
pub mod hir;
pub mod lexer;
pub(crate) mod list;
pub(crate) mod list_context;
pub(crate) mod loans;
pub mod parser;

pub fn compile(source: &str) -> Result<hir::Program, Vec<diagnostic::Diagnostic>> {
    documentation::checked(source, false).map(|(program, _)| program)
}
