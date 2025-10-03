# Three Languages - Pi, Rho, Tau

A multi-paradigm language system with three dialects, each optimized for different computational paradigms.

## Architecture Overview

```mermaid
graph TB
    REPL[REPL Engine]
    REPL -->|:pi| Pi[Pi Parser<br/>Postfix/RPN]
    REPL -->|:rho| Rho[Rho Parser<br/>Infix]
    REPL -->|:tau| Tau[Tau Parser<br/>Network/Futures]

    Pi --> Runtime[Runtime Engine]
    Rho --> Runtime
    Tau --> Runtime
    Tau --> ProxyGen[Proxy Generator]
    Tau --> AgentGen[Agent Generator]

    ProxyGen --> CppHeader[C++ Headers]
    ProxyGen --> Network[App/Network Files]
    AgentGen --> CppHeader
    AgentGen --> Network

    Runtime --> Value[Value System]
    Value --> Num[Numbers f64]
    Value --> Arr[Arrays]
    Value --> Map[Maps]
    Value --> Color[Colors RGB]
    Value --> Fut[Futures]
    Value --> Cont[Continuations]

    style REPL fill:#f9f,stroke:#333,stroke-width:4px
    style Tau fill:#ff9,stroke:#333,stroke-width:2px
    style ProxyGen fill:#9f9,stroke:#333,stroke-width:2px
    style AgentGen fill:#9f9,stroke:#333,stroke-width:2px
```

## File Extensions
- Pi (postfix): `.pi`
- Rho (infix): `.rho`
- Tau (network): `.tsu`

## Language Selection Flow

```mermaid
stateDiagram-v2
    [*] --> Pi: Default
    Pi --> Rho: :rho
    Rho --> Pi: :pi
    Pi --> Tau: :tau
    Rho --> Tau: :tau
    Tau --> Pi: :pi
    Tau --> Rho: :rho
    Pi --> [*]: :quit
    Rho --> [*]: :quit
    Tau --> [*]: :quit
```

## The Three Languages

### Pi - Postfix/RPN Notation

Stack-based postfix language (Reverse Polish Notation):

```mermaid
graph LR
    A[3] --> Stack1[Stack: 3]
    B[4] --> Stack2[Stack: 3, 4]
    C[+] --> Stack3[Stack: 7]

    style Stack3 fill:#9f9
```

**Examples:**
```
3 4 +              # 7 (postfix addition)
5 10 *             # 50 (postfix multiplication)
[1,2,3] "arr" =    # Variable assignment (postfix)
arr -->            # Stack operation: prints "1 2 3"
```

**Execution Model:**
- Values pushed onto stack
- Operators pop operands, push results
- Left-to-right evaluation

### Rho - Infix with Tab Indentation

Traditional infix notation with tab-based indentation:

```mermaid
graph TB
    Expr[3 + 4]
    Expr --> Parse[Parse Tree]
    Parse --> Left[3]
    Parse --> Op[+]
    Parse --> Right[4]
    Parse --> Eval[Evaluate: 7]

    style Eval fill:#9f9
```

**Examples:**
```
3 + 4              # 7 (infix addition)
5 * 10             # 50 (infix multiplication)
arr = [1,2,3]      # Variable assignment (infix)
if a == 1          # Uses tabs for block structure
	doSomething
```

**Features:**
- Natural mathematical notation
- Operator precedence
- Tab-based scoping

### Tau - Network Language with Futures

Asynchronous network operations with code generation:

```mermaid
sequenceDiagram
    participant User
    participant Tau
    participant FileSystem
    participant Generated

    User->>Tau: proxy "myfile.tsu"
    Tau->>FileSystem: Read myfile.tsu
    Tau->>Generated: Create myfileProxy.h
    Tau->>Generated: Create App/Network/myfile.tsu
    Generated-->>User: Proxy generated!

    User->>Tau: agent "myfile.tsu"
    Tau->>FileSystem: Read myfile.tsu
    Tau->>Generated: Create myfileAgent.h
    Tau->>Generated: Create App/Network/myfileAgent.tsu
    Generated-->>User: Agent generated!
```

**Examples:**
```
async fetch        # Returns Future(Pending)
await result       # Resolves Future
proxy "file.tsu"   # Generates proxy wrapper for file
agent "file.tsu"   # Generates autonomous agent for file
```

**Generated Structure:**
```
proxy "mycode.tsu" creates:
├── myCodeProxy.h              # C++ proxy header
└── App/Network/mycode.tsu     # Network implementation

agent "mycode.tsu" creates:
├── myCodeAgent.h              # C++ agent header
└── App/Network/mycodeAgent.tsu # Async wrapper
```

## Switching Languages

Use REPL commands to switch between languages:
- `:pi` - Switch to Pi (postfix/RPN)
- `:rho` - Switch to Rho (infix+tabs)
- `:tau` - Switch to Tau (network+futures)

## Common Features

### Value Type System

```mermaid
classDiagram
    class Value {
        <<enumeration>>
        Num(f64)
        Str(String)
        Array(Vec~Value~)
        Map(Vec~(Value, Value)~)
        Color(r, g, b)
        Future(FutureState)
        Continuation(Fn)
        Bool(bool)
        Unit
    }

    class Color {
        +u8 r
        +u8 g
        +u8 b
        +blend(other) Color
        +scale(factor) Color
        +add(other) Color
        +sub(other) Color
    }

    class FutureState {
        <<enumeration>>
        Pending
        Resolved(Value)
        Rejected(String)
    }

    Value --> Color
    Value --> FutureState

    style Value fill:#f9f
    style Color fill:#ff9
    style FutureState fill:#9ff
```

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

```mermaid
graph LR
    Red[color 255,0,0<br/>Red]
    Green[color 0,255,0<br/>Green]
    Yellow[color 255,255,0<br/>Yellow]

    Red -->|+| Blend[Add Colors]
    Green -->|+| Blend
    Blend --> Yellow

    style Red fill:#f00,color:#fff
    style Green fill:#0f0
    style Yellow fill:#ff0
```

**Examples:**
```
color(255,0,0)                      # Red
color(255,0,0) + color(0,255,0)     # Yellow (additive)
color(200,100,50).blend(color(100,200,150))  # Blended color
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

## Running Files
```bash
cargo run < program.pi   # Pi files
cargo run < program.rho  # Rho files
cargo run < program.tsu  # Tau files
```

## Data Flow Example

```mermaid
flowchart TD
    Start([User Input]) --> Check{Which Language?}

    Check -->|:pi| PiParse[Pi Parser<br/>Stack-based]
    Check -->|:rho| RhoParse[Rho Parser<br/>Infix]
    Check -->|:tau| TauParse[Tau Parser<br/>Network]

    PiParse --> PiStack[Push to Stack<br/>Pop operators]
    RhoParse --> RhoTree[Build Parse Tree]
    TauParse --> TauCheck{Command?}

    TauCheck -->|proxy| ProxyGen[Generate Proxy]
    TauCheck -->|agent| AgentGen[Generate Agent]
    TauCheck -->|async| FutureCreate[Create Future]

    PiStack --> Eval[Runtime Evaluate]
    RhoTree --> Eval
    FutureCreate --> Eval

    ProxyGen --> Files[Write Files]
    AgentGen --> Files

    Eval --> Result([Display Result])
    Files --> Result

    style Start fill:#9f9
    style Result fill:#9f9
    style Eval fill:#f99
    style Files fill:#99f
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
> proxy "mycode.tsu"
Str("Proxy generated: mycodeProxy.h and App/Network/mycode.tsu")
```

## Project Structure

```mermaid
graph TD
    Root[RustAiLang/]
    Root --> Src[src/]
    Root --> Cargo[Cargo.toml]
    Root --> README[README.md]
    Root --> Examples[*.pi, *.rho, *.tsu]
    Root --> Generated[Generated Files]

    Src --> Main[main.rs<br/>REPL & Runtime]
    Src --> Value[value.rs<br/>Type System]
    Src --> Pi[pi.rs<br/>Pi Parser]
    Src --> Rho[rho.rs<br/>Rho Parser]
    Src --> Tau[tau.rs<br/>Tau Parser]

    Generated --> Headers[*.h C++ Headers]
    Generated --> AppNet[App/Network/<br/>Implementations]

    style Root fill:#f9f
    style Src fill:#9f9
    style Generated fill:#99f
```

## Building and Testing

### Build the Project
```bash
cargo build          # Debug build
cargo build --release # Optimized build
```

### Run Tests
```bash
cargo test          # Run all 27 unit tests
```

### Test Coverage

```mermaid
pie title Test Coverage by Component
    "Arithmetic Operations" : 4
    "Color Operations" : 7
    "Array/Map Operations" : 5
    "Continuation System" : 3
    "Expression Evaluation" : 2
    "Future/Value Tests" : 6
```

**Test Results:**
- ✅ 27 tests passing
- ✅ 0 failures
- Coverage: Runtime, Colors, Arrays, Maps, Continuations, Futures

## Advanced Features

### Continuation System

```mermaid
graph TB
    Cont[Continuation Stack]
    Cont --> Resume[resume<br/>Execute stack]
    Cont --> Break[break<br/>Clear stack]
    Cont --> Continue[continue<br/>Push & execute]

    Resume --> Exec[Execute top continuation]
    Break --> Clear[Clear all continuations]
    Continue --> Push[Push continuation<br/>then resume]

    style Cont fill:#f9f
    style Resume fill:#9f9
    style Break fill:#f99
    style Continue fill:#99f
```

**Operations:**
- `resume` - Execute what's on the continuation stack
- `break` - Drop continuation stack and resume next
- `continue(f)` - Takes a continuation argument and executes it

### Future States

```mermaid
stateDiagram-v2
    [*] --> Pending: async operation
    Pending --> Resolved: Success
    Pending --> Rejected: Error
    Resolved --> [*]: await
    Rejected --> [*]: error handler

    note right of Pending
        Future created with
        async keyword
    end note

    note right of Resolved
        Contains wrapped
        Value result
    end note
```

## Use Cases

### 1. Stack-Based Calculations (Pi)
- Forth-style programming
- RPN calculators
- Stack machine simulations

### 2. Natural Expressions (Rho)
- Mathematical computations
- Algorithm prototyping
- Familiar syntax for quick scripts

### 3. Network Code Generation (Tau)
- Proxy pattern generation
- Agent-based systems
- Async/await patterns
- Cross-language integration (Rust → C++)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## License

See LICENSE file for details.

---

**Version:** 0.2.0
**Languages:** Pi (`.pi`), Rho (`.rho`), Tau (`.tsu`)
**Status:** ✅ All 27 tests passing
