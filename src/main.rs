mod lexer;
mod parser;
mod errors;
mod value;
mod interpreter;

use std::{fs, io::{self, Write}};
use interpreter::ScopeList;
use logos::Logos;

use crate::{errors::BaseError, interpreter::{Memory}, lexer::Token, value::Value};


fn run(code: String, memory: &mut Memory, scopes: &mut ScopeList, print_result: bool) {
    let mut tokens = lexer::Token
        ::lexer(&code)
        .collect::<Vec<lexer::Token>>();
    tokens.push(Token::Eol);
    tokens.push(Token::Eof);
    
    let tree = parser::parse(&tokens);

    
    match tree {
        Ok((node, _)) => {
            //println!("{:#?}",node);
            
            let ass = interpreter::start_execute(&node, scopes, memory);
            if let Ok(interpreter::NodeResult::Value(result)) = ass {
                if print_result {
                    match result {
                        Value::Null => (),
                        _ => println!("{}",result.to_str())
                    }
                }
            } else if let Err(BaseError::InterpreterError(message)) = ass {
                println!("{:?}",message);
            }
        },
        Err(BaseError::ParseError(message)) => println!("{}",message),
        _ => unimplemented!(),
    }
    
}

fn main() {
    print!("\x1B[2J\x1B[1;1H");
    if false {
        print!("{:?}",Value::String(String::from("ass")))
    }

    let mut memory = Memory::new();
    let mut scopes = ScopeList::new();
    
    scopes.set_var_local("sin".to_string(), 0, &mut memory, &Value::Builtin("sin".to_string()));
    scopes.set_var_local("cos".to_string(), 0, &mut memory, &Value::Builtin("cos".to_string()));
    scopes.set_var_local("tan".to_string(), 0, &mut memory, &Value::Builtin("tan".to_string()));
    scopes.set_var_local("print".to_string(), 0, &mut memory, &Value::Builtin("print".to_string()));
    scopes.set_var_local("println".to_string(), 0, &mut memory, &Value::Builtin("println".to_string()));
    scopes.set_var_local("memtest".to_string(), 0, &mut memory, &Value::Builtin("memtest".to_string()));
    scopes.set_var_local("collect".to_string(), 0, &mut memory, &Value::Builtin("collect".to_string()));
    
    if false {
        let input_str = fs::read_to_string("code.blb")
            .expect("Something went wrong reading the file");
        
        run(input_str, &mut memory, &mut scopes, false);
        print!("\n");
    } else {

        print!("
BlueBat v0.1.0 Console
--------------------------- 

");

        loop {
            print!("{}", ">>> ");
            io::stdout().flush().unwrap();
    
            let mut input_str = String::new();
            io::stdin()
                .read_line(&mut input_str)
                .expect("Failed to read line");
            
            let input_str = format!("{}{}",&input_str[..input_str.len()-2],"\n");
    
            run(input_str, &mut memory, &mut scopes, true);
            
        }
    }

}
