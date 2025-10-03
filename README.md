# RustAiLang - Language with Native Continuations

## File Extension: `.t`

## Features

### 1. Numeric Type (f64)
All numbers are 64-bit floating point:
```
5.5 + 3.2     # 8.7
10.0 * 2.5    # 25.0
20.0 / 4.0    # 5.0
```

### 2. Arrays `[...]`
Arrays hold any values:
```
[1,2,3]
[1.5,2.5,3.5]
[1,2] + [3,4]  # Concatenation: [1,2,3,4]
```

### 3. Maps `[{key,value},...]`
Maps store key-value pairs:
```
[{1,10},{2,20},{3,30}]
```

### 4. Colors `color(r,g,b)`
RGB colors (0-255):
```
color(255,0,0)              # Red
color(255,0,0) + color(0,255,0)  # Yellow
```

### 5. Bash Injection `` `command` ``
Execute bash commands with backticks:
```
`echo Hello`
`pwd`
`ls -la`
```

### 6. Control Flow
- `resume` - Execute what's on continuation stack
- `break` - Drop continuation stack
- `continue` - Takes continuation argument

### 7. Comments
Lines starting with `#` are comments:
```
# This is a comment
5 + 3  # This works too
```

## REPL Commands
- `:quit` or `:q` - Exit REPL
- `:help` or `:h` - Show help

## Running `.t` Files
```bash
cargo run < program.t
```
