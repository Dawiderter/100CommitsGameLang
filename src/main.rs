use std::io::stdin;

use game_lang::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

fn main() {
    let mut int = Interpreter::new();

    for line in stdin().lines() {
        let line = line.unwrap();
        let stmt = Parser::parse(Lexer::lex(&line)).stmt();
        match stmt {
            Ok(stmt) => {
                let res = int.eval_stmt(&stmt).unwrap();
                if let Some(res) = res {
                    println!("-> {res}");
                }
            },
            Err(err) => {
                println!("{:?}", err)
            },
        }
    }
}