use std::io::stdin;

use game_lang::{interpreter::Interpreter, lexer::Lexer, parser};

fn main() {
    for line in stdin().lines() {
        let line = line.unwrap();
        let expr = parser::expr(&mut Lexer::lex(&line));
        let res = Interpreter.eval_expr(&expr).unwrap();
        println!("-> {res}");
    }
}