use crate::prelude::*;
use std::fmt;

//pub type NativeCall = fn(args: &[Value]) -> Value;

#[derive(Clone, Copy)]
pub struct NativeFunction(pub fn(&TheVM, &[Value]) -> Value);

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn>")
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum FunctionType {
    Script,
    Function,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectFunction {
    pub function_type: FunctionType,
    pub arity: usize,
    pub chunk: TheChunk,
    pub name: String,
}

impl ObjectFunction {
    pub fn new(name: String, function_type: FunctionType) -> Self {
        Self {
            function_type,
            arity: 0,
            chunk: TheChunk::new(),
            name,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Float(f32),
    String(String),
    Function(ObjectFunction),
    NativeFunction(NativeFunction),
}

impl Value {
    pub fn equal(&self, other: &Value) -> bool {
        match self {
            Value::Nil => other.is_nil(),
            Value::Bool(value) => Some(*value) == other.as_bool(),
            Value::Float(value) => Some(*value) == other.as_float(),
            Value::String(value) => {
                if let Some(other_string) = other.as_string() {
                    other_string == *value
                } else {
                    false
                }
            }
            Value::Function(_funcion) => false,
            Value::NativeFunction(_funcion) => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(value) => Some(value.to_string()),
            _ => None,
        }
    }

    pub fn print(&self) {
        match self {
            Value::Nil => println!("nil"),
            Value::Bool(value) => println!("{}", value), //if *value { println!("true") } else { println!("false"); },
            Value::Float(value) => println!("{}", value),
            Value::String(value) => println!("{}", value),
            Value::Function(value) => println!("{}", value.name),
            Value::NativeFunction(_value) => println!("native_function"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Values {
    pub values: Vec<Value>,

    pub count: usize,
    pub capacity: usize,
}

impl Default for Values {
    fn default() -> Self {
        Self::new()
    }
}

impl Values {
    pub fn new() -> Self {
        Self {
            values: vec![],
            count: 0,
            capacity: 0,
        }
    }

    /// Clears the chunk.
    pub fn clear(&mut self) {
        self.values.clear();
        self.count = 0;
        self.capacity = 0;
    }

    /// Resizes the chunk to the new size
    pub fn resize(&mut self, new_size: usize) {
        if new_size == 0 {
            self.clear();
        }

        self.values.resize(new_size, Value::Nil);
        self.capacity = new_size;
    }

    /// Writes an instruction to current location
    pub fn write(&mut self, value: Value) {
        if self.capacity < self.count + 1 {
            let capacity = if self.capacity < 8 {
                8
            } else {
                self.capacity * 2
            };
            self.resize(capacity);
        }
        self.values[self.count] = value;
        self.count += 1;
    }

    pub fn debug(&self, name: &str) {
        println!("== {} ==", name);
        for offset in 0..self.count {
            println!("{} {:?}", offset, self.values[offset]);
        }
    }
}
