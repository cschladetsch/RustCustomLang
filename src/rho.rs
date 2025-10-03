// Rho Language - Infix notation with tab indentation
use crate::value::Value;

pub fn parse_rho(input: &str, parse_eval_fn: impl Fn(&str) -> Result<Value, String>) -> Result<Value, String> {
    // For now, delegate to the main parser
    parse_eval_fn(input)
}
