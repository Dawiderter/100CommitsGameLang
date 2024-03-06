use std::io::stdin;

use game_lang::{interpreter::Interpreter, lexer::Lexer, parser};

fn main() {
    let mut int = Interpreter::new();

    for line in stdin().lines() {
        let line = line.unwrap();
        let stmt = parser::stmt(&mut Lexer::lex(&line));
        println!("{stmt}");
        let res = int.eval_stmt(&stmt).unwrap();
        if let Some(res) = res {
            println!("-> {res}");
        }
    }
}