use std::io::stdin;

use game_lang::{bytecode::{chunk::CodeChunk, vm::VM}, compiler::parser::Parser};

/// Simple REPL
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .format_timestamp(None)
        .init();

    for line in stdin().lines() {
        let line = line.unwrap();
        let mut code = CodeChunk::new();
        Parser::parse_source(&line, &mut code);
        VM::init(&code).run().unwrap();
    }
}