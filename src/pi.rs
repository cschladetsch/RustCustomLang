// Pi Language - Postfix/RPN notation
use crate::value::Value;
use std::collections::HashMap;

pub fn parse_pi(input: &str, variables: &mut HashMap<String, Value>, parse_value_fn: impl Fn(&str) -> Result<Value, String>) -> Result<Value, String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut stack: Vec<Value> = Vec::new();

    for token in tokens {
        match token {
            "+" => {
                if stack.len() < 2 {
                    return Err("Not enough operands for +".to_string());
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a.add(&b)?);
            }
            "-" => {
                if stack.len() < 2 {
                    return Err("Not enough operands for -".to_string());
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a.sub(&b)?);
            }
            "*" => {
                if stack.len() < 2 {
                    return Err("Not enough operands for *".to_string());
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a.mul(&b)?);
            }
            "/" => {
                if stack.len() < 2 {
                    return Err("Not enough operands for /".to_string());
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a.div(&b)?);
            }
            "=" => {
                if stack.len() < 2 {
                    return Err("Not enough operands for =".to_string());
                }
                let name = stack.pop().unwrap();
                let value = stack.pop().unwrap();
                if let Value::Str(var_name) = name {
                    variables.insert(var_name, value.clone());
                    stack.push(value);
                } else {
                    return Err("Variable name must be a string".to_string());
                }
            }
            "-->" => {
                if stack.is_empty() {
                    return Err("No value to print".to_string());
                }
                let val = stack.pop().unwrap();
                match val {
                    Value::Array(ref arr) => {
                        for item in arr {
                            print!("{:?} ", item);
                        }
                        println!();
                        stack.push(Value::Unit);
                    }
                    _ => stack.push(val),
                }
            }
            _ => {
                if let Some(var_val) = variables.get(token) {
                    stack.push(var_val.clone());
                } else {
                    stack.push(parse_value_fn(token)?);
                }
            }
        }
    }

    if stack.len() == 1 {
        Ok(stack.pop().unwrap())
    } else if stack.is_empty() {
        Ok(Value::Unit)
    } else {
        Err(format!("Stack has {} values remaining", stack.len()))
    }
}
