use std::{collections::HashMap};

use crate::{errors::{BaseError}, lexer::Token, parser::ASTNode, value::Value};

pub type RegIndex = usize;
#[derive(Clone, Debug)]
pub enum NodeResult {
    VarName(String),
    Value(Value),
}
type ExecuteResult = Result<NodeResult, BaseError>;
pub type ValueResult = Result<Value, BaseError>;

fn ret_value(value: Value) -> ExecuteResult {
    Ok( NodeResult::Value( value ) )
}

fn derive_scope(scope_id: RegIndex, scopes: &mut ScopeList) -> RegIndex {
    scopes.counter += 1;
    scopes.register.insert( scopes.counter, Scope::new_child(scope_id) );
    scopes.counter
}

#[derive(Debug)]
pub struct Scope {
    parent_id: Option<RegIndex>,
    vars: HashMap<String, RegIndex>,
}

#[derive(Debug)]
pub struct Memory {
    counter: RegIndex,
    register: HashMap<RegIndex, Value>,
}

#[derive(Debug)]
pub struct ScopeList {
    counter: RegIndex,
    pub register: HashMap<RegIndex, Scope>,
}

impl Memory {
    pub fn new() -> Self {
        return Memory {counter: 0, register: HashMap::new()};
    }

    pub fn add(&mut self, value: Value) {
        self.counter += 1;
        self.register.insert(self.counter, value);
    }
    pub fn set(&mut self, value: Value, id: RegIndex) {
        self.register.insert(id, value);
    }
    /*
    pub fn get(&mut self, id: RegIndex) -> Value {
        self.register.get(&id).unwrap().clone()
    }
    */
}

impl ScopeList {
    pub fn new() -> Self {
        let mut register = HashMap::new();
        register.insert(0, Scope::new());
        return ScopeList {counter: 0, register }
    }

    pub fn get_var_id(&self, name: String, scope_id: RegIndex) -> Option<RegIndex> {
        if let Some(value) = self.register.get(&scope_id).unwrap().vars.get(&name) {
            return Some(*value);
        } else {
            match self.register.get(&scope_id).unwrap().parent_id {
                Some(id) => self.get_var_id(name, id),
                None => None,
            }
        }
    }

    pub fn set_var(&mut self, name: String, scope_id: RegIndex, memory: &mut Memory, value: &Value, first_call: bool) -> bool {
        if let Some(id) = self.register.get(&scope_id).unwrap().vars.get(&name) {
            memory.set(value.clone(), *id);
            return true
        }
        if let Some(parent_id) = self.register.get(&scope_id).unwrap().parent_id {
            let success = self.set_var(name.clone(), parent_id, memory, value, false);
            if success {return true;}
        }
        if first_call {
            memory.add(value.clone());
            self.register.get_mut(&scope_id).unwrap().vars.insert(name, memory.counter);
            return true
        }
        return false
    }

    pub fn set_var_local(&mut self, name: String, scope_id: RegIndex, memory: &mut Memory, value: &Value) -> bool {
        memory.add(value.clone());
        self.register.get_mut(&scope_id).unwrap().vars.insert(name, memory.counter);
        return true
    }

}


impl Scope {
    pub fn new() -> Self {
        return Scope {parent_id: None, vars: HashMap::new()};
    }
    pub fn new_child(parent_id: RegIndex) -> Self {
        return Scope {parent_id: Some(parent_id), vars: HashMap::new()};
    }
}

fn extract(node_result: NodeResult, scope_id: RegIndex, memory: &mut Memory, scopes: &mut ScopeList) -> ValueResult {
    match node_result {
        NodeResult::VarName(name) => match scopes.get_var_id(name.clone(), scope_id) {
            Some(id) => Ok(memory.register.get(&id).unwrap().clone()),
            None => Err(BaseError::InterpreterError(format!("Unknown variable: {}", name).to_string())),
        }
        NodeResult::Value(value) => Ok(value)
    }
}

macro_rules! extracute {
    ( $funny_node:expr, $the_scope_id:expr, $the_memory:expr, $the_scopes:expr ) => {
        {
            let bruh: RegIndex = $the_scope_id;
            extract( execute($funny_node, bruh, $the_memory, $the_scopes)?, bruh, $the_memory, $the_scopes)?
        }
    };
}

macro_rules! error_out {
    ( $message:expr ) => {
        { return Err(BaseError::InterpreterError($message.to_string())); }
    }
}

pub fn start_execute(node: &ASTNode, scopes: &mut ScopeList, memory: &mut Memory) -> ExecuteResult {

    execute(node, 0, memory, scopes)

}

fn execute(node: &ASTNode, scope_id: RegIndex, memory: &mut Memory, scopes: &mut ScopeList) -> ExecuteResult {
    ret_value( match node {
        ASTNode::Value { value } => extracute!( value, scope_id, memory, scopes ),
        ASTNode::Num { value } => Value::Number(*value),
        ASTNode::Unary { op, value } => {
            let value = extracute!(value, scope_id, memory, scopes);
            match op {
                crate::lexer::Token::Plus => value.give()?,
                crate::lexer::Token::Minus => value.neg()?,
                crate::lexer::Token::Not => value.not()?,
                _ => error_out!("Non '+','-','!' unary operation")
            }
        },
        ASTNode::Op { left, op, right } => {
            match op {
                Token::Plus | Token::Minus | Token::Mult | Token::Div | Token::Mod | Token::Pow | Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq | Token::Eq | Token::NotEq => {
                    let left = extracute!(left, scope_id, memory, scopes);
                    let right = extracute!(right, scope_id, memory, scopes);

                    match op {
                        Token::Plus => left.plus(right)?,
                        Token::Minus => left.minus(right)?,
                        Token::Mult => left.mult(right)?,
                        Token::Div => left.div(right)?,
                        Token::Mod => left.rem(right)?,
                        Token::Pow => left.pow(right)?,
                        Token::Greater => left.gr(right)?,
                        Token::GreaterEq => left.greq(right)?,
                        Token::Lesser => left.sm(right)?,
                        Token::LesserEq => left.smeq(right)?,
                        Token::Eq => left.eq(right)?,
                        Token::NotEq => left.neq(right)?,
                        _ => unimplemented!(),
                    }

                }
                Token::PlusEq | Token::MinusEq | Token::MultEq | Token::DivEq | Token::ModEq | Token::PowEq  => {
                    let right_eval = extracute!(right, scope_id, memory, scopes);
                    let left_raw = execute(left, scope_id, memory, scopes)?;
                    match left_raw.clone() {
                        NodeResult::VarName(name) => {
                            let value = extract(left_raw, scope_id, memory, scopes)?;
                            let new_value ;
                            match op {
                                Token::PlusEq => {new_value = value.plus(right_eval)?},
                                Token::MinusEq => {new_value = value.minus(right_eval)?},
                                Token::MultEq => {new_value = value.mult(right_eval)?},
                                Token::DivEq => {new_value = value.div(right_eval)?},
                                Token::ModEq => {new_value = value.rem(right_eval)?},
                                Token::PowEq => {new_value = value.pow(right_eval)?},
                                _ => unimplemented!(),
                            }
                            scopes.set_var(name, scope_id, memory, &new_value, true);
                            new_value
                        }
                        NodeResult::Value(_) => error_out!("Expected variable name")
                    }
                },
                Token::And => {
                    if !extracute!(left, scope_id, memory, scopes).to_bool()?
                        { return ret_value( Value::Bool(false) ) }
                    if !extracute!(right, scope_id, memory, scopes).to_bool()?
                        { return ret_value( Value::Bool(false) ) }
                    Value::Bool(true)
                }
                Token::Or => {
                    if extracute!(left, scope_id, memory, scopes).to_bool()?
                        { return ret_value( Value::Bool(true) ) }
                    if extracute!(right, scope_id, memory, scopes).to_bool()?
                        { return ret_value( Value::Bool(true) ) }
                    Value::Bool(false)
                }
                Token::Assign => {
                    let right_eval = extracute!(right, scope_id, memory, scopes);
                    let left_raw = execute(left, scope_id, memory, scopes)?;
                    match left_raw {
                        NodeResult::VarName(name) => {
                            scopes.set_var(name, scope_id, memory, &right_eval, true);
                            right_eval
                        }
                        NodeResult::Value(_) => error_out!("Expected variable name")
                    }
                }
                Token::LocalAssign => {
                    let right_eval = extracute!(right, scope_id, memory, scopes);
                    let left_raw = execute(left, scope_id, memory, scopes)?;
                    match left_raw {
                        NodeResult::VarName(name) => {
                            scopes.set_var_local(name, scope_id, memory, &right_eval);
                            right_eval
                        }
                        NodeResult::Value(_) => error_out!("Expected variable name")
                    }
                }
                _ => todo!(),
            }
        },
        ASTNode::Var { name } => {return Ok( NodeResult::VarName(name.clone()) )},
        ASTNode::StatementList { statements } => {
            let mut last = Value::Null;
            for i in statements {
                last = extracute!( i, scope_id, memory, scopes );
            }
            last
        },
        ASTNode::If { conds, if_none } => {
            for i in conds {
                if extracute!(&i.0, scope_id, memory, scopes).to_bool()? {
                    return ret_value(extracute!(&i.1, derive_scope(scope_id, scopes), memory, scopes))
                }
            }

            match &**if_none {
                Some(node) => extracute!(node, scope_id, memory, scopes),
                None => Value::Null,
            }
        },
        ASTNode::While { cond, code } => {
            let mut last = Value::Null;
            loop {
                if extracute!(cond, scope_id, memory, scopes).to_bool()? {
                    last = extracute!(code, derive_scope(scope_id, scopes), memory, scopes);
                } else { return ret_value( last ); }
            }
        },
        ASTNode::Constant { value } => value.clone(),
        ASTNode::Block { code } =>
            extracute!(code, derive_scope(scope_id, scopes), memory, scopes),
        ASTNode::Func { code, arg_names } => {
            Value::Function {arg_names: arg_names.clone(), code: code.clone(), scope_id}
        }
        ASTNode::Call { base, args } => {
            match extracute!(base, scope_id, memory, scopes) {
                Value::Builtin(name) => {
                    match &name[..] {
                        "sin" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( extracute!(i, scope_id, memory, scopes) );
                            }
                            converted_args[0].sin()?
                        }
                        "cos" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( extracute!(i, scope_id, memory, scopes) );
                            }
                            converted_args[0].cos()?
                        }
                        "tan" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( extracute!(i, scope_id, memory, scopes) );
                            }
                            converted_args[0].tan()?
                        }
                        "print" => {
                            for i in args {
                                println!("{}", extracute!(i, scope_id, memory, scopes).to_str() );
                            }
                            
                            Value::Null
                        }
                        "memtest" => {
                            println!("{:#?}",memory);
                            println!("{:#?}",scopes);
                            
                            Value::Null
                        }
                        _ => unimplemented!(),
                    }
                }
                Value::Function { arg_names, code, scope_id: def_scope } => {
                    if args.len() != arg_names.len() {
                        error_out!(format!{"Expected {} argument(s)", arg_names.len()})
                    }
                    let mut converted_args: Vec<Value> = Vec::new();
                    for i in args {
                        converted_args.push( extracute!(i, scope_id, memory, scopes) );
                    }
                    
                    let run_scope = derive_scope(def_scope, scopes);
                    for (i, j) in arg_names.iter().zip(converted_args.iter()) {
                        scopes.set_var(i.clone(), run_scope, memory, j, true);
                    }

                    extracute!(&code, run_scope, memory, scopes)
                }
                _ => error_out!("Invalid base for call")
            }
        }
        ASTNode::Array {values} => {

            let mut eval_values : Vec<Value> = Vec::new();
            for i in values {
                eval_values.push( extracute!(i, scope_id, memory, scopes) );
            }
            Value::Array(eval_values)
        }
        ASTNode::Index { base, index } => {
            let i = match extracute!(index, scope_id, memory, scopes) {
                Value::Number(value) => value.floor(),
                _ => error_out!("Cannot index with type")
            } as isize;
            match extracute!(base, scope_id, memory, scopes) {
                Value::Array(arr) => if i >= arr.len() as isize || i < 0 {error_out!("Index out of bounds")} else {arr[i as usize].clone()},
                _ => error_out!("Type cannot be indexed")
            }
        }
    } )
}



