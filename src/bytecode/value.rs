use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
}

impl Value {
    pub fn neg(&self) -> Option<Value> {
        let res = match self {
            Value::Number(a) => Value::Number(-a),
            _ => return None,
        };
        Some(res)
    }
    pub fn add(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => return None,
        };
        Some(res)
    }
    pub fn sub(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => return None,
        };
        Some(res)
    }
    pub fn mul(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => return None,
        };
        Some(res)
    }
    pub fn div(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => return None,
        };
        Some(res)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(val) => write!(f, "{val}"),
            
        }
    }
}