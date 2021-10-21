use std::{collections::{HashMap, HashSet}};

use crate::{errors::{BaseError}, lexer::Token, parser::ASTNode, value::Value};

pub type RegIndex = usize;

pub type ValueResult = Result<Value, BaseError>;
/*
fn ret_value(value: Value) -> ExecuteResult {
    Ok( NodeResult::Value( value ) )
}
*/
fn derive_scope(scope_id: RegIndex, caller_id: RegIndex, scopes: &mut ScopeList) -> RegIndex {
    scopes.counter += 1;
    scopes.register.insert( scopes.counter, Scope {parent_id: Some(scope_id), caller_id: Some(caller_id), vars: HashMap::new() } );
    scopes.counter
}

#[derive(Debug)]
pub struct Scope {
    parent_id: Option<RegIndex>,
    caller_id: Option<RegIndex>,
    vars: HashMap<String, RegIndex>,
}

#[derive(Debug)]
pub struct Memory {
    counter: RegIndex,
    register: HashMap<RegIndex, Value>,
    protected: Vec<Vec<RegIndex>>,
}

#[derive(Debug)]
pub struct ScopeList {
    counter: RegIndex,
    pub register: HashMap<RegIndex, Scope>,
}

#[derive(Debug)]
pub struct CollectTracker {
    marked_scopes: HashSet<RegIndex>,
    marked_values: HashSet<RegIndex>,
}

impl CollectTracker {
    fn new(memory: &Memory, scopes: &ScopeList) -> Self {
        let mut marked_scopes = HashSet::new();
        let mut marked_values = HashSet::new();
        for (i, _) in &memory.register {
            marked_values.insert(*i);
        }
        for (i, _) in &scopes.register {
            marked_scopes.insert(*i);
        }
        Self {
            marked_values,
            marked_scopes,
        }
    }
}

fn get_value_references(value: &Value, memory: &Memory, scopes: &ScopeList) -> (Vec<RegIndex>, Vec<RegIndex>) {
    match value {
        Value::Function { arg_names: _, code: _, scope_id } => (vec![], vec![*scope_id]),
        Value::Array (arr) => {
            let mut res_value_ids = Vec::new();
            let mut res_scope_ids = Vec::new();
            for i in arr {
                res_value_ids.push(*i);
                let (mut value_ids, mut scope_ids) = get_value_references(memory.register.get(i).unwrap(), memory, scopes);
                res_value_ids.append(&mut value_ids);
                res_scope_ids.append(&mut scope_ids);
            }
            (res_value_ids, res_scope_ids)
        }
        _ => (vec![], vec![])
    }
}

impl Memory {
    pub fn new() -> Self {
        return Memory {counter: 0, register: HashMap::new(), protected: Vec::new()};
    }

    pub fn add(&mut self, value: Value) -> RegIndex {
        self.counter += 1;
        self.register.insert(self.counter, value);
        self.counter
    }
    pub fn set(&mut self, value: Value, id: RegIndex) {
        self.register.insert(id, value);
    }
    
    pub fn get(&self, id: RegIndex) -> &Value {
        self.register.get(&id).unwrap()
    }
    

    pub fn new_protected(&mut self) {
        self.protected.push(Vec::new());
    }
    pub fn pop_protected(&mut self) {
        //println!("{:?}",self.protected);
        self.protected.pop();
    }
    pub fn protect(&mut self, value: Value) -> Value {
        self.add(value);
        self.protected
            .last_mut()
            .unwrap()
            .push(self.counter);
        self.get(self.counter).clone()
    }
    pub fn protect_id(&mut self, value: Value) -> RegIndex {
        self.add(value);
        self.protected
            .last_mut()
            .unwrap()
            .push(self.counter);
        self.counter
    }



    pub fn collect(&mut self, scopes: &mut ScopeList, scope_id: RegIndex) {
        let mut tracker = CollectTracker::new(self, scopes);
        self.mark(scopes, scope_id, &mut tracker);
        //println!("{:#?}",tracker);
        for vec in &self.protected {
            for var_id in vec {
                //println!("{:#?}",var_id);
                tracker.marked_values.remove(var_id);
                let (value_ids, scope_ids) = get_value_references(self.register.get(&var_id).unwrap(), self, scopes);
                //println!("{:?}; {:?}",value_ids, scope_ids);
                for i in scope_ids {
                    if tracker.marked_scopes.contains(&i) {
                        self.mark(scopes, i, &mut tracker);
                    }
                }
                for i in value_ids {
                    tracker.marked_values.remove(&i);
                }
            }
        }
        //println!("{:#?}",tracker);
        for i in tracker.marked_scopes {
            scopes.register.remove(&i);
        }
        for i in tracker.marked_values {
            self.register.remove(&i);
        }
    }

    pub fn mark(&self, scopes: &mut ScopeList, scope_id: RegIndex, tracker: &mut CollectTracker) {
        let mut var_check_ids = Vec::new();
        tracker.marked_scopes.remove(&scope_id);
        for (_, var_id) in &scopes.register.get(&scope_id).unwrap().vars {
            var_check_ids.push(*var_id);
        }
        for var_id in var_check_ids {
            tracker.marked_values.remove(&var_id);
            let (value_ids, scope_ids) = get_value_references (self.register.get(&var_id).unwrap(), self, scopes);
            for i in scope_ids {
                if tracker.marked_scopes.contains(&i) {
                    self.mark(scopes, i, tracker);
                }
            }
            for i in value_ids {
                tracker.marked_values.remove(&i);
            }
        }
        let parent_id = scopes.register.get(&scope_id).unwrap().parent_id;
        match parent_id {
            Some(id) => self.mark(scopes, id, tracker),
            None => (),
        }
        let caller_id = scopes.register.get(&scope_id).unwrap().caller_id;
        match caller_id {
            Some(id) => self.mark(scopes, id, tracker),
            None => (),
        }
    }

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
        return Scope {parent_id: None, caller_id: None, vars: HashMap::new()};
    }
}

/*
macro_rules! extracute {
    ( $funny_node:expr, $the_scope_id:expr, $the_memory:expr, $the_scopes:expr ) => {
        {
            let bruh: RegIndex = $the_scope_id;
            extract( execute($funny_node, bruh, $the_memory, $the_scopes)?, bruh, $the_memory, $the_scopes)?
        }
    };
}
*/


macro_rules! protecute {
    ( $funny_node:expr, $the_scope_id:expr, $the_memory:expr, $the_scopes:expr ) => {
        {
            let bruh = execute($funny_node, $the_scope_id, $the_memory, $the_scopes)?;
            $the_memory.protect(bruh)
        }
    };
}

macro_rules! protecute_id {
    ( $funny_node:expr, $the_scope_id:expr, $the_memory:expr, $the_scopes:expr ) => {
        {
            let bruh = execute($funny_node, $the_scope_id, $the_memory, $the_scopes)?;
            $the_memory.protect_id(bruh)
        }
    };
}



macro_rules! error_out {
    ( $message:expr ) => {
        { return Err(BaseError::InterpreterError($message.to_string())); }
    }
}

pub fn start_execute(node: &ASTNode, scopes: &mut ScopeList, memory: &mut Memory) -> ValueResult {

    memory.protected.clear();
    execute(node, 0, memory, scopes)

}

fn execute(node: &ASTNode, scope_id: RegIndex, memory: &mut Memory, scopes: &mut ScopeList) -> ValueResult {
    //println!("\n\n{:#?}\nscope_id: {},\n{:#?}\n{:#?}",memory,scope_id,scopes,node);
    //println!("{:?}", memory.protected);
    memory.collect(scopes, scope_id);

    memory.new_protected();

    let val = match node {
        ASTNode::Value { value } => execute( value, scope_id, memory, scopes )?,
        ASTNode::Num { value } => Value::Number(*value),
        ASTNode::Unary { op, value } => {
            let value = protecute!(value, scope_id, memory, scopes);
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
                    let left = protecute!(left, scope_id, memory, scopes);
                    let right = protecute!(right, scope_id, memory, scopes);

                    match op {
                        Token::Plus => left.plus(&right)?,
                        Token::Minus => left.minus(&right)?,
                        Token::Mult => left.mult(&right)?,
                        Token::Div => left.div(&right)?,
                        Token::Mod => left.rem(&right)?,
                        Token::Pow => left.pow(&right)?,
                        Token::Greater => left.gr(&right)?,
                        Token::GreaterEq => left.greq(&right)?,
                        Token::Lesser => left.sm(&right)?,
                        Token::LesserEq => left.smeq(&right)?,
                        Token::Eq => left.eq(&right)?,
                        Token::NotEq => left.neq(&right)?,
                        _ => unimplemented!(),
                    }

                }
                
                Token::PlusEq | Token::MinusEq | Token::MultEq | Token::DivEq | Token::ModEq | Token::PowEq  => {
                    let right_eval = execute(right, scope_id, memory, scopes)?;
                    match (**left).clone() {
                        ASTNode::Var {name} => {
                            let value = execute(left, scope_id, memory, scopes)?;
                            let new_value = match op {
                                Token::PlusEq => value.plus(&right_eval)?,
                                Token::MinusEq => value.minus(&right_eval)?,
                                Token::MultEq => value.mult(&right_eval)?,
                                Token::DivEq => value.div(&right_eval)?,
                                Token::ModEq => value.rem(&right_eval)?,
                                Token::PowEq => value.pow(&right_eval)?,
                                _ => unimplemented!(),
                            };
                            scopes.set_var(name, scope_id, memory, &new_value, true);
                            new_value
                        }
                        _ => error_out!("Expected variable name")
                    }
                },
                Token::Assign => {
                    let right_eval = execute(right, scope_id, memory, scopes)?;
                    match (**left).clone() {
                        ASTNode::Var { name } => {
                            scopes.set_var(name, scope_id, memory, &right_eval, true);
                            right_eval
                        },
                        _ => error_out!("Expected variable name")
                    }
                }
                Token::LocalAssign => {
                    let right_eval = execute(right, scope_id, memory, scopes)?;
                    match (**left).clone() {
                        ASTNode::Var { name } => {
                            scopes.set_var_local(name, scope_id, memory, &right_eval);
                            right_eval
                        },
                        _ => error_out!("Expected variable name")
                    }
                }
                
                Token::And => {
                    if !execute(left, scope_id, memory, scopes)?.to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(false) )
                    }
                    if !execute(right, scope_id, memory, scopes)?.to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(false) )
                    }
                    Value::Bool(true)
                }
                Token::Or => {
                    if execute(left, scope_id, memory, scopes)?.to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(true) )
                    }
                    if execute(right, scope_id, memory, scopes)?.to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(true) )
                    }
                    Value::Bool(false)
                }
                _ => todo!(),
            }
        },
        ASTNode::Var { name } => match scopes.get_var_id(name.clone(), scope_id) {
            Some(id) => memory.register.get(&id).unwrap().clone(),
            None => error_out!("Unknown variable"),
        },
        ASTNode::StatementList { statements } => {
            let mut last = Value::Null;
            for i in statements {
                last = execute( i, scope_id, memory, scopes )?;
            }
            last
        },
        ASTNode::If { conds, if_none } => {
            for i in conds {
                if execute(&i.0, scope_id, memory, scopes)?.to_bool()? {
                    memory.pop_protected();
                    return Ok( execute(&i.1, derive_scope(scope_id, scope_id, scopes), memory, scopes)? )
                }
            }

            match &**if_none {
                Some(node) => execute(node, scope_id, memory, scopes)?,
                None => Value::Null,
            }
        },
        ASTNode::While { cond, code } => {
            let mut last = Value::Null;
            loop {
                if execute(cond, scope_id, memory, scopes)?.to_bool()? {
                    last = execute(code, derive_scope(scope_id, scope_id, scopes), memory, scopes)?;
                } else { memory.pop_protected(); return Ok( last ) ; }
            }
        },
        ASTNode::Constant { value } => value.clone(),
        ASTNode::Block { code } =>
            execute(code, derive_scope(scope_id, scope_id, scopes), memory, scopes)?,
        ASTNode::Func { code, arg_names } => {
            Value::Function {arg_names: arg_names.clone(), code: code.clone(), scope_id}
        }
        ASTNode::Call { base, args } => {
            match execute(base, scope_id, memory, scopes)? {
                Value::Builtin(name) => {
                    match &name[..] {
                        "sin" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( protecute!(i, scope_id, memory, scopes) );
                            }
                            converted_args[0].sin()?
                        }
                        "cos" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( protecute!(i, scope_id, memory, scopes) );
                            }
                            converted_args[0].cos()?
                        }
                        "tan" => {
                            if args.len() != 1 {error_out!("Expected 1 argument")}
                            let mut converted_args: Vec<Value> = Vec::new();
                            for i in args {
                                converted_args.push( protecute!(i, scope_id, memory, scopes) );
                            }
                            converted_args[0].tan()?
                        }
                        "print" => {
                            if args.len() == 0 {
                                print!("");
                            } else {
                                let mut s = String::from("");
                                for i in args {
                                    s.push_str( &format!("{} ", execute(i, scope_id, memory, scopes)?.to_str(memory)) );
                                }
                                print!("{}",s);
                            }
                            Value::Null
                        }
                        "println" => {
                            if args.len() == 0 {
                                println!("");
                            } else {
                                let mut s = String::from("");
                                for i in args {
                                    s.push_str( &format!("{} ", execute(i, scope_id, memory, scopes)?.to_str(memory)) );
                                }
                                println!("{}",s);
                            }
                            Value::Null
                        }
                        "memtest" => {
                            println!("{:#?}",memory);
                            println!("{:#?}",scopes);
                            
                            Value::Null
                        }
                        "collect" => {
                            memory.collect(scopes, scope_id);
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
                        converted_args.push( protecute!(i, scope_id, memory, scopes) );
                    }
                    
                    let run_scope = derive_scope(def_scope, scope_id, scopes);
                    for (i, j) in arg_names.iter().zip(converted_args.iter()) {
                        scopes.set_var(i.clone(), run_scope, memory, j, true);
                    }

                    execute(&code, run_scope, memory, scopes)?
                }
                _ => error_out!("Invalid base for call")
            }
        }
        ASTNode::Array {values} => {

            let mut eval_values = Vec::new();
            for i in values {
                eval_values.push( protecute_id!(i, scope_id, memory, scopes) );
            }
            Value::Array(eval_values)
        }
        ASTNode::Index { base, index } => {
            let i = match execute(index, scope_id, memory, scopes)? {
                Value::Number(value) => value.floor(),
                _ => error_out!("Cannot index with type")
            } as isize;
            match execute(base, scope_id, memory, scopes)? {
                Value::Array(arr) => if i >= arr.len() as isize || i < 0 {
                    error_out!("Index out of bounds")
                } else { memory.get(arr[i as usize]).clone()},
                _ => error_out!("Type cannot be indexed")
            }
        }
    };
    memory.pop_protected();
    Ok( val )
}



