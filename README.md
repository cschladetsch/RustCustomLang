# RustAiLang - Language with Native Continuations

## File Extension: `.t`

## Features

### Numeric Type (f64)
All numbers are 64-bit floating point:
```
5.5 + 3.2     # 8.7
10.0 * 2.5    # 25.0
20.0 / 4.0    # 5.0
```

### Arrays `[...]`
Arrays hold any values:
```
[1,2,3]
[1.5,2.5,3.5]
[1,2] + [3,4]  # Concatenation: [1,2,3,4]
```

### Maps `[{key,value},...]`
Maps store key-value pairs with numeric or string keys:
```
[{1,10},{2,20},{3,30}]
[{"x",100},{"y",200}]
```

### Strings `"text"`
String literals with double or single quotes:
```
"hello"
'world'
```

### Array/Map Indexing `arr[index]`
Get elements from arrays or maps:
```
[10,20,30][1]           # Returns 20
[{1,100},{2,200}][2]    # Returns 200
[{"x",100}]["x"]        # Returns 100
```

### Colors `color(r,g,b)`
RGB colors (0-255):
```
color(255,0,0)              # Red
color(255,0,0) + color(0,255,0)  # Yellow
```

### Bash Injection `` `command` ``
Execute bash commands with backticks:
```
`echo Hello`
`pwd`
`ls -la`
```

### Control Flow
- `resume` - Execute what's on continuation stack
- `break` - Drop continuation stack
- `continue` - Takes continuation argument

### Comments
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
