use std::ops::Range;

use crate::{
    ast::{Expr, Value},
    lexer::{Lexer, Operator, Token},
};

#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
    UnexpectedToken(Range<usize>),
    LexerError(Range<usize>),
}

pub fn expr(lexer: &mut Lexer<'_>) -> Expr {
    expr_bp(lexer, 0).unwrap()
}

fn expr_bp(lexer: &mut Lexer<'_>, min_bp: u8) -> Result<Expr, ParserError> {
    let mut left = match next_token(lexer)? {
        Token::Number(n) => Expr::Value(Value::Number(n)),
        Token::String(s) => Expr::Value(Value::String(s.to_owned())),
        Token::Bool(b) => Expr::Value(Value::Bool(b)),
        Token::Operator(op) => {
            let (_, r_bp) = prefix_binding_power(op).ok_or(ParserError::UnexpectedToken(lexer.span()))?;
            let right = expr_bp(lexer, r_bp)?;

            Expr::Unary(op, Box::new(right))
        }
        Token::ParenOpen => {
            let left = expr_bp(lexer, 0)?;
            let end_token = next_token(lexer)?;
            if end_token != Token::ParenClose {
                eprintln!("{:?}", end_token);
                return Err(ParserError::UnexpectedToken(lexer.span()));
            };
            left
        }
        _ => return Err(ParserError::UnexpectedToken(lexer.span())),
    };

    loop {
        let &op = match peek_token(lexer)? {
            Token::EOF | Token::ParenClose => break,
            Token::Operator(op) => op,
            _ => return Err(ParserError::UnexpectedToken(lexer.span())),
        };

        if let Some((l_bp, _)) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
    
            next_token(lexer)?;
            left = Expr::Unary(op, Box::new(left));

            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
    
            next_token(lexer)?;
            let right = expr_bp(lexer, r_bp)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));

            continue;
        }

        break;
    }

    Ok(left)
}

fn prefix_binding_power(op: Operator) -> Option<((), u8)> {
    let bp = match op {
        Operator::Add | Operator::Sub | Operator::Not => ((), 201),
        _ => return None,
    };
    Some(bp)
}

fn infix_binding_power(op: Operator) -> Option<(u8, u8)> {
    let bp = match op {
        Operator::Eq | Operator::Neq | Operator::Leq | Operator::Geq | Operator::Gr | Operator::Le => (50,51),
        Operator::Add | Operator::Sub | Operator::Or => (100, 101),
        Operator::Mul | Operator::Div | Operator::Rem | Operator::And => (150, 151),
        _ => return None,
    };
    Some(bp)
}

fn postfix_binding_power(op: Operator) -> Option<(u8, ())> {
    let bp = match op {
        _ => return None,
    };
    Some(bp)
}

fn next_token<'source>(lexer: &mut Lexer<'source>) -> Result<Token<'source>, ParserError> {
    lexer
        .next()
        .ok_or(ParserError::EndOfInput)?
        .map_err(|_| ParserError::LexerError(lexer.span()))
}

fn peek_token<'source, 'lex>(
    lexer: &'lex mut Lexer<'source>,
) -> Result<&'lex Token<'source>, ParserError> {
    lexer.peek();

    let peeked_span = lexer.span();

    lexer
        .peek()
        .ok_or(ParserError::EndOfInput)?
        .as_ref()
        .map_err(|_| ParserError::LexerError(peeked_span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expr_test() {
        let input = "-(1 + 10) * 20 / ((+(5 + 9)))";

        let parsed = expr(&mut Lexer::lex(input));

        println!("{}", parsed);
    }
}