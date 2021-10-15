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

#[derive(Clone, Debug)]
pub struct Scope {
    parent: Box<Option<Scope>>,
    vars: HashMap<String, RegIndex>,
}

pub struct Memory {
    counter: RegIndex,
    register: HashMap<RegIndex, Value>,
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


impl Scope {
    pub fn new() -> Self {
        return Scope {parent: Box::new(None), vars: HashMap::new()};
    }

    pub fn derive(&self) -> Scope {
        return Scope {parent: Box::new(Some(self.clone())), vars: HashMap::new()}
    }

    fn get_id(&self, name: String) -> Option<RegIndex> {
        if let Some(value) = self.vars.get(&name) {
            return Some(*value);
        } else {
            match &*self.parent {
                Some(parent) => parent.get_id(name),
                None => None,
            }
        }
    }
    pub fn set_var(&mut self, name: String, memory: &mut Memory, value: &Value, first_call: bool) -> bool {
        if let Some(id) = self.vars.get(&name) {
            memory.set(value.clone(), *id);
            return true
        }
        if let Some(parent) = &mut *self.parent {
            let success = parent.set_var(name.clone(), memory, value, false);
            if success {return true}
        }
        if first_call {
            memory.add(value.clone());
            self.vars.insert(name, memory.counter);
            return true
        }
        return false
    }
    fn set_var_local(&mut self, name: String, memory: &mut Memory, value: &Value) -> bool {
        memory.add(value.clone());
        self.vars.insert(name, memory.counter);
        return true
    }
}

fn extract(node_result: NodeResult, scope: &Scope, memory: &mut Memory) -> ValueResult {
    match node_result {
        NodeResult::VarName(name) => match scope.get_id(name.clone()) {
            Some(id) => Ok(memory.register.get(&id).unwrap().clone()),
            None => Err(BaseError::InterpreterError(format!("Unknown variable: {}", name).to_string())),
        }
        NodeResult::Value(value) => Ok(value)
    }
}

macro_rules! extracute {
    ( $funny_node:expr, $the_scope:expr, $the_memory:expr ) => {
        {
            let bruh: &mut Scope = $the_scope;
            extract( execute($funny_node, bruh, $the_memory)?, bruh, $the_memory)?
        }
    };
}

macro_rules! error_out {
    ( $message:expr ) => {
        { return Err(BaseError::InterpreterError($message.to_string())); }
    }
}

pub fn start_execute(node: &ASTNode, scope: &mut Scope, memory: &mut Memory) -> ExecuteResult {

    execute(node, scope, memory)

}

fn execute(node: &ASTNode, scope: &mut Scope, memory: &mut Memory) -> ExecuteResult {
    ret_value( match node {
        ASTNode::Value { value } => extracute!( value, scope, memory ),
        ASTNode::Num { value } => Value::Number(*value),
        ASTNode::Unary { op, value } => {
            let value = extracute!(value, scope, memory);
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
                    let left = extracute!(left, scope, memory);
                    let right = extracute!(right, scope, memory);

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
                    let right_eval = extracute!(right,scope,memory);
                    let left_raw = execute(left, scope, memory)?;
                    match left_raw.clone() {
                        NodeResult::VarName(name) => {
                            let value = extract(left_raw, scope, memory)?;
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
                            scope.set_var(name, memory, &new_value, true);
                            new_value
                        }
                        NodeResult::Value(_) => error_out!("Expected variable name")
                    }
                },
                Token::And => {
                    if !extracute!(left, scope, memory).to_bool()?
                        { return ret_value( Value::Bool(false) ) }
                    if !extracute!(right, scope, memory).to_bool()?
                        { return ret_value( Value::Bool(false) ) }
                    Value::Bool(true)
                }
                Token::Or => {
                    if extracute!(left, scope, memory).to_bool()?
                        { return ret_value( Value::Bool(true) ) }
                    if extracute!(right, scope, memory).to_bool()?
                        { return ret_value( Value::Bool(true) ) }
                    Value::Bool(false)
                }
                Token::Assign => {
                    let right_eval = extracute!(right,scope,memory);
                    let left_raw = execute(left, scope, memory)?;
                    match left_raw {
                        NodeResult::VarName(name) => {
                            scope.set_var(name, memory, &right_eval, true);
                            right_eval
                        }
                        NodeResult::Value(_) => error_out!("Expected variable name")
                    }
                }
                Token::LocalAssign => {
                    let right_eval = extracute!(right,scope,memory);
                    let left_raw = execute(left, scope, memory)?;
                    match left_raw {
                        NodeResult::VarName(name) => {
                            scope.set_var_local(name, memory, &right_eval);
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
                last = extracute!( i, scope, memory );
            }
            last
        },
        ASTNode::If { conds, if_none } => {
            for i in conds {
                if extracute!(&i.0, scope, memory).to_bool()? {
                    return ret_value(extracute!(&i.1, &mut scope.derive(), memory))
                }
            }

            match &**if_none {
                Some(node) => extracute!(node, scope, memory),
                None => Value::Null,
            }
        },
        ASTNode::While { cond, code } => {
            let mut last = Value::Null;
            loop {
                if extracute!(cond, scope, memory).to_bool()? {
                    last = extracute!(code, &mut scope.derive(), memory);
                } else { return ret_value( last ); }
            }
        },
        ASTNode::Constant { value } => value.clone(),
        ASTNode::Block { code } =>
            extracute!(code, &mut scope.derive(), memory),
        ASTNode::Func { code, arg_names } => {
            Value::Function {arg_names: arg_names.clone(), code: code.clone(), scope: scope.clone()}
        }
        ASTNode::Call { base, args } => {
            match extracute!(base, scope, memory) {
                Value::Builtin(name) => {
                    match &name[..] {
                        "sin" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( extracute!(i, scope, memory) );
                            }
                            converted_args[0].sin()?
                        }
                        "cos" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( extracute!(i, scope, memory) );
                            }
                            converted_args[0].cos()?
                        }
                        "tan" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( extracute!(i, scope, memory) );
                            }
                            converted_args[0].tan()?
                        }
                        "print" => {
                            for i in args {
                                println!("{:?}", extracute!(i, scope, memory) );
                            }
                            
                            Value::Null
                        }
                        _ => unimplemented!(),
                    }
                }
                Value::Function { arg_names, code, scope: def_scope } => {
                    if args.len() != arg_names.len() {
                        error_out!(format!{"Expected {} argument(s)", arg_names.len()})
                    }
                    let mut converted_args: Vec<Value> = Vec::new();
                    for i in args {
                        converted_args.push( extracute!(i, scope, memory) );
                    }
                    
                    let run_scope = &mut def_scope.derive();
                    for (i, j) in arg_names.iter().zip(converted_args.iter()) {
                        run_scope.set_var(i.clone(), memory, j, true);
                    }

                    extracute!(&code, run_scope, memory)
                }
                _ => error_out!("Invalid base for call")
            }
        }
    } )
}



