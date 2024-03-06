use std::ops::Range;

use crate::{
    ast::{Expr, Stmt, Value, Var},
    lexer::{Lexer, Operator, Token},
};

#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
    UnexpectedToken(Range<usize>),
    LexerError(Range<usize>),
}

pub fn stmt(lexer: &mut Lexer<'_>) -> Stmt {
    let peeked = peek_token(lexer).unwrap();

    if matches!(peeked, Token::Let) {
        next_token(lexer).unwrap();
        let tok_id = next_token(lexer).unwrap();
        let Token::Identifier(id) = tok_id else { panic!() };

        assert_eq!(peek_token(lexer).unwrap(), &Token::Assign);
        next_token(lexer).unwrap();

        let expr = expr(lexer).unwrap();

        assert_eq!(peek_token(lexer).unwrap(), &Token::Semicolon);
        next_token(lexer).unwrap();

        return Stmt::Declaration(id.to_owned(), expr);
    }

    let left = expr(lexer).unwrap();
    if let Expr::Variable(var) = left {
        assert_eq!(peek_token(lexer).unwrap(), &Token::Assign);
        next_token(lexer).unwrap();

        let right = expr(lexer).unwrap();

        assert_eq!(peek_token(lexer).unwrap(), &Token::Semicolon);
        next_token(lexer).unwrap();

        return Stmt::Assign(var, right);
    }

    assert_eq!(peek_token(lexer).unwrap(), &Token::Semicolon);
    next_token(lexer).unwrap();
    Stmt::Expr(left)
}

pub fn expr(lexer: &mut Lexer<'_>) -> Result<Expr, ParserError> {
    expr_bp(lexer, 0)
}

fn expr_bp(lexer: &mut Lexer<'_>, min_bp: u8) -> Result<Expr, ParserError> {
    let mut left = match next_token(lexer)? {
        Token::Number(n) => Expr::Value(Value::Number(n)),
        Token::String(s) => Expr::Value(Value::String(s.to_owned())),
        Token::Bool(b) => Expr::Value(Value::Bool(b)),
        Token::Identifier(s) => Expr::Variable(Var { name : s.to_owned() } ),
        Token::Operator(Operator::ParenOpen) => {
            let left = expr_bp(lexer, 0)?;
            let end_token = next_token(lexer)?;
            if end_token != Token::Operator(Operator::ParenClose) {
                return Err(ParserError::UnexpectedToken(lexer.span()));
            };
            left
        }
        Token::Operator(op) => {
            let (_, r_bp) = prefix_binding_power(op).ok_or(ParserError::UnexpectedToken(lexer.span()))?;
            let right = expr_bp(lexer, r_bp)?;

            Expr::Unary(op, Box::new(right))
        }
        _ => return Err(ParserError::UnexpectedToken(lexer.span())),
    };

    loop {
        let &op = match peek_token(lexer)? {
            Token::EOF => break,
            Token::Operator(op) => op,
            _ => break,
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
        // Operator::ParenOpen => (220, ()),
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

        println!("{:?}", parsed);
    }
}