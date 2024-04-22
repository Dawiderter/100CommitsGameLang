use std::fmt::Display;

use super::object::ObjectId;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    Object(ObjectId)
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
    pub fn not(&self) -> Option<Value> {
        let res = match self {
            Value::Bool(val) => !val,
            Value::Nil => true,
            _ => return None,
        };
        Some(Value::Bool(res))
    }
    pub fn and(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => *a && *b,
            _ => return None,
        };
        Some(Value::Bool(res))
    }
    pub fn or(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => *a || *b,
            _ => return None,
        };
        Some(Value::Bool(res))
    }
    pub fn equal(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => return None,
        };
        Some(Value::Bool(res))
    }
    pub fn greater(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => a > b,
            _ => return None,
        };
        Some(Value::Bool(res))
    }
    pub fn less(&self, other: &Self) -> Option<Value> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => a < b,
            _ => return None,
        };
        Some(Value::Bool(res))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(val) => write!(f, "{val}"),
            Value::Object(id) => write!(f, "Object${id:?}"),
        }
    }
}