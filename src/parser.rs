
use crate::{errors::{self, BaseError}, lexer::Token, value::Value};

type ParsePos = usize;
type ParseResult = Result<(ASTNode, ParsePos), errors::BaseError>;
type TokenList = Vec<Token>;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Value {value: Box<ASTNode>},
    StatementList {statements: Vec<ASTNode>},
    Op {left: Box<ASTNode>, op: Token, right: Box<ASTNode>},
    Block {code: Box<ASTNode> },
    Call {base: Box<ASTNode>, args: Vec<ASTNode>},
    Num {value: f64},
    Unary {op: Token, value: Box<ASTNode>},
    Var {name: String},
    Constant {value: Value},
    If {conds: Vec<(ASTNode,ASTNode)>, if_none: Box<Option<ASTNode>>},
    While {cond: Box<ASTNode>, code: Box<ASTNode>},
    Func {code: Box<ASTNode>, arg_names: Vec<String>},
}

struct Precedence {
    tok_check: fn(&Token) -> bool,
    right_assoc: bool,
}


macro_rules! destr {
    {
        $(!$let1:ident)? $arg1:ident, $(!$let2:ident)? $arg2:ident from $func:expr
    } => {
        let destr_temp_owo = $func?;
        $($let1)? $arg1 = destr_temp_owo.0;
        $($let2)? $arg2 = destr_temp_owo.1;
    }
}

const PRECEDENCES: &[Precedence] = &[
    Precedence {right_assoc: true, tok_check: ( |t| matches!(t, Token::Assign | Token::LocalAssign | Token::PlusEq | Token::MinusEq | Token::MultEq | Token::DivEq | Token::ModEq | Token::PowEq)) },
    Precedence {right_assoc: false, tok_check: ( |t| matches!(t, Token::Or )) },
    Precedence {right_assoc: false, tok_check: ( |t| matches!(t, Token::And )) },
    Precedence {right_assoc: false, tok_check: ( |t| matches!(t, Token::Greater | Token::Lesser | Token::GreaterEq | Token::LesserEq | Token::Eq | Token::NotEq )) },
    Precedence {right_assoc: false, tok_check: ( |t| matches!(t, Token::Plus | Token::Minus )) },
    Precedence {right_assoc: false, tok_check: ( |t| matches!(t, Token::Mult | Token::Div | Token::Mod )) },
    Precedence {right_assoc: true, tok_check: ( |t| matches!(t, Token::Pow )) },
];


fn skip_eol(tokens: &TokenList, mut pos: ParsePos) -> ParsePos {
    while matches!(&tokens[pos], Token::Eol) {
        pos += 1;
    }
    pos
}

fn parse_value(tokens: &TokenList, mut pos: ParsePos) -> ParseResult {
    let tok = &tokens[pos];
    match tok {
        Token::Number(value) => Ok((ASTNode::Num{ value: *value }, pos + 1)),
        Token::Plus | Token::Minus | Token::Not => {
            let op = tok;
            destr!{!let value, pos from parse_op(tokens, pos + 1, PRECEDENCES.len() - 2)}
            Ok((ASTNode::Unary{op: op.clone(), value: Box::new(value)}, pos))
        },
        Token::LParen => {
            destr!{!let value, pos from parse_expr(tokens, pos + 1)}
            if !matches!(&tokens[pos], Token::RParen) {
                return Err(BaseError::ParseError("Expected ')'".to_string()));
            }
            Ok((value, pos + 1))
        },
        Token::Identifier(name) => Ok((ASTNode::Var{name: name.clone()}, pos + 1)),
        Token::True => Ok((ASTNode::Constant{value: Value::Bool(true)}, pos + 1)),
        Token::False => Ok((ASTNode::Constant{value: Value::Bool(false)}, pos + 1)),
        Token::Null => Ok((ASTNode::Constant{value: Value::Null}, pos + 1)),
        Token::If => {
            let mut conds: Vec<(ASTNode, ASTNode)> = Vec::new();

            destr!{!let condition, pos from parse_expr(tokens, pos + 1)}
            destr!{!let branch, pos from parse_expr(tokens, pos)}
            conds.push((condition, branch));

            let mut if_none: Option<ASTNode> = None;

            while matches!(&tokens[pos], Token::Elif) {
                destr!{!let condition, pos from parse_expr(tokens, pos + 1)}
                destr!{!let branch, pos from parse_expr(tokens, pos)}

                conds.push((condition, branch));
            }
            if matches!(&tokens[pos], Token::Else) {
                let temp = parse_expr(tokens, pos + 1)?;
                if_none = Some(temp.0);
                pos = temp.1;
            }
            
            Ok((ASTNode::If{conds, if_none: Box::new(if_none)}, pos))
        },
        Token::While => {
            destr!{!let condition, pos from parse_expr(tokens, pos + 1)}
            destr!{!let code, pos from parse_expr(tokens, pos)}
            
            Ok((ASTNode::While{cond: Box::new(condition), code: Box::new(code)}, pos))
        },
        Token::LBracket => {
            parse_block(tokens, pos)
        },
        Token::Pipe | Token::Or => {
            let mut arg_names: Vec<String> = Vec::new();
            if let Token::Pipe = tok {
                pos += 1;
                pos = skip_eol(tokens, pos);
                while !matches!(&tokens[pos], Token::Pipe) {
                    if let Token::Identifier(name) = &tokens[pos] {
                        arg_names.push(name.clone());
                        pos += 1;
                        pos = skip_eol(tokens, pos);
                        if !matches!(&tokens[pos], Token::Comma) {
                            if !matches!(&tokens[pos], Token::Pipe) {
                                return Err(BaseError::ParseError("Expected ',' or '|'".to_string()));
                            }
                        } else {
                            pos += 1;
                        }
                    } else {
                        return Err(BaseError::ParseError("Expected argument name".to_string()));
                    }
                }
            }
            destr!{!let code, pos from parse_expr(tokens, pos + 1)}
            Ok((ASTNode::Func{code: Box::new(code), arg_names}, pos))
        }
        _ => Err(BaseError::ParseError("Expected value".to_string()))
    }
}

fn parse_term(tokens: &TokenList, pos: ParsePos) -> ParseResult {
    let (mut value, mut pos) = parse_value(tokens, pos)?;

    loop {
        if matches!(&tokens[pos], Token::LParen) {
            pos += 1;
            pos = skip_eol(tokens, pos);
            let mut args: Vec<ASTNode> = Vec::new();
            while !matches!(&tokens[pos], Token::RParen) {
                destr!{!let arg, pos from parse_expr(tokens, pos)}
                args.push(arg);
                pos = skip_eol(tokens, pos);
                if !matches!(&tokens[pos], Token::Comma) {
                    if !matches!(&tokens[pos], Token::RParen) {
                        return Err(BaseError::ParseError("Expected ',' or ')'".to_string()));
                    }
                } else { pos += 1; }
            }
            pos += 1;
            value = ASTNode::Call {base: Box::new(value), args}
        } else {
            return Ok((value, pos))
        }
    }
}

fn parse_block(tokens: &TokenList, mut pos: ParsePos) -> ParseResult {
    pos = skip_eol(tokens, pos);
    if !matches!(&tokens[pos], Token::LBracket) {
        return Err(BaseError::ParseError("Expected '{'".to_string()))
    }
    destr!{!let code, pos from parse_statements(tokens, pos + 1)}

    if !matches!(&tokens[pos], Token::RBracket) {
        return Err(BaseError::ParseError("Expected '}'".to_string()))
    } 

    Ok((ASTNode::Block{code: Box::new(code)}, pos + 1))
}

fn parse_op(tokens: &TokenList, pos: ParsePos, op_id: usize) -> ParseResult {
    let tok_check = &PRECEDENCES[op_id].tok_check;
    let right_assoc = &PRECEDENCES[op_id].right_assoc;
    
    let (mut left, mut pos) = 
        if op_id + 1 >= PRECEDENCES.len()
        { parse_term(tokens, pos) }
        else
        { parse_op(tokens, pos, op_id + 1) }?;
    
    while tok_check(&tokens[pos]) {
        let op = tokens.get(pos).unwrap(); pos += 1;
        let right: ASTNode;

        destr!{right, pos from if !right_assoc {
            if op_id + 1 >= PRECEDENCES.len()
            { parse_term(tokens, pos) } 
            else
            { parse_op(tokens, pos, op_id + 1) }
        } else {
            parse_op(tokens, pos, op_id)
        }};

        left = ASTNode::Op {left: Box::new(left), op: op.clone(), right: Box::new(right)};

    }

    Ok((left,pos))

}

fn parse_expr(tokens: &TokenList, pos: ParsePos) -> ParseResult {
    parse_op(tokens, pos, 0)
}

fn parse_statement(tokens: &TokenList, pos: ParsePos) -> ParseResult {
    
    let (statement, pos) = parse_expr(tokens, pos)?;
    if !matches!(&tokens[pos], Token::Eol) {
        return Err(BaseError::ParseError("Expected end of line (';' or newline)".to_string()))
    }
    
    Ok((statement,pos + 1))
}

fn parse_statements(tokens: &TokenList, mut pos: ParsePos) -> ParseResult {
    
    let mut statements: Vec<ASTNode> = Vec::new();
    
    while !matches!(&tokens[pos], Token::Eof | Token::RBracket) {
        pos = skip_eol(tokens, pos);
        destr!{!let statement, pos from parse_statement(tokens, pos)};
        statements.push(statement);
        pos = skip_eol(tokens, pos);
    }
    
    Ok((ASTNode::StatementList{statements},pos))
}

pub fn parse(tokens: &TokenList) -> ParseResult {
    let (result, pos) = parse_statements(tokens, 0)?;
    if !matches!(&tokens[pos], Token::Eof) {
        return Err(BaseError::ParseError("Expected end of file".to_string()))
    }

    Ok((ASTNode::Value{value: Box::new(result)},pos))
}

