pub mod lexer;
pub mod parser;
pub mod runtime;

use lexer::Lexer;
use parser::Parser;
use runtime::Runtime;

/// Thực thi một chuỗi mã nguồn MemScript
pub fn execute_script(script: &str) {
    let lexer = Lexer::new(script);
    let mut parser = Parser::new(lexer);
    let statements = parser.parse_program();
    let runtime = Runtime::new();
    runtime.execute_statements(&statements);
}
