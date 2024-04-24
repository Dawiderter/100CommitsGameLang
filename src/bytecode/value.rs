use std::fmt::Display;

use ecow::eco_format;

use super::object::{HeapError, ObjectHeap, ObjectKey, ObjectKind};

#[derive(Debug, Clone, Copy)]
pub enum ValueError {
    UnSupportedOperation,
    HeapError(HeapError),
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    Object(ObjectKey),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Nil | Value::Bool(false))
    }
    pub fn neg(&self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match self {
            Value::Number(a) => Value::Number(-a),
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(res)
    }
    pub fn add(&self, other: &Self, heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Object(a), Value::Object(b)) => {
                match (&heap.get_object(*a)?.kind, &heap.get_object(*b)?.kind) {
                    (ObjectKind::String(a), ObjectKind::String(b)) => {
                        let joined_string = eco_format!("{}{}", a, b);
                        let key = heap.intern_string(joined_string);
                        Value::Object(key)
                    },
                }
            }
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(res)
    }
    pub fn sub(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(res)
    }
    pub fn mul(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(res)
    }
    pub fn div(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(res)
    }
    pub fn not(&self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match self {
            Value::Bool(val) => !val,
            Value::Nil => true,
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(Value::Bool(res))
    }
    pub fn and(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => *a && *b,
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(Value::Bool(res))
    }
    pub fn or(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => *a || *b,
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(Value::Bool(res))
    }
    pub fn equal(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(Value::Bool(res))
    }
    pub fn greater(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => a > b,
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(Value::Bool(res))
    }
    pub fn less(&self, other: &Self, _heap: &mut ObjectHeap) -> Result<Value, ValueError> {
        let res = match (self, other) {
            (Value::Number(a), Value::Number(b)) => a < b,
            _ => return Err(ValueError::UnSupportedOperation),
        };
        Ok(Value::Bool(res))
    }
}

impl From<HeapError> for ValueError {
    fn from(value: HeapError) -> Self {
        Self::HeapError(value)
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

#[derive(Debug)]
pub struct ValueHeapDisplay<'value, 'heap> {
    value: &'value Value,
    heap: &'heap ObjectHeap,
}

impl Value {
    pub fn print_with_heap<'value, 'heap>(
        &'value self,
        heap: &'heap ObjectHeap,
    ) -> ValueHeapDisplay<'value, 'heap> {
        ValueHeapDisplay { value: self, heap }
    }
}

impl<'value, 'heap> Display for ValueHeapDisplay<'value, 'heap> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Object(key) => {
                let obj = self
                    .heap
                    .get_object(*key);
                obj.unwrap().fmt(f)
            }
            val => val.fmt(f),
        }
    }
}

