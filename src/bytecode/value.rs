use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64)
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{num}"),
        }
    }
}