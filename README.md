# Three Languages - Pi, Rho, Tau

A multi-paradigm language system with three dialects:

## File Extension: `.t`

## The Three Languages

### Pi - Postfix/RPN Notation
Stack-based postfix language (Reverse Polish Notation):
```
3 4 +              # 7 (postfix addition)
5 10 *             # 50 (postfix multiplication)
[1,2,3] "arr" =    # Variable assignment (postfix)
arr -->            # Stack operation: prints "1 2 3"
```

### Rho - Infix with Tab Indentation
Traditional infix notation with tab-based indentation:
```
3 + 4              # 7 (infix addition)
5 * 10             # 50 (infix multiplication)
arr = [1,2,3]      # Variable assignment (infix)
if a == 1          # Uses tabs for block structure
	doSomething
```

### Tau - Network Language with Futures
Asynchronous network operations returning futures:
```
async fetch        # Returns Future(Pending)
await result       # Resolves Future
```

## Switching Languages

Use REPL commands to switch between languages:
- `:pi` - Switch to Pi (postfix/RPN)
- `:rho` - Switch to Rho (infix+tabs)
- `:tau` - Switch to Tau (network+futures)

## Common Features

### Numeric Type (f64)
All numbers are 64-bit floating point

### Arrays `[...]`
```
[1,2,3]
[1,2] + [3,4]  # Concatenation
```

### Maps `[{key,value},...]`
```
[{1,10},{2,20}]
[{"x",100},{"y",200}]
```

### Strings `"text"`
```
"hello"
'world'
```

### Array/Map Indexing
```
[10,20,30][1]           # 20
[{"x",100}]["x"]        # 100
```

### Colors `color(r,g,b)`
```
color(255,0,0)              # Red
color(255,0,0) + color(0,255,0)  # Yellow
```

### Bash Injection `` `command` ``
```
`echo Hello`
`pwd`
`ls -la`
```

### Control Flow
- `resume` - Execute continuation stack
- `break` - Drop continuation stack
- `continue` - Takes continuation argument

### Comments
```
# This is a comment
```

## REPL Commands
- `:quit` or `:q` - Exit
- `:help` or `:h` - Show help
- `:pi` - Switch to Pi
- `:rho` - Switch to Rho
- `:tau` - Switch to Tau

## Running `.t` Files
```bash
cargo run < program.t
```

## Example Session
```
> :pi
Switched to Pi (postfix/RPN notation)
> 3 4 +
Num(7.0)
> :rho
Switched to Rho (infix with tab indentation)
> 3 + 4
Num(7.0)
> :tau
Switched to Tau (network language with futures)
> async fetch
Future(Pending)
```
