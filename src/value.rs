use std::fmt;

// Color type - RGB with 0-255 values
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn blend(&self, other: &Color) -> Color {
        Color {
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8,
        }
    }

    pub fn mix(&self, other: &Color, ratio: f32) -> Color {
        let ratio = ratio.clamp(0.0, 1.0);
        let inv_ratio = 1.0 - ratio;
        Color {
            r: (self.r as f32 * inv_ratio + other.r as f32 * ratio) as u8,
            g: (self.g as f32 * inv_ratio + other.g as f32 * ratio) as u8,
            b: (self.b as f32 * inv_ratio + other.b as f32 * ratio) as u8,
        }
    }

    pub fn add(&self, other: &Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }

    pub fn sub(&self, other: &Color) -> Color {
        Color {
            r: self.r.saturating_sub(other.r),
            g: self.g.saturating_sub(other.g),
            b: self.b.saturating_sub(other.b),
        }
    }

    pub fn scale(&self, factor: f32) -> Color {
        Color {
            r: (self.r as f32 * factor).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * factor).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * factor).clamp(0.0, 255.0) as u8,
        }
    }
}

// Continuation type
pub enum Continuation {
    Resume(Box<dyn Fn() -> Value + 'static>),
    Empty,
}

impl fmt::Debug for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Continuation::Resume(_) => write!(f, "Continuation::Resume"),
            Continuation::Empty => write!(f, "Continuation::Empty"),
        }
    }
}

// Future type for Tau language
#[derive(Debug, Clone)]
pub enum FutureState {
    Pending,
    Resolved(Box<Value>),
    Rejected(String),
}

// Value types
#[derive(Debug)]
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Unit,
    Color(Color),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Future(FutureState),
    Continuation(Box<Continuation>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Num(n) => Value::Num(*n),
            Value::Str(s) => Value::Str(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Unit => Value::Unit,
            Value::Color(c) => Value::Color(*c),
            Value::Array(a) => Value::Array(a.clone()),
            Value::Map(m) => Value::Map(m.clone()),
            Value::Future(f) => Value::Future(f.clone()),
            Value::Continuation(_) => Value::Unit,
        }
    }
}

impl Value {
    pub fn as_num(&self) -> Result<f64, String> {
        match self {
            Value::Num(n) => Ok(*n),
            _ => Err(format!("Expected number, got {:?}", self)),
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.add(b))),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::Array(result))
            }
            _ => Err(format!("Cannot add {:?} and {:?}", self, other)),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),
            (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.sub(b))),
            _ => Err(format!("Cannot subtract {:?} and {:?}", self, other)),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
            _ => Err(format!("Cannot multiply {:?} and {:?}", self, other)),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(_a), Value::Num(b)) if *b == 0.0 => Err("Division by zero".to_string()),
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a / b)),
            _ => Err(format!("Cannot divide {:?} and {:?}", self, other)),
        }
    }

    pub fn blend(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.blend(b))),
            _ => Err(format!("Cannot blend {:?} and {:?}", self, other)),
        }
    }

    pub fn scale(&self, factor: f32) -> Result<Value, String> {
        match self {
            Value::Color(c) => Ok(Value::Color(c.scale(factor))),
            _ => Err(format!("Cannot scale {:?}", self)),
        }
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Bool(a < b)),
            _ => Err(format!("Cannot compare {:?} and {:?}", self, other)),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Bool(a > b)),
            _ => Err(format!("Cannot compare {:?} and {:?}", self, other)),
        }
    }

    pub fn equals(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Bool((a - b).abs() < f64::EPSILON)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
            _ => Ok(Value::Bool(false)),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Num(n) => *n != 0.0,
            Value::Unit => false,
            _ => true,
        }
    }
}
