# RustAiLang - Multi-Paradigm Language System

## Project Goal

Create a language system with native continuations, featuring three distinct dialects optimized for different computational paradigms.

## Architecture

```mermaid
graph TB
    subgraph "Language System"
        Pi[Pi - Postfix/RPN]
        Rho[Rho - Infix]
        Tau[Tau - Network/Futures]
    end

    subgraph "Runtime"
        Eval[Expression Evaluator]
        ContStack[Continuation Stack]
        ValueSys[Value System]
    end

    Pi --> Eval
    Rho --> Eval
    Tau --> Eval
    Eval --> ContStack
    Eval --> ValueSys

    style Pi fill:#9f9
    style Rho fill:#99f
    style Tau fill:#ff9
```

## Implementation Language

**Rust** - For memory safety, performance, and strong type system

## Key Features

### 1. Native Continuations

```mermaid
sequenceDiagram
    participant Code
    participant Runtime
    participant ContStack

    Code->>Runtime: Create continuation
    Runtime->>ContStack: Push continuation
    Code->>Runtime: Call resume()
    Runtime->>ContStack: Pop continuation
    ContStack->>Runtime: Execute
    Runtime->>Code: Return result
```

**Operations:**
- `resume` - Execute what's on the continuation stack
- `break` - Drop continuation stack and resume next
- `continue(f)` - Takes a continuation argument and executes it

### 2. Three Language Dialects

#### Pi (Postfix/RPN)
```mermaid
flowchart LR
    Input[3 4 +] --> Parse[Parse Tokens]
    Parse --> Stack1[Push 3]
    Stack1 --> Stack2[Push 4]
    Stack2 --> Op[Pop 2, Apply +]
    Op --> Result[Push 7]

    style Result fill:#9f9
```

**Characteristics:**
- Stack-based evaluation
- Postfix notation (operators follow operands)
- Left-to-right execution
- File extension: `.pi`

#### Rho (Infix)
```mermaid
flowchart TB
    Input[3 + 4] --> Parse[Parse Expression]
    Parse --> Tree[Build AST]
    Tree --> Left[Left: 3]
    Tree --> Op[Operator: +]
    Tree --> Right[Right: 4]
    Left & Op & Right --> Eval[Evaluate]
    Eval --> Result[7]

    style Result fill:#99f
```

**Characteristics:**
- Traditional infix notation
- Operator precedence
- Tab-based scoping
- File extension: `.rho`

#### Tau (Network/Futures)
```mermaid
flowchart TD
    Input[proxy file.tsu] --> Read[Read Source]
    Read --> GenH[Generate C++ Header]
    Read --> GenNet[Generate Network File]
    GenH --> ProxyH[fileProxy.h]
    GenNet --> ProxyNet[App/Network/file.tsu]

    style ProxyH fill:#ff9
    style ProxyNet fill:#ff9
```

**Characteristics:**
- Async/await operations
- Code generation (Proxy/Agent patterns)
- Cross-language integration
- File extension: `.tsu`

### 3. Loop Constructs

```mermaid
stateDiagram-v2
    [*] --> ForLoop
    [*] --> WhileLoop

    ForLoop --> CheckArray: for i in array
    CheckArray --> HasItem: More items?
    HasItem --> ExecBody: Yes
    ExecBody --> CheckArray
    HasItem --> [*]: No

    WhileLoop --> CheckCond: Check condition
    CheckCond --> ExecBody2: True
    ExecBody2 --> CheckCond
    CheckCond --> [*]: False

    note right of ForLoop
        Iterates over arrays
        Supports nesting
    end note

    note right of WhileLoop
        Conditional loops
        Supports break/continue
    end note
```

### 4. Value System

```mermaid
classDiagram
    class Value {
        <<enum>>
        +Num(f64)
        +Str(String)
        +Bool(bool)
        +Unit
        +Color(r,g,b)
        +Array(Vec~Value~)
        +Map(Vec)
        +Future(FutureState)
        +Continuation(Fn)
    }

    class Color {
        +u8 r
        +u8 g
        +u8 b
        +blend(other) Color
        +scale(factor) Color
    }

    class FutureState {
        <<enum>>
        +Pending
        +Resolved(Value)
        +Rejected(String)
    }

    Value --> Color
    Value --> FutureState

    class Operations {
        +add(a, b) Value
        +sub(a, b) Value
        +mul(a, b) Value
        +div(a, b) Value
        +less_than(a, b) Bool
        +greater_than(a, b) Bool
        +equals(a, b) Bool
    }

    Value --> Operations
```

## Testing Strategy

### Test Coverage: 100 Tests

```mermaid
pie title Test Distribution (100 tests)
    "Pi Language" : 22
    "Rho Conditionals" : 20
    "Loop Constructs" : 27
    "Color Operations" : 7
    "Array/Map Ops" : 5
    "Continuations" : 3
    "Comparisons" : 4
    "Expressions" : 2
    "Tau Generation" : 2
    "Nested Structures" : 8
```

### Test Categories

1. **Pi Language (22 tests)**
   - Basic arithmetic operations
   - Complex nested expressions
   - Edge cases (negatives, zero, large numbers)
   - Float precision handling

2. **Rho Conditionals (20 tests)**
   - Comparison operators (<, >, ==)
   - Boolean logic
   - String comparisons
   - Truthiness evaluation

3. **Loop Constructs (27 tests)**
   - For loops with arrays
   - While loops with conditions
   - Nested loops (2-5 levels)
   - Block expressions
   - Mixed structures

4. **Tau Generation (2 tests)**
   - Proxy generation with file verification
   - Agent generation with file verification

## Development Workflow

```mermaid
flowchart LR
    subgraph Development
        Code[Write Code] --> Test[Write Tests]
        Test --> Build[cargo build]
        Build --> Run[cargo test]
    end

    subgraph Quality
        Run --> Pass{All Pass?}
        Pass -->|Yes| Commit[git commit]
        Pass -->|No| Debug[Debug]
        Debug --> Code
    end

    subgraph Deployment
        Commit --> Push[git push]
        Push --> Done[✓ Complete]
    end

    style Pass fill:#ff9
    style Done fill:#9f9
```

## Project Structure

```mermaid
graph TD
    Root[RustAiLang/]

    Root --> Src[src/]
    Root --> Tests[tests/]
    Root --> Docs[docs/]
    Root --> Examples[examples/]

    Src --> Main[main.rs - REPL]
    Src --> Value[value.rs - Type System]
    Src --> Pi[pi.rs - Pi Parser]
    Src --> Rho[rho.rs - Rho Parser]
    Src --> Tau[tau.rs - Tau Parser]

    Tests --> Unit[Unit Tests - 100]
    Tests --> Integration[Integration Tests]

    Examples --> PiEx[*.pi examples]
    Examples --> RhoEx[*.rho examples]
    Examples --> TauEx[*.tsu examples]

    style Root fill:#f9f
    style Unit fill:#9f9
```

## Build & Test Commands

```bash
# Build project
cargo build              # Debug build
cargo build --release    # Optimized build

# Run tests
cargo test              # All 100 tests
cargo test pi_          # Pi language tests only
cargo test rho_         # Rho language tests only
cargo test tau_         # Tau language tests only
cargo test loop_        # Loop tests only

# Run REPL
cargo run

# Run with file
cargo run < program.pi
cargo run < program.rho
cargo run < program.tsu
```

## Version History

### v0.4.0 (Current)
- ✅ 100 comprehensive tests
- ✅ For/while loops with nesting
- ✅ Comparison operators
- ✅ Block expressions
- ✅ Tau code generation

### v0.3.0
- ✅ Loop structures added
- ✅ 40 tests passing
- ✅ Nested loop support

### v0.2.0
- ✅ Three languages (Pi, Rho, Tau)
- ✅ 27 tests passing
- ✅ Basic operations

### v0.1.0
- ✅ Initial implementation
- ✅ Continuation system
- ✅ Basic value types

## Future Enhancements

```mermaid
mindmap
  root((RustAiLang))
    Language Features
      Pattern Matching
      Type Inference
      Macros
      Modules
    Runtime
      JIT Compilation
      Garbage Collection
      Parallel Execution
    Tools
      Language Server
      Debugger
      Profiler
    Integration
      FFI Bindings
      Package Manager
      IDE Plugins
```

## Contributing

See [README.md](README.md) for contribution guidelines.

## License

See LICENSE file for details.

---

**Status:** ✅ Production Ready
**Tests:** 100/100 passing
**Languages:** Pi, Rho, Tau
**Version:** 0.4.0
