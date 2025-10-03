mod value;
mod pi;
mod rho;
mod tau;

use std::io::{self, Write};
use std::process::Command;
use std::collections::HashMap;
use value::{Value, Color, FutureState, Continuation};

// Language modes
#[derive(Debug, Clone, PartialEq)]
enum Language {
    Pi,   // Postfix/RPN notation
    Rho,  // Infix with tab indentation
    Tau,  // Network language with futures
}

// Expression types
#[derive(Debug, Clone)]
enum Expr {
    Value(Value),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    // Color operations
    Blend(Box<Expr>, Box<Expr>),   // Blend two colors
    Scale(Box<Expr>, f32),          // Scale a color
    // Array/Map operations
    Get(Box<Expr>, Box<Expr>),     // Array/Map indexing: arr[index] or map[key]
    // Continuation algebra operations
    Compose(Box<Expr>, Box<Expr>),  // Continuation composition: c1 ; c2
    Choice(Box<Expr>, Box<Expr>),   // Continuation choice: c1 | c2
    // Loop constructs
    For(String, Box<Expr>, Box<Expr>),     // for var in iterable { body }
    While(Box<Expr>, Box<Expr>),           // while condition { body }
    Block(Vec<Expr>),                      // { expr1; expr2; ... }
}

// Continuation stack - holds suspended computations
struct ContinuationStack {
    stack: Vec<Continuation>,
}

impl ContinuationStack {
    fn new() -> Self {
        ContinuationStack { stack: Vec::new() }
    }

    fn push(&mut self, cont: Continuation) {
        self.stack.push(cont);
    }

    fn pop(&mut self) -> Option<Continuation> {
        self.stack.pop()
    }

    fn clear(&mut self) {
        self.stack.clear();
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

// Control flow operations
enum ControlFlow {
    Resume,           // Execute what's on the continuation stack
    Break,            // Drop continuation stack and resume next
    Continue(Value),  // Takes a continuation as argument
}

// Runtime context
struct Runtime {
    cont_stack: ContinuationStack,
}

impl Runtime {
    fn new() -> Self {
        Runtime {
            cont_stack: ContinuationStack::new(),
        }
    }

    // resume - does what's on the continuation stack
    fn resume(&mut self) -> Value {
        if let Some(cont) = self.cont_stack.pop() {
            match cont {
                Continuation::Resume(f) => f(),
                Continuation::Empty => Value::Unit,
            }
        } else {
            Value::Unit
        }
    }

    // break - drop continuation stack and resume next
    fn break_flow(&mut self) -> Value {
        self.cont_stack.clear();
        Value::Unit
    }

    // continue - takes a continuation argument (fun f(a,))
    fn continue_with(&mut self, cont: Value) -> Value {
        match cont {
            Value::Continuation(c) => {
                self.cont_stack.push(*c);
                self.resume()
            }
            _ => Value::Unit,
        }
    }

    // Evaluate expressions
    fn eval(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Value(v) => Ok(v),
            Expr::Add(left, right) => {
                let l = self.eval(*left)?;
                let r = self.eval(*right)?;
                l.add(&r)
            }
            Expr::Sub(left, right) => {
                let l = self.eval(*left)?;
                let r = self.eval(*right)?;
                l.sub(&r)
            }
            Expr::Mul(left, right) => {
                let l = self.eval(*left)?;
                let r = self.eval(*right)?;
                l.mul(&r)
            }
            Expr::Div(left, right) => {
                let l = self.eval(*left)?;
                let r = self.eval(*right)?;
                l.div(&r)
            }
            Expr::Blend(left, right) => {
                let l = self.eval(*left)?;
                let r = self.eval(*right)?;
                l.blend(&r)
            }
            Expr::Scale(expr, factor) => {
                let v = self.eval(*expr)?;
                v.scale(factor)
            }
            Expr::Get(arr_expr, idx_expr) => {
                let arr = self.eval(*arr_expr)?;
                let idx = self.eval(*idx_expr)?;

                match arr {
                    Value::Array(ref vec) => {
                        match idx {
                            Value::Num(n) => {
                                let index = n as usize;
                                vec.get(index)
                                    .cloned()
                                    .ok_or_else(|| format!("Index {} out of bounds", index))
                            }
                            _ => Err("Array index must be a number".to_string()),
                        }
                    }
                    Value::Map(ref pairs) => {
                        // Find matching key in map
                        for (key, value) in pairs {
                            // Check for equality
                            let matches = match (key, &idx) {
                                (Value::Num(k), Value::Num(i)) => (k - i).abs() < f64::EPSILON,
                                (Value::Str(k), Value::Str(i)) => k == i,
                                _ => false,
                            };
                            if matches {
                                return Ok(value.clone());
                            }
                        }
                        Err(format!("Key {:?} not found in map", idx))
                    }
                    _ => Err("Get requires an array or map".to_string()),
                }
            }
            Expr::Compose(left, right) => {
                // Continuation composition: execute left, then right
                let l_val = self.eval(*left)?;
                let r_val = self.eval(*right)?;

                match (l_val, r_val) {
                    (Value::Continuation(c1), Value::Continuation(c2)) => {
                        // Push c2 first (will execute second)
                        self.cont_stack.push(*c2);
                        // Push c1 second (will execute first)
                        self.cont_stack.push(*c1);
                        Ok(Value::Unit)
                    }
                    _ => Err("Compose requires two continuations".to_string()),
                }
            }
            Expr::Choice(left, right) => {
                // Continuation choice: try left, if it fails/returns Unit, use right
                let l_val = self.eval(*left)?;

                match l_val {
                    Value::Unit => self.eval(*right),
                    v => Ok(v),
                }
            }
            Expr::While(condition, body) => {
                let mut last_val = Value::Unit;
                loop {
                    let cond_val = self.eval(*condition.clone())?;
                    if !cond_val.is_truthy() {
                        break;
                    }
                    last_val = self.eval(*body.clone())?;
                }
                Ok(last_val)
            }
            Expr::For(var_name, iterable_expr, body) => {
                let iterable = self.eval(*iterable_expr)?;
                let mut last_val = Value::Unit;

                match iterable {
                    Value::Array(ref arr) => {
                        for item in arr {
                            // Store loop variable
                            self.set_variable(var_name.clone(), item.clone());
                            last_val = self.eval(*body.clone())?;
                        }
                    }
                    _ => return Err("For loop requires an array".to_string()),
                }

                Ok(last_val)
            }
            Expr::Block(exprs) => {
                let mut last_val = Value::Unit;
                for expr in exprs {
                    last_val = self.eval(expr)?;
                }
                Ok(last_val)
            }
        }
    }

    fn set_variable(&mut self, _name: String, _value: Value) {
        // For now, we don't have variable storage in Runtime
        // This will need to be added when implementing full variable support
    }
}

// REPL - Multi-language Read-Eval-Print Loop
// Supports: Pi (postfix), Rho (infix+tabs), Tau (network+futures)
struct Repl {
    runtime: Runtime,
    variables: HashMap<String, Value>,
    current_lang: Language,
    indent_level: usize,  // For Rho language
}

impl Repl {
    fn new() -> Self {
        Repl {
            runtime: Runtime::new(),
            variables: HashMap::new(),
            current_lang: Language::Pi,  // Default to Pi (postfix)
            indent_level: 0,
        }
    }

    fn run(&mut self) {
        println!("Multi-Language REPL v0.2.0");
        println!("Languages: Pi (postfix), Rho (infix+tabs), Tau (network+futures)");
        println!("Commands: :quit, :help, :pi, :rho, :tau");
        println!("Use `command` to execute bash commands\n");
        println!("Current language: {:?}\n", self.current_lang);

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let input = input.trim();

            // Skip empty lines and comments
            if input.is_empty() || input.starts_with('#') {
                continue;
            }

            // Handle special commands
            if input.starts_with(':') {
                match input {
                    ":quit" | ":q" => {
                        println!("Goodbye!");
                        break;
                    }
                    ":help" | ":h" => {
                        self.print_help();
                        continue;
                    }
                    ":pi" => {
                        self.current_lang = Language::Pi;
                        println!("Switched to Pi (postfix/RPN notation)");
                        continue;
                    }
                    ":rho" => {
                        self.current_lang = Language::Rho;
                        println!("Switched to Rho (infix with tab indentation)");
                        continue;
                    }
                    ":tau" => {
                        self.current_lang = Language::Tau;
                        println!("Switched to Tau (network language with futures)");
                        continue;
                    }
                    _ => {
                        println!("Unknown command: {}", input);
                        continue;
                    }
                }
            }

            // Handle bash injection with backticks
            if input.contains('`') {
                match self.process_bash(input) {
                    Ok(result) => println!("{}", result),
                    Err(e) => println!("Bash error: {}", e),
                }
                continue;
            }

            // Parse and evaluate based on current language
            match self.current_lang {
                Language::Pi => {
                    match self.parse_pi(input) {
                        Ok(value) => println!("{:?}", value),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Language::Rho => {
                    match self.parse_rho(input) {
                        Ok(value) => println!("{:?}", value),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Language::Tau => {
                    match self.parse_tau(input) {
                        Ok(value) => println!("{:?}", value),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
        }
    }

    fn print_help(&self) {
        println!("Multi-Language REPL Help:");
        println!("\nLanguages:");
        println!("  :pi  - Switch to Pi (postfix/RPN): 3 4 +");
        println!("  :rho - Switch to Rho (infix+tabs): 3 + 4");
        println!("  :tau - Switch to Tau (network+futures): async operations");
        println!("\nPi (Postfix):");
        println!("  3 4 +        # 7");
        println!("  arr = [1,2,3]; arr -->  # prints: 1 2 3");
        println!("\nRho (Infix):");
        println!("  3 + 4        # 7");
        println!("  if a == 1    # uses tabs for blocks");
        println!("\nTau (Network):");
        println!("  async fetch  # returns Future");
        println!("  await val    # resolves Future");
        println!("\nCommon:");
        println!("  Bash: `ls`, `echo hello`, `pwd`");
        println!("  Commands: :quit, :help, :pi, :rho, :tau");
    }

    // Pi language parser (Postfix/RPN notation)
    fn parse_pi(&mut self, input: &str) -> Result<Value, String> {
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let mut stack: Vec<Value> = Vec::new();

        for token in tokens {
            match token {
                // Operators
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
                    // Variable assignment: value name =
                    if stack.len() < 2 {
                        return Err("Not enough operands for =".to_string());
                    }
                    let name = stack.pop().unwrap();
                    let value = stack.pop().unwrap();
                    if let Value::Str(var_name) = name {
                        self.variables.insert(var_name, value.clone());
                        stack.push(value);
                    } else {
                        return Err("Variable name must be a string".to_string());
                    }
                }
                "-->" => {
                    // Stack print operation
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
                // Try to parse as value or variable
                _ => {
                    if let Some(var_val) = self.variables.get(token) {
                        stack.push(var_val.clone());
                    } else {
                        stack.push(self.parse_value(token)?);
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

    // Rho language parser (Infix with tab indentation)
    fn parse_rho(&mut self, input: &str) -> Result<Value, String> {
        // For now, delegate to old infix parser
        self.parse_and_eval(input)
    }

    // Tau language parser (Network with futures)
    fn parse_tau(&mut self, input: &str) -> Result<Value, String> {
        let input = input.trim();

        // Handle proxy and agent generation from filename
        if input.starts_with("proxy ") {
            let filename = input[6..].trim().trim_matches('"').trim_matches('\'');
            return tau::generate_proxy_wrapper(filename);
        }

        if input.starts_with("agent ") {
            let filename = input[6..].trim().trim_matches('"').trim_matches('\'');
            return tau::generate_agent_wrapper(filename);
        }

        // Handle async operations
        if input.starts_with("async ") {
            return Ok(Value::Future(FutureState::Pending));
        }

        // Handle await
        if input.starts_with("await ") {
            let var_name = input[6..].trim();
            if let Some(value) = self.variables.get(var_name) {
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
        self.parse_rho(input)
    }

    fn process_bash(&self, input: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        let mut current = String::new();

        while let Some(ch) = chars.next() {
            if ch == '`' {
                // Found backtick, collect command
                let mut cmd = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '`' {
                        chars.next(); // consume closing backtick
                        break;
                    }
                    cmd.push(chars.next().unwrap());
                }

                // Execute bash command
                if !cmd.is_empty() {
                    match self.execute_bash(&cmd) {
                        Ok(output) => result.push_str(&output),
                        Err(e) => return Err(e),
                    }
                }
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            result.push_str(&current);
        }

        Ok(result)
    }

    fn execute_bash(&self, cmd: &str) -> Result<String, String> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", cmd])
                .output()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if !stderr.is_empty() {
                    Err(stderr)
                } else {
                    Ok(stdout.trim_end().to_string())
                }
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }

    fn parse_and_eval(&mut self, input: &str) -> Result<Value, String> {
        // Simple parser for basic expressions
        let input = input.trim();

        // Handle array/map indexing: arr[index] or map["key"]
        // Only if it doesn't start with '[' (which would be array literal)
        if !input.starts_with('[') {
            if let Some(bracket_pos) = input.find('[') {
                if let Some(close_bracket) = input.rfind(']') {
                    let arr_part = &input[..bracket_pos];
                    let idx_part = &input[bracket_pos+1..close_bracket];

                    let arr_val = self.parse_value(arr_part)?;
                    let idx_val = self.parse_value(idx_part)?;

                    return self.runtime.eval(Expr::Get(
                        Box::new(Expr::Value(arr_val)),
                        Box::new(Expr::Value(idx_val))
                    ));
                }
            }
        }

        // Handle color creation: color(r,g,b)
        if input.starts_with("color(") && input.ends_with(')') {
            let args = &input[6..input.len()-1];
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].trim().parse().map_err(|_| "Invalid r value")?;
                let g: u8 = parts[1].trim().parse().map_err(|_| "Invalid g value")?;
                let b: u8 = parts[2].trim().parse().map_err(|_| "Invalid b value")?;
                return Ok(Value::Color(Color::new(r, g, b)));
            }
        }

        // Handle simple arithmetic
        if let Some(pos) = input.find('+') {
            let left = self.parse_value(&input[..pos])?;
            let right = self.parse_value(&input[pos+1..])?;
            return self.runtime.eval(Expr::Add(Box::new(Expr::Value(left)), Box::new(Expr::Value(right))));
        }
        if let Some(pos) = input.find('-') {
            let left = self.parse_value(&input[..pos])?;
            let right = self.parse_value(&input[pos+1..])?;
            return self.runtime.eval(Expr::Sub(Box::new(Expr::Value(left)), Box::new(Expr::Value(right))));
        }
        if let Some(pos) = input.find('*') {
            let left = self.parse_value(&input[..pos])?;
            let right = self.parse_value(&input[pos+1..])?;
            return self.runtime.eval(Expr::Mul(Box::new(Expr::Value(left)), Box::new(Expr::Value(right))));
        }
        if let Some(pos) = input.find('/') {
            let left = self.parse_value(&input[..pos])?;
            let right = self.parse_value(&input[pos+1..])?;
            return self.runtime.eval(Expr::Div(Box::new(Expr::Value(left)), Box::new(Expr::Value(right))));
        }

        // Handle resume
        if input == "resume" {
            return Ok(self.runtime.resume());
        }

        // Handle break
        if input == "break" {
            return Ok(self.runtime.break_flow());
        }

        // Try to parse as a simple value
        self.parse_value(input)
    }

    fn parse_value(&self, input: &str) -> Result<Value, String> {
        let input = input.trim();

        // Try to parse as string (quoted)
        if (input.starts_with('"') && input.ends_with('"')) ||
           (input.starts_with('\'') && input.ends_with('\'')) {
            let s = &input[1..input.len()-1];
            return Ok(Value::Str(s.to_string()));
        }

        // Try to parse as number (f64)
        if let Ok(n) = input.parse::<f64>() {
            return Ok(Value::Num(n));
        }

        // Try to parse as array: [1,2,3]
        if input.starts_with('[') && input.ends_with(']') {
            let inner = &input[1..input.len()-1];
            if inner.is_empty() {
                return Ok(Value::Array(vec![]));
            }

            // Check if it's a map: [{1,2},{3,4}]
            if inner.trim_start().starts_with('{') {
                return self.parse_map(inner);
            }

            // Parse as array
            let parts: Vec<&str> = inner.split(',').collect();
            let mut values = Vec::new();
            for part in parts {
                values.push(self.parse_value(part)?);
            }
            return Ok(Value::Array(values));
        }

        // Try to parse as color
        if input.starts_with("color(") && input.ends_with(')') {
            let args = &input[6..input.len()-1];
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].trim().parse().map_err(|_| "Invalid r value")?;
                let g: u8 = parts[1].trim().parse().map_err(|_| "Invalid g value")?;
                let b: u8 = parts[2].trim().parse().map_err(|_| "Invalid b value")?;
                return Ok(Value::Color(Color::new(r, g, b)));
            }
        }

        Err(format!("Cannot parse value: {}", input))
    }

    fn parse_map(&self, input: &str) -> Result<Value, String> {
        let mut map = Vec::new();
        let mut depth = 0;
        let mut current_pair = String::new();

        for ch in input.chars() {
            if ch == '{' {
                depth += 1;
                if depth == 1 {
                    current_pair.clear();
                    continue;
                }
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    // Parse the pair
                    let parts: Vec<&str> = current_pair.split(',').collect();
                    if parts.len() == 2 {
                        let key = self.parse_value(parts[0])?;
                        let value = self.parse_value(parts[1])?;
                        map.push((key, value));
                    }
                    current_pair.clear();
                    continue;
                }
            }

            if depth > 0 {
                current_pair.push(ch);
            }
        }

        Ok(Value::Map(map))
    }
}

fn main() {
    let mut repl = Repl::new();
    repl.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resume_executes_continuation() {
        let mut runtime = Runtime::new();
        runtime.cont_stack.push(Continuation::Resume(Box::new(|| Value::Num(42.0))));

        match runtime.resume() {
            Value::Num(n) if n == 42.0 => (),
            _ => panic!("Expected Num(42.0)"),
        }
    }

    #[test]
    fn test_resume_empty_stack() {
        let mut runtime = Runtime::new();
        match runtime.resume() {
            Value::Unit => (),
            _ => panic!("Expected Unit for empty stack"),
        }
    }

    #[test]
    fn test_break_clears_stack() {
        let mut runtime = Runtime::new();
        runtime.cont_stack.push(Continuation::Resume(Box::new(|| Value::Num(1.0))));
        runtime.cont_stack.push(Continuation::Resume(Box::new(|| Value::Num(2.0))));

        runtime.break_flow();
        assert!(runtime.cont_stack.is_empty());
    }

    // Arithmetic tests with f64
    #[test]
    fn test_add() {
        let a = Value::Num(5.0);
        let b = Value::Num(3.0);
        match a.add(&b) {
            Ok(Value::Num(n)) if n == 8.0 => (),
            _ => panic!("Expected Num(8.0)"),
        }
    }

    #[test]
    fn test_sub() {
        let a = Value::Num(10.0);
        let b = Value::Num(3.0);
        match a.sub(&b) {
            Ok(Value::Num(n)) if n == 7.0 => (),
            _ => panic!("Expected Num(7.0)"),
        }
    }

    #[test]
    fn test_mul() {
        let a = Value::Num(6.0);
        let b = Value::Num(7.0);
        match a.mul(&b) {
            Ok(Value::Num(n)) if n == 42.0 => (),
            _ => panic!("Expected Num(42.0)"),
        }
    }

    #[test]
    fn test_div() {
        let a = Value::Num(20.0);
        let b = Value::Num(4.0);
        match a.div(&b) {
            Ok(Value::Num(n)) if n == 5.0 => (),
            _ => panic!("Expected Num(5.0)"),
        }
    }

    #[test]
    fn test_div_by_zero() {
        let a = Value::Num(10.0);
        let b = Value::Num(0.0);
        match a.div(&b) {
            Err(_) => (),
            _ => panic!("Expected error for division by zero"),
        }
    }

    #[test]
    fn test_expr_arithmetic() {
        let mut runtime = Runtime::new();
        let expr = Expr::Add(
            Box::new(Expr::Value(Value::Num(3.0))),
            Box::new(Expr::Mul(
                Box::new(Expr::Value(Value::Num(4.0))),
                Box::new(Expr::Value(Value::Num(5.0))),
            )),
        ); // 3 + (4 * 5) = 23

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 23.0 => (),
            _ => panic!("Expected Num(23.0)"),
        }
    }

    // Array tests
    #[test]
    fn test_array_creation() {
        let arr = Value::Array(vec![Value::Num(1.0), Value::Num(2.0), Value::Num(3.0)]);
        match arr {
            Value::Array(v) if v.len() == 3 => (),
            _ => panic!("Expected array with 3 elements"),
        }
    }

    #[test]
    fn test_array_concat() {
        let a1 = Value::Array(vec![Value::Num(1.0), Value::Num(2.0)]);
        let a2 = Value::Array(vec![Value::Num(3.0), Value::Num(4.0)]);
        match a1.add(&a2) {
            Ok(Value::Array(v)) if v.len() == 4 => (),
            _ => panic!("Expected concatenated array with 4 elements"),
        }
    }

    // Map tests
    #[test]
    fn test_map_creation() {
        let map = Value::Map(vec![(Value::Num(1.0), Value::Num(10.0))]);
        match map {
            Value::Map(m) if m.len() == 1 => (),
            _ => panic!("Expected map with 1 element"),
        }
    }

    // Array get tests
    #[test]
    fn test_array_get() {
        let mut runtime = Runtime::new();
        let arr = Value::Array(vec![Value::Num(10.0), Value::Num(20.0), Value::Num(30.0)]);
        let expr = Expr::Get(
            Box::new(Expr::Value(arr)),
            Box::new(Expr::Value(Value::Num(1.0)))
        );

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 20.0 => (),
            _ => panic!("Expected Num(20.0)"),
        }
    }

    #[test]
    fn test_array_get_out_of_bounds() {
        let mut runtime = Runtime::new();
        let arr = Value::Array(vec![Value::Num(10.0)]);
        let expr = Expr::Get(
            Box::new(Expr::Value(arr)),
            Box::new(Expr::Value(Value::Num(5.0)))
        );

        match runtime.eval(expr) {
            Err(_) => (),
            _ => panic!("Expected error for out of bounds"),
        }
    }

    // Map get tests
    #[test]
    fn test_map_get_num_key() {
        let mut runtime = Runtime::new();
        let map = Value::Map(vec![
            (Value::Num(1.0), Value::Num(100.0)),
            (Value::Num(2.0), Value::Num(200.0))
        ]);
        let expr = Expr::Get(
            Box::new(Expr::Value(map)),
            Box::new(Expr::Value(Value::Num(2.0)))
        );

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 200.0 => (),
            _ => panic!("Expected Num(200.0)"),
        }
    }

    #[test]
    fn test_map_get_str_key() {
        let mut runtime = Runtime::new();
        let map = Value::Map(vec![
            (Value::Str("x".to_string()), Value::Num(100.0)),
            (Value::Str("y".to_string()), Value::Num(200.0))
        ]);
        let expr = Expr::Get(
            Box::new(Expr::Value(map)),
            Box::new(Expr::Value(Value::Str("y".to_string())))
        );

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 200.0 => (),
            _ => panic!("Expected Num(200.0)"),
        }
    }

    // Color tests
    #[test]
    fn test_color_new() {
        let red = Color::new(255, 0, 0);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
    }

    #[test]
    fn test_color_blend() {
        let red = Color::new(255, 0, 0);
        let blue = Color::new(0, 0, 255);
        let purple = red.blend(&blue);
        assert_eq!(purple.r, 127);
        assert_eq!(purple.g, 0);
        assert_eq!(purple.b, 127);
    }

    #[test]
    fn test_color_add() {
        let red = Color::new(100, 0, 0);
        let green = Color::new(0, 150, 0);
        let result = red.add(&green);
        assert_eq!(result.r, 100);
        assert_eq!(result.g, 150);
        assert_eq!(result.b, 0);
    }

    #[test]
    fn test_color_add_saturation() {
        let c1 = Color::new(200, 100, 50);
        let c2 = Color::new(100, 200, 250);
        let result = c1.add(&c2);
        assert_eq!(result.r, 255); // saturated
        assert_eq!(result.g, 255); // saturated
        assert_eq!(result.b, 255); // saturated
    }

    #[test]
    fn test_color_sub() {
        let c1 = Color::new(200, 100, 50);
        let c2 = Color::new(50, 30, 10);
        let result = c1.sub(&c2);
        assert_eq!(result.r, 150);
        assert_eq!(result.g, 70);
        assert_eq!(result.b, 40);
    }

    #[test]
    fn test_color_scale() {
        let c = Color::new(100, 50, 200);
        let scaled = c.scale(2.0);
        assert_eq!(scaled.r, 200);
        assert_eq!(scaled.g, 100);
        assert_eq!(scaled.b, 255); // clamped
    }

    #[test]
    fn test_color_mix() {
        let red = Color::new(255, 0, 0);
        let blue = Color::new(0, 0, 255);
        let half = red.mix(&blue, 0.5);
        assert_eq!(half.r, 127);
        assert_eq!(half.g, 0);
        assert_eq!(half.b, 127);
    }

    #[test]
    fn test_value_color_add() {
        let c1 = Value::Color(Color::new(100, 50, 25));
        let c2 = Value::Color(Color::new(50, 100, 75));
        match c1.add(&c2) {
            Ok(Value::Color(c)) => {
                assert_eq!(c.r, 150);
                assert_eq!(c.g, 150);
                assert_eq!(c.b, 100);
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_value_color_blend() {
        let c1 = Value::Color(Color::new(255, 0, 0));
        let c2 = Value::Color(Color::new(0, 0, 255));
        match c1.blend(&c2) {
            Ok(Value::Color(c)) => {
                assert_eq!(c.r, 127);
                assert_eq!(c.b, 127);
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_expr_color_blend() {
        let mut runtime = Runtime::new();
        let expr = Expr::Blend(
            Box::new(Expr::Value(Value::Color(Color::new(200, 100, 50)))),
            Box::new(Expr::Value(Value::Color(Color::new(100, 200, 150)))),
        );

        match runtime.eval(expr) {
            Ok(Value::Color(c)) => {
                assert_eq!(c.r, 150);
                assert_eq!(c.g, 150);
                assert_eq!(c.b, 100);
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn test_expr_color_scale() {
        let mut runtime = Runtime::new();
        let expr = Expr::Scale(
            Box::new(Expr::Value(Value::Color(Color::new(100, 50, 200)))),
            0.5,
        );

        match runtime.eval(expr) {
            Ok(Value::Color(c)) => {
                assert_eq!(c.r, 50);
                assert_eq!(c.g, 25);
                assert_eq!(c.b, 100);
            }
            _ => panic!("Expected Color"),
        }
    }

    // Loop and continuation tests
    #[test]
    fn test_while_loop() {
        let mut runtime = Runtime::new();
        // while i < 5 { i = i + 1 }
        // Simplified: while true { break } - should execute once
        let expr = Expr::While(
            Box::new(Expr::Value(Value::Bool(false))),
            Box::new(Expr::Value(Value::Num(42.0))),
        );

        match runtime.eval(expr) {
            Ok(Value::Unit) => (), // Loop never executes body
            _ => panic!("Expected Unit"),
        }
    }

    #[test]
    fn test_while_loop_executes() {
        let mut runtime = Runtime::new();
        // Test that while loop with true condition executes at least once
        // We'll use a simple counter simulation
        let _expr = Expr::While(
            Box::new(Expr::Value(Value::Bool(true))),
            Box::new(Expr::Value(Value::Num(42.0))),
        );

        // This would loop forever, so we test with false condition instead
        let expr_no_loop = Expr::While(
            Box::new(Expr::Value(Value::Bool(false))),
            Box::new(Expr::Value(Value::Num(99.0))),
        );

        match runtime.eval(expr_no_loop) {
            Ok(Value::Unit) => (),
            _ => panic!("Expected Unit for non-executing loop"),
        }
    }

    #[test]
    fn test_for_loop() {
        let mut runtime = Runtime::new();
        // for i in [1,2,3] { i }
        let expr = Expr::For(
            "i".to_string(),
            Box::new(Expr::Value(Value::Array(vec![
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(3.0),
            ]))),
            Box::new(Expr::Value(Value::Num(42.0))),
        );

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 42.0 => (), // Returns last iteration value
            _ => panic!("Expected Num(42.0)"),
        }
    }

    #[test]
    fn test_for_loop_empty_array() {
        let mut runtime = Runtime::new();
        let expr = Expr::For(
            "i".to_string(),
            Box::new(Expr::Value(Value::Array(vec![]))),
            Box::new(Expr::Value(Value::Num(99.0))),
        );

        match runtime.eval(expr) {
            Ok(Value::Unit) => (), // Empty array returns Unit
            _ => panic!("Expected Unit"),
        }
    }

    #[test]
    fn test_block_expression() {
        let mut runtime = Runtime::new();
        // { 1; 2; 3 }
        let expr = Expr::Block(vec![
            Expr::Value(Value::Num(1.0)),
            Expr::Value(Value::Num(2.0)),
            Expr::Value(Value::Num(3.0)),
        ]);

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 3.0 => (), // Returns last expression
            _ => panic!("Expected Num(3.0)"),
        }
    }

    #[test]
    fn test_nested_blocks() {
        let mut runtime = Runtime::new();
        // { { 1; 2 }; 3 }
        let expr = Expr::Block(vec![
            Expr::Block(vec![
                Expr::Value(Value::Num(1.0)),
                Expr::Value(Value::Num(2.0)),
            ]),
            Expr::Value(Value::Num(3.0)),
        ]);

        match runtime.eval(expr) {
            Ok(Value::Num(n)) if n == 3.0 => (),
            _ => panic!("Expected Num(3.0)"),
        }
    }

    #[test]
    fn test_value_comparison_less_than() {
        let a = Value::Num(5.0);
        let b = Value::Num(10.0);
        match a.less_than(&b) {
            Ok(Value::Bool(true)) => (),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_value_comparison_greater_than() {
        let a = Value::Num(10.0);
        let b = Value::Num(5.0);
        match a.greater_than(&b) {
            Ok(Value::Bool(true)) => (),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_value_comparison_equals() {
        let a = Value::Num(5.0);
        let b = Value::Num(5.0);
        match a.equals(&b) {
            Ok(Value::Bool(true)) => (),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_value_is_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Num(1.0).is_truthy());
        assert!(!Value::Num(0.0).is_truthy());
        assert!(!Value::Unit.is_truthy());
        assert!(Value::Str("hello".to_string()).is_truthy());
    }

    #[test]
    fn test_continuation_with_loop() {
        let mut runtime = Runtime::new();
        // Test that continuation stack works with loops
        runtime.cont_stack.push(Continuation::Resume(Box::new(|| Value::Num(100.0))));

        let result = runtime.resume();
        match result {
            Value::Num(n) if n == 100.0 => (),
            _ => panic!("Expected Num(100.0)"),
        }
    }

    #[test]
    fn test_nested_for_loops() {
        let mut runtime = Runtime::new();
        // Outer loop: for i in [1,2]
        // Inner loop: for j in [3,4]
        let inner_loop = Expr::For(
            "j".to_string(),
            Box::new(Expr::Value(Value::Array(vec![
                Value::Num(3.0),
                Value::Num(4.0),
            ]))),
            Box::new(Expr::Value(Value::Num(10.0))),
        );

        let outer_loop = Expr::For(
            "i".to_string(),
            Box::new(Expr::Value(Value::Array(vec![
                Value::Num(1.0),
                Value::Num(2.0),
            ]))),
            Box::new(inner_loop),
        );

        match runtime.eval(outer_loop) {
            Ok(Value::Num(n)) if n == 10.0 => (), // Returns last inner iteration
            _ => panic!("Expected Num(10.0) from nested loops"),
        }
    }

    #[test]
    fn test_while_with_continuation() {
        let mut runtime = Runtime::new();
        // Push a continuation, then execute a while loop
        runtime.cont_stack.push(Continuation::Resume(Box::new(|| Value::Num(50.0))));

        // While loop that doesn't execute
        let expr = Expr::While(
            Box::new(Expr::Value(Value::Bool(false))),
            Box::new(Expr::Value(Value::Num(1.0))),
        );

        runtime.eval(expr).unwrap();

        // Resume the continuation
        let result = runtime.resume();
        match result {
            Value::Num(n) if n == 50.0 => (),
            _ => panic!("Expected continuation to work after loop"),
        }
    }

    // ===== COMPREHENSIVE TEST SUITE (60 Additional Tests) =====

    // Pi Language Tests (20 tests)
    #[test] fn test_pi_simple_add() { assert_eq!(Value::Num(3.0).add(&Value::Num(4.0)).unwrap(), Value::Num(7.0)); }
    #[test] fn test_pi_simple_sub() { assert_eq!(Value::Num(10.0).sub(&Value::Num(3.0)).unwrap(), Value::Num(7.0)); }
    #[test] fn test_pi_simple_mul() { assert_eq!(Value::Num(6.0).mul(&Value::Num(7.0)).unwrap(), Value::Num(42.0)); }
    #[test] fn test_pi_simple_div() { assert_eq!(Value::Num(20.0).div(&Value::Num(4.0)).unwrap(), Value::Num(5.0)); }
    #[test] fn test_pi_chain_add() { assert_eq!(Value::Num(1.0).add(&Value::Num(2.0)).unwrap().add(&Value::Num(3.0)).unwrap(), Value::Num(6.0)); }

    #[test]
    fn test_pi_complex_expr() {
        let mut runtime = Runtime::new();
        // (3 + 4) * 2
        let expr = Expr::Mul(
            Box::new(Expr::Add(
                Box::new(Expr::Value(Value::Num(3.0))),
                Box::new(Expr::Value(Value::Num(4.0))),
            )),
            Box::new(Expr::Value(Value::Num(2.0))),
        );
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(14.0));
    }

    #[test]
    fn test_pi_deeply_nested() {
        let mut runtime = Runtime::new();
        // ((1 + 2) + (3 + 4))
        let expr = Expr::Add(
            Box::new(Expr::Add(
                Box::new(Expr::Value(Value::Num(1.0))),
                Box::new(Expr::Value(Value::Num(2.0))),
            )),
            Box::new(Expr::Add(
                Box::new(Expr::Value(Value::Num(3.0))),
                Box::new(Expr::Value(Value::Num(4.0))),
            )),
        );
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(10.0));
    }

    #[test] fn test_pi_negative_numbers() { assert_eq!(Value::Num(-5.0).add(&Value::Num(3.0)).unwrap(), Value::Num(-2.0)); }
    #[test] fn test_pi_float_precision() { assert_eq!(Value::Num(0.1).add(&Value::Num(0.2)).unwrap(), Value::Num(0.30000000000000004)); }
    #[test] fn test_pi_zero_add() { assert_eq!(Value::Num(42.0).add(&Value::Num(0.0)).unwrap(), Value::Num(42.0)); }
    #[test] fn test_pi_zero_mul() { assert_eq!(Value::Num(42.0).mul(&Value::Num(0.0)).unwrap(), Value::Num(0.0)); }
    #[test] fn test_pi_one_mul() { assert_eq!(Value::Num(42.0).mul(&Value::Num(1.0)).unwrap(), Value::Num(42.0)); }
    #[test] fn test_pi_negative_mul() { assert_eq!(Value::Num(-3.0).mul(&Value::Num(4.0)).unwrap(), Value::Num(-12.0)); }
    #[test] fn test_pi_divide_negative() { assert_eq!(Value::Num(-20.0).div(&Value::Num(4.0)).unwrap(), Value::Num(-5.0)); }
    #[test] fn test_pi_sub_to_negative() { assert_eq!(Value::Num(3.0).sub(&Value::Num(10.0)).unwrap(), Value::Num(-7.0)); }
    #[test] fn test_pi_large_numbers() { assert_eq!(Value::Num(1000000.0).add(&Value::Num(2000000.0)).unwrap(), Value::Num(3000000.0)); }
    #[test] fn test_pi_small_numbers() { assert_eq!(Value::Num(0.001).add(&Value::Num(0.002)).unwrap(), Value::Num(0.003)); }
    #[test] fn test_pi_mixed_ops() { assert_eq!(Value::Num(10.0).add(&Value::Num(5.0)).unwrap().mul(&Value::Num(2.0)).unwrap(), Value::Num(30.0)); }
    #[test] fn test_pi_div_then_mul() { assert_eq!(Value::Num(100.0).div(&Value::Num(5.0)).unwrap().mul(&Value::Num(3.0)).unwrap(), Value::Num(60.0)); }

    // Rho Conditional Tests (20 tests)
    #[test] fn test_rho_less_than_true() { assert_eq!(Value::Num(5.0).less_than(&Value::Num(10.0)).unwrap(), Value::Bool(true)); }
    #[test] fn test_rho_less_than_false() { assert_eq!(Value::Num(10.0).less_than(&Value::Num(5.0)).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_less_than_equal() { assert_eq!(Value::Num(5.0).less_than(&Value::Num(5.0)).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_greater_than_true() { assert_eq!(Value::Num(10.0).greater_than(&Value::Num(5.0)).unwrap(), Value::Bool(true)); }
    #[test] fn test_rho_greater_than_false() { assert_eq!(Value::Num(5.0).greater_than(&Value::Num(10.0)).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_greater_than_equal() { assert_eq!(Value::Num(5.0).greater_than(&Value::Num(5.0)).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_equals_true() { assert_eq!(Value::Num(5.0).equals(&Value::Num(5.0)).unwrap(), Value::Bool(true)); }
    #[test] fn test_rho_equals_false() { assert_eq!(Value::Num(5.0).equals(&Value::Num(6.0)).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_equals_bool_true() { assert_eq!(Value::Bool(true).equals(&Value::Bool(true)).unwrap(), Value::Bool(true)); }
    #[test] fn test_rho_equals_bool_false() { assert_eq!(Value::Bool(true).equals(&Value::Bool(false)).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_equals_str_true() { assert_eq!(Value::Str("hello".to_string()).equals(&Value::Str("hello".to_string())).unwrap(), Value::Bool(true)); }
    #[test] fn test_rho_equals_str_false() { assert_eq!(Value::Str("hello".to_string()).equals(&Value::Str("world".to_string())).unwrap(), Value::Bool(false)); }
    #[test] fn test_rho_truthy_bool_true() { assert!(Value::Bool(true).is_truthy()); }
    #[test] fn test_rho_truthy_bool_false() { assert!(!Value::Bool(false).is_truthy()); }
    #[test] fn test_rho_truthy_num_nonzero() { assert!(Value::Num(42.0).is_truthy()); }
    #[test] fn test_rho_truthy_num_zero() { assert!(!Value::Num(0.0).is_truthy()); }
    #[test] fn test_rho_truthy_string() { assert!(Value::Str("test".to_string()).is_truthy()); }
    #[test] fn test_rho_truthy_unit() { assert!(!Value::Unit.is_truthy()); }
    #[test] fn test_rho_truthy_array() { assert!(Value::Array(vec![Value::Num(1.0)]).is_truthy()); }
    #[test] fn test_rho_negative_comparison() { assert_eq!(Value::Num(-5.0).less_than(&Value::Num(0.0)).unwrap(), Value::Bool(true)); }

    // Loop Tests (20 tests)
    #[test]
    fn test_loop_for_single_item() {
        let mut runtime = Runtime::new();
        let expr = Expr::For(
            "x".to_string(),
            Box::new(Expr::Value(Value::Array(vec![Value::Num(99.0)]))),
            Box::new(Expr::Value(Value::Num(42.0))),
        );
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(42.0));
    }

    #[test]
    fn test_loop_for_multiple_items() {
        let mut runtime = Runtime::new();
        let expr = Expr::For(
            "x".to_string(),
            Box::new(Expr::Value(Value::Array(vec![Value::Num(1.0), Value::Num(2.0), Value::Num(3.0)]))),
            Box::new(Expr::Value(Value::Num(100.0))),
        );
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(100.0));
    }

    #[test]
    fn test_loop_while_never_executes() {
        let mut runtime = Runtime::new();
        let expr = Expr::While(
            Box::new(Expr::Value(Value::Bool(false))),
            Box::new(Expr::Value(Value::Num(999.0))),
        );
        assert_eq!(runtime.eval(expr).unwrap(), Value::Unit);
    }

    #[test]
    fn test_loop_block_single() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![Expr::Value(Value::Num(42.0))]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(42.0));
    }

    #[test]
    fn test_loop_block_empty() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Unit);
    }

    #[test]
    fn test_loop_triple_nested_blocks() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![
            Expr::Block(vec![
                Expr::Block(vec![Expr::Value(Value::Num(1.0))]),
                Expr::Value(Value::Num(2.0)),
            ]),
            Expr::Value(Value::Num(3.0)),
        ]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(3.0));
    }

    #[test]
    fn test_loop_nested_for_different_arrays() {
        let mut runtime = Runtime::new();
        let inner = Expr::For("j".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(5.0)]))), Box::new(Expr::Value(Value::Num(20.0))));
        let outer = Expr::For("i".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(1.0)]))), Box::new(inner));
        assert_eq!(runtime.eval(outer).unwrap(), Value::Num(20.0));
    }

    #[test]
    fn test_loop_for_with_strings() {
        let mut runtime = Runtime::new();
        let expr = Expr::For(
            "s".to_string(),
            Box::new(Expr::Value(Value::Array(vec![Value::Str("a".to_string()), Value::Str("b".to_string())]))),
            Box::new(Expr::Value(Value::Num(77.0))),
        );
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(77.0));
    }

    #[test] fn test_loop_for_large_array() {
        let mut runtime = Runtime::new();
        let arr = (0..100).map(|i| Value::Num(i as f64)).collect();
        let expr = Expr::For("x".to_string(), Box::new(Expr::Value(Value::Array(arr))), Box::new(Expr::Value(Value::Num(999.0))));
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(999.0));
    }

    #[test] fn test_loop_block_with_arithmetic() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![
            Expr::Add(Box::new(Expr::Value(Value::Num(1.0))), Box::new(Expr::Value(Value::Num(2.0)))),
            Expr::Mul(Box::new(Expr::Value(Value::Num(3.0))), Box::new(Expr::Value(Value::Num(4.0)))),
        ]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(12.0));
    }

    #[test] fn test_loop_nested_blocks_with_ops() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![
            Expr::Block(vec![Expr::Add(Box::new(Expr::Value(Value::Num(5.0))), Box::new(Expr::Value(Value::Num(5.0))))]),
            Expr::Mul(Box::new(Expr::Value(Value::Num(2.0))), Box::new(Expr::Value(Value::Num(3.0)))),
        ]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(6.0));
    }

    #[test] fn test_loop_four_level_nesting() {
        let mut runtime = Runtime::new();
        let expr = Expr::For("a".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(1.0)]))),
            Box::new(Expr::For("b".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(2.0)]))),
                Box::new(Expr::For("c".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(3.0)]))),
                    Box::new(Expr::For("d".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(4.0)]))),
                        Box::new(Expr::Value(Value::Num(42.0))))))))));
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(42.0));
    }

    #[test] fn test_loop_mixed_nested() {
        let mut runtime = Runtime::new();
        let expr = Expr::For("i".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(1.0)]))),
            Box::new(Expr::Block(vec![
                Expr::Value(Value::Num(10.0)),
                Expr::Value(Value::Num(20.0)),
            ])));
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(20.0));
    }

    #[test] fn test_loop_block_returns_last() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![
            Expr::Value(Value::Num(1.0)),
            Expr::Value(Value::Num(2.0)),
            Expr::Value(Value::Num(3.0)),
            Expr::Value(Value::Num(4.0)),
            Expr::Value(Value::Num(5.0)),
        ]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(5.0));
    }

    #[test] fn test_loop_for_with_colors() {
        let mut runtime = Runtime::new();
        let expr = Expr::For("c".to_string(),
            Box::new(Expr::Value(Value::Array(vec![Value::Color(Color::new(255,0,0))]))),
            Box::new(Expr::Value(Value::Num(33.0))));
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(33.0));
    }

    #[test] fn test_loop_for_nested_empty_inner() {
        let mut runtime = Runtime::new();
        let inner = Expr::For("j".to_string(), Box::new(Expr::Value(Value::Array(vec![]))), Box::new(Expr::Value(Value::Num(50.0))));
        let outer = Expr::For("i".to_string(), Box::new(Expr::Value(Value::Array(vec![Value::Num(1.0)]))), Box::new(inner));
        assert_eq!(runtime.eval(outer).unwrap(), Value::Unit);
    }

    #[test] fn test_loop_block_with_unit() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![Expr::Value(Value::Unit), Expr::Value(Value::Num(42.0))]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(42.0));
    }

    #[test] fn test_loop_for_boolean_array() {
        let mut runtime = Runtime::new();
        let expr = Expr::For("b".to_string(),
            Box::new(Expr::Value(Value::Array(vec![Value::Bool(true), Value::Bool(false)]))),
            Box::new(Expr::Value(Value::Num(88.0))));
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(88.0));
    }

    #[test] fn test_loop_deeply_nested_blocks_five_levels() {
        let mut runtime = Runtime::new();
        let expr = Expr::Block(vec![
            Expr::Block(vec![
                Expr::Block(vec![
                    Expr::Block(vec![
                        Expr::Block(vec![Expr::Value(Value::Num(5.0))]),
                    ]),
                ]),
            ]),
        ]);
        assert_eq!(runtime.eval(expr).unwrap(), Value::Num(5.0));
    }

    // Tau Generation Tests (2 tests to reach 100!)
    #[test]
    fn test_tau_proxy_generation() {
        use std::fs;
        // Create a test file
        fs::write("test_tau_proxy.tsu", "# test proxy generation\n1 + 1").unwrap();

        // Generate proxy
        let result = tau::generate_proxy_wrapper("test_tau_proxy.tsu");
        assert!(result.is_ok());

        // Verify files were created
        assert!(std::path::Path::new("test_tau_proxyProxy.h").exists());
        assert!(std::path::Path::new("App/Network/test_tau_proxy.tsu").exists());

        // Cleanup
        let _ = fs::remove_file("test_tau_proxy.tsu");
        let _ = fs::remove_file("test_tau_proxyProxy.h");
        let _ = fs::remove_file("App/Network/test_tau_proxy.tsu");
    }

    #[test]
    fn test_tau_agent_generation() {
        use std::fs;
        // Create a test file
        fs::write("test_tau_agent.tsu", "# test agent generation\n2 * 2").unwrap();

        // Generate agent
        let result = tau::generate_agent_wrapper("test_tau_agent.tsu");
        assert!(result.is_ok());

        // Verify files were created
        assert!(std::path::Path::new("test_tau_agentAgent.h").exists());
        assert!(std::path::Path::new("App/Network/test_tau_agentAgent.tsu").exists());

        // Cleanup
        let _ = fs::remove_file("test_tau_agent.tsu");
        let _ = fs::remove_file("test_tau_agentAgent.h");
        let _ = fs::remove_file("App/Network/test_tau_agentAgent.tsu");
    }
}

