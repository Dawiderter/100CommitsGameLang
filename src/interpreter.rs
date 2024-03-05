use crate::ast::*;
use crate::lexer::Operator;

#[derive(Debug, Clone, Copy)]
pub enum InterpreterError {
    TypeMismatch,
}

pub struct Interpreter;

impl Interpreter {
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        match expr {
            Expr::Value(val) => Ok(val.to_owned()),
            Expr::Binary(op, left, right) => {
                let left_value = self.eval_expr(left)?;
                let right_value = self.eval_expr(right)?;

                let result = match (op, left_value, right_value) {
                    (Operator::Add, Value::Number(x), Value::Number(y)) => Value::Number(x + y),
                    (Operator::Sub, Value::Number(x), Value::Number(y)) => Value::Number(x - y),
                    (Operator::Mul, Value::Number(x), Value::Number(y)) => Value::Number(x * y),
                    (Operator::Div, Value::Number(x), Value::Number(y)) => Value::Number(x / y),
                    (Operator::Rem, Value::Number(x), Value::Number(y)) => Value::Number(x % y),
                    (Operator::Eq, Value::Number(x), Value::Number(y)) => Value::Bool(x == y),
                    (Operator::Neq, Value::Number(x), Value::Number(y)) => Value::Bool(x != y),
                    (Operator::Geq, Value::Number(x), Value::Number(y)) => Value::Bool(x >= y),
                    (Operator::Leq, Value::Number(x), Value::Number(y)) => Value::Bool(x <= y),
                    (Operator::Gr, Value::Number(x), Value::Number(y)) => Value::Bool(x > y),
                    (Operator::Le, Value::Number(x), Value::Number(y)) => Value::Bool(x < y),
                    (Operator::Eq, Value::Bool(x), Value::Bool(y)) => Value::Bool(x == y),
                    (Operator::Neq, Value::Bool(x), Value::Bool(y)) => Value::Bool(x != y),
                    (Operator::And, Value::Bool(x), Value::Bool(y)) => Value::Bool(x && y),
                    (Operator::Or, Value::Bool(x), Value::Bool(y)) => Value::Bool(x || y),
                    (Operator::Eq, Value::String(x), Value::String(y)) => Value::Bool(x == y),
                    (Operator::Neq, Value::String(x), Value::String(y)) => Value::Bool(x != y),
                    _ => {
                        return Err(InterpreterError::TypeMismatch);
                    }
                };

                Ok(result)
            }
            Expr::Unary(op, expr) => {
                let value = self.eval_expr(expr)?;

                let res = match (op, value) {
                    (Operator::Sub, Value::Number(x)) => Value::Number(-x),
                    (Operator::Not, Value::Bool(x)) => Value::Bool(!x),
                    _ => {
                        return Err(InterpreterError::TypeMismatch);
                    }
                };

                Ok(res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expr_test() {
        let expr = Expr::Binary(
            Operator::Div,
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

        println!("result: {}", Interpreter.eval_expr(&expr).unwrap());
    }
}
