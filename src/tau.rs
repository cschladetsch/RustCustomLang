// Tau Language - Network language with futures
use crate::value::{Value, FutureState};
use std::collections::HashMap;

pub fn parse_tau(input: &str, variables: &HashMap<String, Value>, parse_rho_fn: impl Fn(&str) -> Result<Value, String>) -> Result<Value, String> {
    let input = input.trim();

    // Handle async operations
    if input.starts_with("async ") {
        let _operation = &input[6..];
        return Ok(Value::Future(FutureState::Pending));
    }

    // Handle await
    if input.starts_with("await ") {
        let var_name = input[6..].trim();
        if let Some(value) = variables.get(var_name) {
            match value {
                Value::Future(FutureState::Resolved(v)) => return Ok((**v).clone()),
                Value::Future(FutureState::Pending) => return Err("Future still pending".to_string()),
                Value::Future(FutureState::Rejected(e)) => return Err(e.clone()),
                _ => return Ok(value.clone()),
            }
        }
        return Err(format!("Variable {} not found", var_name));
    }

    // Default to Rho parsing
    parse_rho_fn(input)
}
