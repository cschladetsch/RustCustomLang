use std::fmt;
use std::io::{self, Write};
use std::process::Command;

// Continuation type - represents a suspended computation
enum Continuation {
    Resume(Box<dyn Fn() -> Value + 'static>),
    Empty,
}

impl fmt::Debug for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Continuation::Resume(_) => write!(f, "Continuation::Resume"),
            Continuation::Empty => write!(f, "Continuation::Empty"),
        }
    }
}

// Color type - RGB with 0-255 values
#[derive(Debug, Clone, Copy, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    // Blend two colors (average)
    fn blend(&self, other: &Color) -> Color {
        Color {
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8,
        }
    }

    // Mix two colors with a ratio (0.0 = all self, 1.0 = all other)
    fn mix(&self, other: &Color, ratio: f32) -> Color {
        let ratio = ratio.clamp(0.0, 1.0);
        let inv_ratio = 1.0 - ratio;
        Color {
            r: (self.r as f32 * inv_ratio + other.r as f32 * ratio) as u8,
            g: (self.g as f32 * inv_ratio + other.g as f32 * ratio) as u8,
            b: (self.b as f32 * inv_ratio + other.b as f32 * ratio) as u8,
        }
    }

    // Add colors (clamped to 255)
    fn add(&self, other: &Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }

    // Subtract colors (clamped to 0)
    fn sub(&self, other: &Color) -> Color {
        Color {
            r: self.r.saturating_sub(other.r),
            g: self.g.saturating_sub(other.g),
            b: self.b.saturating_sub(other.b),
        }
    }

    // Scale color by a factor
    fn scale(&self, factor: f32) -> Color {
        Color {
            r: (self.r as f32 * factor).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * factor).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * factor).clamp(0.0, 255.0) as u8,
        }
    }
}

// Value types in the language
#[derive(Debug)]
enum Value {
    Num(f64),
    Bool(bool),
    Unit,
    Color(Color),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Continuation(Box<Continuation>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Num(n) => Value::Num(*n),
            Value::Bool(b) => Value::Bool(*b),
            Value::Unit => Value::Unit,
            Value::Color(c) => Value::Color(*c),
            Value::Array(a) => Value::Array(a.clone()),
            Value::Map(m) => Value::Map(m.clone()),
            Value::Continuation(_) => Value::Unit, // Can't clone closures
        }
    }
}

impl Value {
    fn as_num(&self) -> Result<f64, String> {
        match self {
            Value::Num(n) => Ok(*n),
            _ => Err(format!("Expected number, got {:?}", self)),
        }
    }
}

// Arithmetic operations on values
impl Value {
    fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.add(b))),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::Array(result))
            }
            _ => Err(format!("Cannot add {:?} and {:?}", self, other)),
        }
    }

    fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),
            (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.sub(b))),
            _ => Err(format!("Cannot subtract {:?} and {:?}", self, other)),
        }
    }

    fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
            _ => Err(format!("Cannot multiply {:?} and {:?}", self, other)),
        }
    }

    fn div(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) if *b == 0.0 => Err("Division by zero".to_string()),
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a / b)),
            _ => Err(format!("Cannot divide {:?} and {:?}", self, other)),
        }
    }

    // Color-specific operations
    fn blend(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Color(a), Value::Color(b)) => Ok(Value::Color(a.blend(b))),
            _ => Err(format!("Cannot blend {:?} and {:?}", self, other)),
        }
    }

    fn scale(&self, factor: f32) -> Result<Value, String> {
        match self {
            Value::Color(c) => Ok(Value::Color(c.scale(factor))),
            _ => Err(format!("Cannot scale {:?}", self)),
        }
    }
}

// Expression types
#[derive(Debug)]
enum Expr {
    Value(Value),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    // Color operations
    Blend(Box<Expr>, Box<Expr>),   // Blend two colors
    Scale(Box<Expr>, f32),          // Scale a color
    // Continuation algebra operations
    Compose(Box<Expr>, Box<Expr>),  // Continuation composition: c1 ; c2
    Choice(Box<Expr>, Box<Expr>),   // Continuation choice: c1 | c2
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
        }
    }
}

// REPL - Read-Eval-Print Loop
struct Repl {
    runtime: Runtime,
}

impl Repl {
    fn new() -> Self {
        Repl {
            runtime: Runtime::new(),
        }
    }

    fn run(&mut self) {
        println!("RustAiLang REPL v0.1.0");
        println!("Commands: :quit, :help");
        println!("Use `command` to execute bash commands\n");

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

            // Parse and evaluate expressions
            match self.parse_and_eval(input) {
                Ok(value) => println!("{:?}", value),
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    fn print_help(&self) {
        println!("RustAiLang REPL Help:");
        println!("  Arithmetic: 3 + 4, 10 - 2, 5 * 6, 20 / 4");
        println!("  Colors: color(255,0,0), blend(...), scale(...)");
        println!("  Bash: `ls`, `echo hello`, `pwd`");
        println!("  Commands: :quit, :help");
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
}
