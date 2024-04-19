use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64)
}

impl Value {
    pub fn neg(&self) -> Value {
        match self {
            Value::Number(a) => Value::Number(-a),
        }
    }
    pub fn add(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
        }
    }
    pub fn sub(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
        }
    }
    pub fn mul(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
        }
    }
    pub fn div(&self, other: &Self) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{num}"),
        }
    }
}