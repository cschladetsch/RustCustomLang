# Architecture Documentation

## System Overview

```mermaid
C4Context
    title System Context - RustAiLang

    Person(user, "Developer", "Uses RustAiLang for<br/>programming tasks")

    System(rustailang, "RustAiLang", "Multi-paradigm language<br/>with continuations")

    System_Ext(github, "GitHub", "Source code hosting")
    System_Ext(cargo, "Cargo", "Rust package manager")

    Rel(user, rustailang, "Writes code in", "Pi/Rho/Tau")
    Rel(rustailang, github, "Hosted on")
    Rel(rustailang, cargo, "Built with")
```

## Component Architecture

```mermaid
graph TB
    subgraph "User Interface"
        REPL[REPL Engine]
        CLI[Command Line Interface]
    end

    subgraph "Language Frontend"
        PiParser[Pi Parser<br/>Postfix]
        RhoParser[Rho Parser<br/>Infix]
        TauParser[Tau Parser<br/>Network]
    end

    subgraph "Core Runtime"
        Evaluator[Expression Evaluator]
        ContStack[Continuation Stack]
        Memory[Value Storage]
    end

    subgraph "Backend"
        Compiler[Expression Compiler]
        CodeGen[Code Generator<br/>Tau]
    end

    REPL --> PiParser
    REPL --> RhoParser
    REPL --> TauParser
    CLI --> REPL

    PiParser --> Evaluator
    RhoParser --> Evaluator
    TauParser --> Evaluator
    TauParser --> CodeGen

    Evaluator --> ContStack
    Evaluator --> Memory
    Evaluator --> Compiler

    CodeGen --> FileSystem[File System]

    style REPL fill:#f9f
    style Evaluator fill:#ff9
    style CodeGen fill:#9f9
```

## Data Flow

```mermaid
flowchart TD
    Input[User Input] --> LangDetect{Language?}

    LangDetect -->|:pi| PiFlow[Pi Processing]
    LangDetect -->|:rho| RhoFlow[Rho Processing]
    LangDetect -->|:tau| TauFlow[Tau Processing]

    PiFlow --> PiParse[Parse Postfix]
    PiParse --> PiStack[Stack Operations]
    PiStack --> Eval[Evaluate]

    RhoFlow --> RhoParse[Parse Infix]
    RhoParse --> RhoAST[Build AST]
    RhoAST --> Eval

    TauFlow --> TauCheck{Command Type?}
    TauCheck -->|proxy/agent| Generate[Code Generation]
    TauCheck -->|async/await| Eval

    Generate --> WriteFiles[Write to Disk]

    Eval --> ContCheck{Continuations?}
    ContCheck -->|Yes| ContProc[Process Continuations]
    ContCheck -->|No| Result[Return Result]
    ContProc --> Result

    Result --> Display[Display to User]

    style Eval fill:#ff9
    style Generate fill:#9f9
    style Result fill:#9ff
```

## Module Structure

```mermaid
classDiagram
    class main {
        +Repl
        +Runtime
        +Language enum
        +Expr enum
        +main()
    }

    class value {
        +Value enum
        +Color struct
        +FutureState enum
        +Continuation enum
    }

    class pi {
        +parse_pi()
    }

    class rho {
        +parse_rho()
    }

    class tau {
        +parse_tau()
        +generate_proxy()
        +generate_agent()
    }

    main --> value
    main --> pi
    main --> rho
    main --> tau

    pi --> value
    rho --> value
    tau --> value
```

## Expression Evaluation Pipeline

```mermaid
sequenceDiagram
    participant User
    participant REPL
    participant Parser
    participant Eval
    participant Runtime

    User->>REPL: Enter expression
    REPL->>Parser: Parse input
    Parser->>Parser: Tokenize
    Parser->>Parser: Build Expr tree
    Parser->>Eval: Expr
    Eval->>Runtime: Execute

    alt Has continuations
        Runtime->>Runtime: Push to stack
        Runtime->>Runtime: Resume
    end

    Runtime->>Eval: Result
    Eval->>REPL: Value
    REPL->>User: Display
```

## Continuation Stack Model

```mermaid
graph LR
    subgraph "Continuation Stack"
        Top[Top: Current]
        Mid1[Cont 1]
        Mid2[Cont 2]
        Bottom[Bottom: Base]
    end

    subgraph "Operations"
        Push[Push]
        Pop[Pop/Resume]
        Clear[Break/Clear]
    end

    Push -->|Add| Top
    Pop -->|Remove & Execute| Top
    Clear -->|Empty| Bottom

    style Top fill:#9f9
    style Push fill:#99f
    style Pop fill:#f99
    style Clear fill:#ff9
```

## Value Type Hierarchy

```mermaid
graph TD
    Value[Value Enum]

    Value --> Num[Num - f64]
    Value --> Str[Str - String]
    Value --> Bool[Bool - bool]
    Value --> Unit[Unit]
    Value --> Color[Color - RGB]
    Value --> Array[Array - Vec]
    Value --> Map[Map - Vec Pairs]
    Value --> Future[Future - State]
    Value --> Cont[Continuation - Fn]

    Color --> R[r: u8]
    Color --> G[g: u8]
    Color --> B[b: u8]

    Future --> Pending[Pending]
    Future --> Resolved[Resolved Value]
    Future --> Rejected[Rejected String]

    Array --> Elements[Vec~Value~]
    Map --> Pairs[Vec~Value,Value~]

    style Value fill:#f9f
    style Color fill:#ff9
    style Future fill:#9ff
```

## Loop Execution Model

```mermaid
stateDiagram-v2
    [*] --> ParseLoop

    ParseLoop --> ForLoop: for var in array
    ParseLoop --> WhileLoop: while condition
    ParseLoop --> Block: { expressions }

    ForLoop --> GetArray: Evaluate array
    GetArray --> CheckItem: Has next item?
    CheckItem --> BindVar: Yes - bind variable
    BindVar --> ExecBody: Execute body
    ExecBody --> CheckItem
    CheckItem --> ReturnLast: No - return last value
    ReturnLast --> [*]

    WhileLoop --> EvalCond: Evaluate condition
    EvalCond --> CheckTrue: Check truthiness
    CheckTrue --> ExecWhileBody: True - execute
    ExecWhileBody --> EvalCond
    CheckTrue --> ReturnUnit: False - return Unit
    ReturnUnit --> [*]

    Block --> ExecSeq: Execute sequentially
    ExecSeq --> NextExpr: More expressions?
    NextExpr --> ExecSeq: Yes
    NextExpr --> ReturnLastB: No
    ReturnLastB --> [*]

    note right of ForLoop
        Iterates over array items
        Returns last body result
    end note

    note right of WhileLoop
        Loops while condition true
        Returns Unit on exit
    end note
```

## Tau Code Generation Flow

```mermaid
flowchart TD
    Input[Tau Command] --> Parse{Command Type?}

    Parse -->|proxy| ProxyFlow[Proxy Generation]
    Parse -->|agent| AgentFlow[Agent Generation]

    ProxyFlow --> ReadSource[Read .tsu file]
    ReadSource --> ExtractName[Extract base name]
    ExtractName --> CreateDir[Create App/Network/]
    CreateDir --> GenProxyH[Generate Proxy.h]
    CreateDir --> GenProxyTsu[Generate Network .tsu]

    AgentFlow --> ReadSource2[Read .tsu file]
    ReadSource2 --> ExtractName2[Extract base name]
    ExtractName2 --> CreateDir2[Create App/Network/]
    CreateDir2 --> GenAgentH[Generate Agent.h]
    CreateDir2 --> GenAgentTsu[Generate Agent .tsu]

    GenProxyH --> WriteH1[Write C++ header]
    GenProxyTsu --> WriteNet1[Write network file]
    GenAgentH --> WriteH2[Write C++ header]
    GenAgentTsu --> WriteNet2[Write network file]

    WriteH1 & WriteNet1 --> DoneProxy[Proxy Complete]
    WriteH2 & WriteNet2 --> DoneAgent[Agent Complete]

    style GenProxyH fill:#9f9
    style GenAgentH fill:#99f
```

## Memory Management

```mermaid
graph TB
    subgraph "Stack Memory"
        CallStack[Call Stack]
        LocalVars[Local Variables]
    end

    subgraph "Heap Memory"
        Values[Value Objects]
        Arrays[Array Storage]
        Strings[String Storage]
    end

    subgraph "Rust Ownership"
        Owner[Owner]
        Borrower[Borrower]
        Lifetime[Lifetimes]
    end

    CallStack --> LocalVars
    LocalVars --> Owner
    Owner --> Values
    Values --> Arrays
    Values --> Strings

    Owner --> Borrower
    Borrower --> Lifetime

    style Owner fill:#9f9
    style Values fill:#99f
```

## Error Handling Flow

```mermaid
flowchart TD
    Operation[Operation] --> Try{Try Execute}

    Try -->|Success| Result[Ok Value]
    Try -->|Error| Catch[Catch Error]

    Catch --> ParseErr{Error Type?}

    ParseErr -->|Parse Error| ParseMsg[Invalid syntax message]
    ParseErr -->|Runtime Error| RuntimeMsg[Runtime error message]
    ParseErr -->|Type Error| TypeMsg[Type mismatch message]
    ParseErr -->|IO Error| IOMsg[File system error]

    ParseMsg & RuntimeMsg & TypeMsg & IOMsg --> Display[Display to user]
    Result --> Display

    Display --> Continue{Continue?}
    Continue -->|Yes| REPL[Return to REPL]
    Continue -->|No| Exit[Exit]

    style Catch fill:#f99
    style Result fill:#9f9
```

## Performance Profile

```mermaid
pie title Execution Time Distribution
    "Parsing" : 20
    "Evaluation" : 50
    "Memory Ops" : 15
    "I/O Operations" : 10
    "Other" : 5
```

## Concurrency Model

```mermaid
sequenceDiagram
    participant Main
    participant Async
    participant Future
    participant Executor

    Main->>Async: async operation
    Async->>Future: Create Future(Pending)
    Future->>Executor: Register task

    Executor->>Executor: Process

    alt Success
        Executor->>Future: Resolve(value)
        Future->>Main: Resolved
    else Failure
        Executor->>Future: Reject(error)
        Future->>Main: Rejected
    end
```

---

**Document Version:** 1.0
**Last Updated:** 2025
**Status:** Current Architecture
