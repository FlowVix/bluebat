use std::{collections::{HashMap, HashSet}, io::{self, Write}};

use crate::{errors::{BaseError}, lexer::Token, parser::ASTNode, value::Value};

pub type RegIndex = usize;

pub type ValueResult = Result<Value, BaseError>;

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
    last_amount: usize,
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

fn get_value_references(
    value: &Value,
    memory: &Memory,
    scopes: &ScopeList,
    value_ids: &mut Vec<RegIndex>,
    scope_ids: &mut Vec<RegIndex>
) {
    match value {
        Value::Function { arg_names: _, code: _, scope_id } => {
            if !scope_ids.contains(scope_id) {
                scope_ids.push(*scope_id);
            }
        },
        Value::Array (arr) => {
            for i in arr {
                if !value_ids.contains(i) {
                    value_ids.push(*i);
                    get_value_references(memory.register.get(i).unwrap(), memory, scopes, value_ids, scope_ids);
                }
            }
        }
        _ => (),
    }
}

impl Memory {
    pub fn new() -> Self {
        return Memory {counter: 0, register: HashMap::new(), protected: Vec::new(), last_amount: 0};
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
        
        //println!("\n\n{:#?}\nscope_id: {},\n{:#?}\n{:#?}",self,scope_id,scopes,tracker);
        io::stdout().flush().unwrap();

        self.mark(scopes, scope_id, &mut tracker);

        for vec in &self.protected {
            for var_id in vec {
                if tracker.marked_values.contains(&var_id) {
                    tracker.marked_values.remove(var_id);
                    let mut value_ids: Vec<RegIndex> = Vec::new();
                    let mut scope_ids: Vec<RegIndex> = Vec::new();
                    get_value_references(
                        self.register.get(&var_id).unwrap(),
                        self, scopes,
                        &mut value_ids,
                        &mut scope_ids,
                    );
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
        }
        //println!("{:#?}",tracker);
        for i in tracker.marked_scopes {
            scopes.register.remove(&i);
        }
        for i in tracker.marked_values {
            self.register.remove(&i);
        }
        self.last_amount = self.register.len()
    }

    pub fn mark(&self, scopes: &mut ScopeList, scope_id: RegIndex, tracker: &mut CollectTracker) {
        let mut var_check_ids = Vec::new();
        tracker.marked_scopes.remove(&scope_id);
        for (_, var_id) in &scopes.register.get(&scope_id).unwrap().vars {
            var_check_ids.push(*var_id);
        }
        for var_id in var_check_ids {
            if tracker.marked_values.contains(&var_id) {
                tracker.marked_values.remove(&var_id);
                let mut value_ids: Vec<RegIndex> = Vec::new();
                let mut scope_ids: Vec<RegIndex> = Vec::new();
                get_value_references(
                    self.register.get(&var_id).unwrap(),
                    self, scopes,
                    &mut value_ids,
                    &mut scope_ids,
                );
                for i in scope_ids {
                    if tracker.marked_scopes.contains(&i) {
                        self.mark(scopes, i, tracker);
                    }
                }
                for i in value_ids {
                    tracker.marked_values.remove(&i);
                }
            }
        }
        let parent_id = scopes.register.get(&scope_id).unwrap().parent_id;
        match parent_id {
            Some(id) => if tracker.marked_scopes.contains(&id) { self.mark(scopes, id, tracker) },
            None => (),
        }
        let caller_id = scopes.register.get(&scope_id).unwrap().caller_id;
        match caller_id {
            Some(id) => if tracker.marked_scopes.contains(&id) { self.mark(scopes, id, tracker) },
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

enum VarExistence {
    Name(String),
    Id(RegIndex),
}

fn get_value_id(node: &ASTNode, scope_id: RegIndex, memory: &mut Memory, scopes: &mut ScopeList) 
    -> Result<VarExistence,BaseError>
{
    match node {
        ASTNode::Var { name } => match scopes.get_var_id(name.clone(), scope_id) {
            Some(id) => Ok(VarExistence::Id(id)),
            None => Ok(VarExistence::Name(name.clone())),
        },
        ASTNode::Index { base, index } => {
            let i = match protecute!(index, scope_id, memory, scopes) {
                Value::Number(value) => value.floor(),
                _ => error_out!("Cannot index with type")
            } as isize;
            let base_id = get_value_id(base, scope_id, memory, scopes)?;
            match base_id {
                VarExistence::Name(name) => error_out!(format!("Unknown variable {}", name)),
                VarExistence::Id(id) => match memory.get(id).clone() {
                    Value::Array(arr) => if i >= arr.len() as isize || i < 0 {
                        error_out!("Index out of bounds")
                    } else { Ok(VarExistence::Id(arr[i as usize])) },
                    _ => error_out!("Type cannot be indexed"),
                },
            }
        }
        _ => error_out!("Expected variable name")
    }
}


fn execute(node: &ASTNode, scope_id: RegIndex, memory: &mut Memory, scopes: &mut ScopeList) -> ValueResult {
    //println!("\n\n{:#?}\nscope_id: {},\n{:#?}\n{:#?}",memory,scope_id,scopes,node);
    //println!("{:?}", memory.protected);
    
    if memory.register.len() > 50000 + memory.last_amount {
        memory.collect(scopes, scope_id);
    }

    memory.new_protected();

    let val = match node {
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
                    let right_eval = protecute!(right, scope_id, memory, scopes);
                    let value_id = match get_value_id(left, scope_id, memory, scopes)? {
                        VarExistence::Id(id) => id,
                        VarExistence::Name(name) => error_out!(format!("Unknown variable {}", name)),
                    };
                    let value = memory.register.get(&value_id).unwrap();
                    let new_value = match op {
                        Token::PlusEq => value.plus(&right_eval)?,
                        Token::MinusEq => value.minus(&right_eval)?,
                        Token::MultEq => value.mult(&right_eval)?,
                        Token::DivEq => value.div(&right_eval)?,
                        Token::ModEq => value.rem(&right_eval)?,
                        Token::PowEq => value.pow(&right_eval)?,
                        _ => unimplemented!(),
                    };
                    memory.set(new_value.clone(), value_id);
                    new_value
                },
                Token::Assign => {
                    let right_eval = protecute!(right, scope_id, memory, scopes);
                    match get_value_id(left, scope_id, memory, scopes)? {
                        VarExistence::Id(id) => {
                            memory.set(right_eval.clone(), id);
                            right_eval
                        },
                        VarExistence::Name(name) => {
                            scopes.set_var(name, scope_id, memory, &right_eval, true);
                            right_eval
                        },
                    }
                }
                Token::LocalAssign => {
                    let right_eval = protecute!(right, scope_id, memory, scopes);
                    match (**left).clone() {
                        ASTNode::Var { name } => {
                            scopes.set_var_local(name, scope_id, memory, &right_eval);
                            right_eval
                        },
                        _ => error_out!("Expected variable name")
                    }
                }
                
                Token::And => {
                    if !protecute!(left, scope_id, memory, scopes).to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(false) )
                    }
                    if !protecute!(right, scope_id, memory, scopes).to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(false) )
                    }
                    Value::Bool(true)
                }
                Token::Or => {
                    if protecute!(left, scope_id, memory, scopes).to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(true) )
                    }
                    if protecute!(right, scope_id, memory, scopes).to_bool()? { 
                        memory.pop_protected();
                        return Ok( Value::Bool(true) )
                    }
                    Value::Bool(false)
                }
                _ => todo!(),
            }
        },
        ASTNode::Var { name: _ } => {
            match get_value_id(node, scope_id, memory, scopes)? {
                VarExistence::Name(name) => error_out!(format!("Unknown variable {}", name)),
                VarExistence::Id(id) => memory.register.get(&id).unwrap().clone(),
            }
        },
        ASTNode::StatementList { statements } => {
            let mut last = Value::Null;
            for i in statements {
                last = protecute!( i, scope_id, memory, scopes );
            }
            last
        },
        ASTNode::If { conds, if_none } => {
            for i in conds {
                if protecute!(&i.0, scope_id, memory, scopes).to_bool()? {
                    memory.pop_protected();
                    return Ok( execute(&i.1, derive_scope(scope_id, scope_id, scopes), memory, scopes)? )
                }
            }

            match &**if_none {
                Some(node) => protecute!(node, scope_id, memory, scopes),
                None => Value::Null,
            }
        },
        ASTNode::While { cond, code } => {
            let mut last = Value::Null;
            loop {
                if protecute!(cond, scope_id, memory, scopes).to_bool()? {
                    last = protecute!(code, derive_scope(scope_id, scope_id, scopes), memory, scopes);
                } else { memory.pop_protected(); return Ok( last ) ; }
            }
        },
        ASTNode::Value { value } => value.clone(),
        ASTNode::Block { code } =>
            protecute!(code, derive_scope(scope_id, scope_id, scopes), memory, scopes),
        ASTNode::Func { code, arg_names } => {
            Value::Function {arg_names: arg_names.clone(), code: code.clone(), scope_id}
        }
        ASTNode::Call { base, args } => {
            match protecute!(base, scope_id, memory, scopes) {
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
                                let mut strs = Vec::new();
                                for i in args {
                                    strs.push( protecute!(i, scope_id, memory, scopes).to_str(memory, &mut vec![]));
                                }
                                print!("{}",strs.join(" "));
                            }
                            Value::Null
                        }
                        "println" => {
                            if args.len() == 0 {
                                println!("");
                            } else {
                                let mut strs = Vec::new();
                                for i in args {
                                    strs.push( protecute!(i, scope_id, memory, scopes).to_str(memory, &mut vec![]));
                                }
                                println!("{}",strs.join(" "));
                            }
                            Value::Null
                        }
                        "memtest" => {
                            println!("{:#?}",memory);
                            println!("{:#?}",scopes);
                            io::stdout().flush().unwrap();
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
            let i = match protecute!(index, scope_id, memory, scopes) {
                Value::Number(value) => value.floor(),
                _ => error_out!("Cannot index with type")
            } as isize;
            match protecute!(base, scope_id, memory, scopes) {
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



