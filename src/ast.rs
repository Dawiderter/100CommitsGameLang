use std::fmt::{Debug, Display};

use crate::lexer::Operator;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name : String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Binary(Operator, Box<Expr>, Box<Expr>),
    Unary(Operator, Box<Expr>),
    Variable(Var),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaration(String, Expr),
    Assign(Var, Expr),
    Expr(Expr),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(bool) => write!(f, "{bool}"),
            Value::String(string) => write!(f, "{string}"),
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Value(val) => {
                write!(f, "{val}")
            }
            Expr::Binary(op, left, right) => {
                write!(f, "({op} {left} {right})")
            }
            Expr::Unary(op, expr) => {
                write!(f, "({op} {expr})")
            }
            Expr::Variable(var) => {
                write!(f, "{var}")
            },
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Declaration(name, e) => {
                write!(f, "let {name} = {e};")
            },
            Stmt::Assign(var, e) => {
                write!(f, "{var} = {e};")
            },
            Stmt::Expr(e) => {
                write!(f, "{e};")
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_test() {
        let expr = Expr::Binary(
            Operator::Add,
            Box::new(Expr::Unary(
                Operator::Sub,
                Box::new(Expr::Value(Value::Number(50.0))),
            )),
            Box::new(Expr::Binary(
                Operator::Mul,
                Box::new(Expr::Value(Value::Number(100.0))),
                Box::new(Expr::Value(Value::Number(2.0))),
            )),
        );

        println!("{expr}");
    }
}
