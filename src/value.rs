use crate::{errors::BaseError, interpreter::{Memory, RegIndex, ValueResult}, parser::ASTNode};


#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Number(f64),
    Bool(bool),
    String(String), //not usable yet
    Builtin(String),
    Function {arg_names: Vec<String>, code: Box<ASTNode>, scope_id: RegIndex},
    Array(Vec<RegIndex>),
}

impl Value {

    pub fn to_str(&self, memory: &Memory) -> String {
        match self {
            Value::Null => String::from("Null"),
            Value::Number(value) => value.to_string(),
            Value::Bool(value) => if *value { String::from("True") } else { String::from("False") },
            Value::String(value) => format!("'{}'",value),
            Value::Builtin(name) => format!("<builtin: {}>", name),
            Value::Function { arg_names: _, code: _, scope_id: _ } => String::from("|...| {...}"),
            Value::Array(arr) => {
                let mut str_vec = Vec::new();
                for i in arr {
                    str_vec.push(memory.get(*i).to_str(memory));
                }
                format!("[{}]",str_vec.join(","))
            },
        }
    }

    pub fn plus(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) =>
                Ok(Value::Number( *v1 + v2 )),
            _ => Err(BaseError::InterpreterError("Operation '+' not defined for types".to_string()))
        }
    }
    pub fn minus(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) =>
                Ok(Value::Number( *v1 - v2 )),
            _ => Err(BaseError::InterpreterError("Operation '-' not defined for types".to_string()))
        }
    }
    pub fn mult(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) =>
                Ok(Value::Number( *v1 * v2 )),
            _ => Err(BaseError::InterpreterError("Operation '*' not defined for types".to_string()))
        }
    }
    pub fn div(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) =>
                Ok(Value::Number( *v1 / v2 )),
            _ => Err(BaseError::InterpreterError("Operation '/' not defined for types".to_string()))
        }
    }
    pub fn rem(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) =>
                Ok(Value::Number( *v1 % v2 )),
            _ => Err(BaseError::InterpreterError("Operation '%' not defined for types".to_string()))
        }
    }
    pub fn pow(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) =>
                Ok(Value::Number(f64::powf(*v1,*v2))),
            _ => Err(BaseError::InterpreterError("Operation '^' not defined for types".to_string()))
        }
    }
    pub fn neg(&self) -> ValueResult {
        match self {
            Value::Number(value) => Ok(Value::Number(-value)),
            _ => Err(BaseError::InterpreterError("Unary operation '-' not defined for type".to_string()))
        }
    }
    pub fn give(&self) -> ValueResult {
        match self {
            Value::Number(value) => Ok(Value::Number(*value)),
            _ => Err(BaseError::InterpreterError("Unary operation '+' not defined for type (flushed emoji)".to_string()))
        }
    }
    pub fn not(&self) -> ValueResult {
        match self {
            Value::Bool(value) => Ok(Value::Bool(!value)),
            _ => Err(BaseError::InterpreterError("Unary operation '!' not defined for type".to_string()))
        }
    }

    
    pub fn eq(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Null, Value::Null) => Ok(Value::Bool( true )),
            (Value::Number(v1), Value::Number(v2)) => Ok(Value::Bool( *v1 == *v2 )),
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool( *v1 == *v2 )),
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool( *v1 == *v2 )),
            _ => Err(BaseError::InterpreterError("Operation '==' not defined for types".to_string()))
        }
    }
    pub fn neq(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Null, Value::Null) => Ok(Value::Bool( false )),
            (Value::Number(v1), Value::Number(v2)) => Ok(Value::Bool( *v1 != *v2 )),
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool( *v1 != *v2 )),
            (Value::String(v1), Value::String(v2)) => Ok(Value::Bool( *v1 != *v2 )),
            _ => Err(BaseError::InterpreterError("Operation '!=' not defined for types".to_string()))
        }
    }
    pub fn gr(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) => Ok(Value::Bool( *v1 > *v2 )),
            _ => Err(BaseError::InterpreterError("Operation '!=' not defined for types".to_string()))
        }
    }
    pub fn greq(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) => Ok(Value::Bool( *v1 >= *v2 )),
            _ => Err(BaseError::InterpreterError("Operation '!=' not defined for types".to_string()))
        }
    }
    pub fn sm(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) => Ok(Value::Bool( *v1 < *v2 )),
            _ => Err(BaseError::InterpreterError("Operation '!=' not defined for types".to_string()))
        }
    }
    pub fn smeq(&self, other: &Value) -> ValueResult {
        match (self, other) {
            (Value::Number(v1), Value::Number(v2)) => Ok(Value::Bool( *v1 <= *v2 )),
            _ => Err(BaseError::InterpreterError("Operation '!=' not defined for types".to_string()))
        }
    }

    pub fn to_bool(&self) -> Result<bool, BaseError> {
        match self {
            Value::Bool(result) => Ok(*result),
            _ => Err(BaseError::InterpreterError("Cannot convert to boolean".to_string()))
        }
    }

    pub fn sin(&self) -> ValueResult {
        match self {
            Value::Number(value) => Ok(Value::Number(value.sin())),
            _ => Err(BaseError::InterpreterError("Expected number for argument".to_string()))
        }
    }
    pub fn cos(&self) -> ValueResult {
        match self {
            Value::Number(value) => Ok(Value::Number(value.cos())),
            _ => Err(BaseError::InterpreterError("Expected number for argument".to_string()))
        }
    }
    pub fn tan(&self) -> ValueResult {
        match self {
            Value::Number(value) => Ok(Value::Number(value.tan())),
            _ => Err(BaseError::InterpreterError("Expected number for argument".to_string()))
        }
    }

}





