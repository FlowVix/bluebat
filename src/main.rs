mod lexer;
mod parser;
mod errors;
mod value;
mod interpreter;

use std::{fs, io::{self, Write}};
use logos::Logos;

use crate::{errors::BaseError, interpreter::{Memory, Scope}, lexer::Token, value::Value};


fn run(code: String, memory: &mut Memory, global_scope: &mut Scope, print_result: bool) {
    let mut tokens = lexer::Token
        ::lexer(&code)
        .collect::<Vec<lexer::Token>>();
    tokens.push(Token::Eol);
    tokens.push(Token::Eof);
    

    let tree = parser::parse(&tokens);
    match tree {
        Ok((node, _)) => {
            //println!("{:?}",node);
            let ass = interpreter::start_execute(&node, global_scope, memory);
            if let Ok(interpreter::NodeResult::Value(result)) = ass {
                if print_result {
                    match result {
                        Value::Null => (),
                        _ => println!("{:#?}",result)
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

    if false {
        print!("{:?}",Value::String(String::from("ass")))
    }

    let mut memory = Memory::new();
    let mut global_scope = Scope::new();
    global_scope.set_var("sin".to_string(), &mut memory, &Value::Builtin("sin".to_string()), true);
    global_scope.set_var("cos".to_string(), &mut memory, &Value::Builtin("cos".to_string()), true);
    global_scope.set_var("tan".to_string(), &mut memory, &Value::Builtin("tan".to_string()), true);
    global_scope.set_var("print".to_string(), &mut memory, &Value::Builtin("print".to_string()), true);

    if true {
        print!("\n----------------------------------------\n\n");
        let input_str = fs::read_to_string("code.blb")
            .expect("Something went wrong reading the file");
        
        run(input_str, &mut memory, &mut global_scope, false);
        print!("\n----------------------------------------\n\n");
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
    
            run(input_str, &mut memory, &mut global_scope, true);
            
        }
    }

}
