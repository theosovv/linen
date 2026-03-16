# Linen Architecture Specification

This document describes the internal architecture of the Linen language implementation.

---

## 1. Compiler Pipeline

### 1.1 Overview

```
Source (.ln)
    ↓
Tokenizer (chumsky) → Token stream
    ↓
Parser (chumsky) → Green Tree (CST)
    ↓
AST Lowering → Untyped AST
    ↓
Macro Expansion (compile-time)
    ↓
Type Checker (HM + Linear) → Typed AST
    ↓
IR Generation (ANF/CPS)
    ↓
Optimizations (constant fold, inlining)
    ↓
Code Generation → Bytecode (.lnc)
    ↓
VM / JIT Execution
```

### 1.2 Pipeline Stages

| Stage | Component | Technology | Responsibility |
|-------|-----------|------------|----------------|
| 1 | Lexer | `chumsky` | Tokenization with error recovery |
| 2 | Parser | `chumsky` | CST → Untyped AST |
| 3 | Macro Engine | Built-in | Compile-time expansion |
| 4 | Type Checker | Custom HM | Inference + Linear type checking |
| 5 | IR Gen | Custom | AST → Intermediate Representation |
| 6 | Optimizer | Custom | Constant folding, inlining, DCE |
| 7 | CodeGen | Custom | IR → Bytecode |
| 8 | VM | Stack-based | Interpretation / JIT |

---

## 2. AST Format

The Abstract Syntax Tree (AST) is the central intermediate representation used by the compiler frontend.

### 2.1 Source Locations

Every AST node carries span information for error reporting:

```rust
/// Byte offset into the source file
pub type Offset = u32;

/// Span from start (inclusive) to end (exclusive)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Offset,
    pub end: Offset,
}

/// AST node with metadata
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub span: Span,
    pub data: T,
}
```

### 2.2 AST Nodes

```rust
/// Module is the top-level construct
#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
}

/// Import statement
#[derive(Debug)]
pub struct Import {
    pub span: Span,
    pub path: Vec<String>,  // e.g., ["std", "audio"]
    pub alias: Option<String>,
}

/// Top-level declarations
#[derive(Debug)]
pub enum Declaration {
    Let(LetBinding),
    LetRec(Vec<LetBinding>),  // Mutually recursive
    Type(TypeDeclaration),
    Foreign(ForeignDeclaration),
}

/// let binding: let name params* : type = expr
#[derive(Debug)]
pub struct LetBinding {
    pub span: Span,
    pub name: String,
    pub params: Vec<Parameter>,
    pub type_ann: Option<Type>,
    pub body: Expr,
}

/// Function parameter
#[derive(Debug)]
pub struct Parameter {
    pub span: Span,
    pub name: String,
    pub type_ann: Option<Type>,
}

/// Type declaration: type Name params = constructors
#[derive(Debug)]
pub struct TypeDeclaration {
    pub span: Span,
    pub name: String,
    pub params: Vec<String>,
    pub constructors: Vec<Constructor>,
}

/// Data constructor
#[derive(Debug)]
pub struct Constructor {
    pub span: Span,
    pub name: String,
    pub fields: Vec<Type>,
}

/// Foreign function declaration (built-in FFI)
#[derive(Debug)]
pub struct ForeignDeclaration {
    pub span: Span,
    pub name: String,
    pub type_sig: Type,
    pub abi: Abi,
}

#[derive(Debug, Clone, Copy)]
pub enum Abi {
    C,
    Internal,
}
```

### 2.3 Expressions

```rust
#[derive(Debug)]
pub enum Expr {
    /// Variable reference
    Var(String),
    
    /// Literal value
    Literal(Literal),
    
    /// Function application: f x y
    App(Box<Expr>, Vec<Expr>),
    
    /// Lambda abstraction: \x -> e
    Lambda(Vec<Parameter>, Box<Expr>),
    
    /// Let binding: let x = e1 in e2
    Let(LetBinding, Box<Expr>),
    
    /// If-then-else: if c then t else e
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    
    /// Pattern matching: match e with | p -> e
    Match(Box<Expr>, Vec<Arm>),
    
    /// Type annotation: e : T
    Annot(Box<Expr>, Type),
    
    /// Binary operator: a + b
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    
    /// Unary operator: -x
    UnOp(UnOp, Box<Expr>),
    
    /// Proc notation for signal functions: proc x -> do ...
    Proc(Vec<Parameter>, Vec<ProcStmt>),
    
    /// Macro invocation: name!(args)
    MacroInvoke {
        span: Span,
        name: String,
        args: Vec<TokenTree>,  // Unexpanded tokens
    },
    
    /// Quasi-quote: quote! { ... }
    Quote {
        span: Span,
        content: Vec<TokenTree>,
    },
    
    /// Unquote within quote: #expr or #(expr)
    Unquote {
        span: Span,
        expr: Box<Expr>,
        splice: bool,  // #(list) vs #single
    },
}

#[derive(Debug)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug)]
pub enum Pattern {
    Wildcard,           // _
    Var(String),        // x
    Literal(Literal),   // 42, "hello"
    Constructor(String, Vec<Pattern>),  // Just x
    Tuple(Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logical
    And, Or,
    // Other
    Compose,  // >>>
    Split,    // &&&
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

/// Statement in proc notation
#[derive(Debug)]
pub enum ProcStmt {
    Bind(String, Expr),        // y <- f -< x
    Let(LetBinding),           // let z = ...
    Rec(Vec<(String, Expr)>),  // rec
    Return(Expr),              // returnA -< x
}

/// Token tree for macro processing
#[derive(Debug)]
pub enum TokenTree {
    Token(Token),
    Group(Delimiter, Vec<TokenTree>),
}

#[derive(Debug, Clone, Copy)]
pub enum Delimiter {
    Paren, Brace, Bracket,
}
```

### 2.4 Types and Type Flags

```rust
#[derive(Debug, Clone)]
pub enum Type {
    /// Type variable: a, b
    Var(String),
    
    /// Named type: Int, Behavior Float
    Con(String, Vec<Type>),
    
    /// Function type: a -> b
    Arrow(Box<Type>, Box<Type>),
    
    /// Tuple type: (a, b)
    Tuple(Vec<Type>),
    
    /// Forall: forall a. a -> a
    Forall(Vec<String>, Box<Type>),
    
    /// Constrained type: Num a => a -> a
    Constraint(Vec<Constraint>, Box<Type>),
    
    /// Linear type: Lin τ (must use exactly once)
    Linear(Box<Type>),
    
    /// Thread-bound type: only valid in specific context
    Ephemeral(ExecutionContext, Box<Type>),
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub class: String,
    pub types: Vec<Type>,
}

/// Type modifiers for advanced type system features
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeFlags {
    /// Lin τ - must use exactly once
    pub is_linear: bool,
    /// Ephemeral τ - audio thread only
    pub is_ephemeral: bool,
    /// Can cross thread boundary
    pub is_send: bool,
    /// Can be shared between threads
    pub is_sync: bool,
}

impl Type {
    pub fn int() -> Self { Type::Con("Int".to_string(), vec![]) }
    pub fn float() -> Self { Type::Con("Float".to_string(), vec![]) }
    pub fn bool() -> Self { Type::Con("Bool".to_string(), vec![]) }
    pub fn string() -> Self { Type::Con("String".to_string(), vec![]) }
    pub fn unit() -> Self { Type::Con("()".to_string(), vec![]) }
    
    pub fn behavior(t: Type) -> Self { 
        Type::Con("Behavior".to_string(), vec![t]) 
    }
    pub fn event(t: Type) -> Self { 
        Type::Con("Event".to_string(), vec![t]) 
    }
    pub fn sf(a: Type, b: Type) -> Self { 
        Type::Con("SF".to_string(), vec![a, b]) 
    }
    
    /// Create linear version of type
    pub fn linear(self) -> Self {
        Type::Linear(Box::new(self))
    }
}
```

### 2.5 AST Properties

- **Immutable**: All AST nodes are immutable after construction
- **Arena allocation**: Uses arena allocator for efficient memory management
- **Hash-consing**: Types are hash-consed for fast equality comparison
- **Serialization**: AST can be serialized to JSON for debugging

---

## 3. Bytecode Specification

The Linen VM uses a stack-based bytecode with strict separation between Audio and Control contexts for real-time safety.

### 3.1 Execution Context

```rust
/// Execution context determines available opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionContext {
    /// Audio callback - hard real-time, no allocation, no blocking
    /// Allowed: OSC_*, ENV_*, FILTER_*, arithmetic, local variables
    Audio,
    
    /// Control thread - soft real-time, GC allowed, full VM
    /// Allowed: all opcodes except direct audio output
    Control,
}

/// Thread-safe value passing between contexts
pub enum ThreadMessage {
    /// Control → Audio: parameter change
    ParamChange { node_id: u32, value: f32 },
    /// Control → Audio: note event
    NoteOn { channel: u8, note: u8, vel: u8 },
    /// Control → Audio: note off
    NoteOff { channel: u8, note: u8 },
    /// Control → Audio: controller change
    CC { channel: u8, cc: u8, value: u8 },
    /// Audio → Control: metering (non-critical)
    Meter { channel: u8, level: f32 },
    /// Audio → Control: clip detection
    Clip { channel: u8 },
}
```

### 3.2 Instruction Format

```
┌─────────┬─────────────────────────────────────┐
│ Opcode  │  Operands (0-4 bytes each)          │
│ 1 byte  │  (variable length based on opcode)  │
└─────────┴─────────────────────────────────────┘
```

- Instructions are variable length
- Operands use LEB128 encoding for compactness
- All values on stack are 64-bit tagged pointers (NaN boxing)
- Opcode ranges enforce context safety:
  - `0x00-0x9F`: Universal (both contexts)
  - `0xA0-0xCF`: Control context only (FRP, closures)
  - `0xD0-0xDF`: Audio context only (oscillators, filters)
  - `0xE0-0xEF`: SIMD operations (both contexts)
  - `0xF0-0xFF`: Control flow & exceptions (context-dependent)

### 3.3 Value Representation

```rust
/// Tagged pointer using NaN boxing
pub type Value = u64;

const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const TAG_INT: u64   = 0x7FF8_0000_0000_0000;
const TAG_BOOL: u64  = 0x7FF9_0000_0000_0000;
const TAG_NIL: u64   = 0x7FFA_0000_0000_0000;
const TAG_OBJ: u64   = 0x7FFB_0000_0000_0000;
const TAG_LINEAR: u64 = 0x7FFC_0000_0000_0000;

/// Object header with type flags
pub struct ObjectHeader {
    pub flags: u32,      // TypeFlags
    pub size: u32,
    pub type_id: u64,
}

pub const FLAG_LINEAR: u32 = 0x01;        // Must move, not copy
pub const FLAG_AUDIO_THREAD: u32 = 0x02;  // Only valid in audio context

/// Object types (heap allocated)
pub enum Object {
    String(String),
    Array(Vec<Value>),
    Closure(Closure),
    Behavior(BehaviorState),
    Event(EventStream),
    AudioBuffer(AudioBuffer),
    Linear(LinearBox),  // Linear type wrapper
}
```

### 3.4 Linear Type Runtime Operations

```rust
/// Linear type runtime operations (debug builds verify, release trusts compiler)
#[derive(Debug, Clone, Copy)]
pub enum LinearOp {
    /// Transfer ownership (default for linear values)
    Move = 0x90,
    /// Explicit copy (requires Clone trait)
    Clone = 0x91,
    /// Explicit destroy (drop handler)
    Drop = 0x92,
    /// Temporary reference (borrow)
    Borrow = 0x93,
}

/// Linear type verification (debug builds only)
pub struct LinearChecker {
    pub owned_values: HashSet<Value>,
    pub borrowed_values: HashMap<Value, usize>, // refcount
}

impl LinearChecker {
    pub fn verify_move(&mut self, val: Value) -> Result<(), LinearViolation>;
    pub fn verify_clone(&mut self, val: Value) -> Result<(), LinearViolation>;
    pub fn verify_drop(&mut self, val: Value) -> Result<(), LinearViolation>;
    pub fn verify_borrow(&mut self, val: Value) -> Result<BorrowId, LinearViolation>;
}

#[derive(Debug, Error)]
pub enum LinearViolation {
    UseAfterMove { value: Value, location: Span },
    DoubleMove { value: Value, first: Span, second: Span },
    MissingDrop { value: Value, location: Span },
    BorrowAfterMove { value: Value, borrow: Span },
}
```

### 3.5 Opcodes by Category

#### Universal Opcodes (0x00-0x9F) - Both Contexts

| Opcode | Name | Operands | Description |
|--------|------|----------|-------------|
| 0x01 | PUSH_CONST | const_idx: u32 | Push constant from pool |
| 0x02 | PUSH_NIL | - | Push nil/() |
| 0x03 | PUSH_TRUE | - | Push true |
| 0x04 | PUSH_FALSE | - | Push false |
| 0x05 | POP | - | Pop and discard |
| 0x06 | DUP | - | Duplicate top of stack |
| 0x07 | SWAP | - | Swap top two elements |
| 0x08 | ROT | n: u8 | Rotate stack (move nth to top) |
| 0x09 | PICK | n: u8 | Copy nth from top |
| 0x10 | LOAD_LOCAL | idx: u16 | Push local variable |
| 0x11 | STORE_LOCAL | idx: u16 | Pop to local variable |
| 0x12 | LOAD_UPVALUE | idx: u16 | Load from closure |
| 0x13 | STORE_UPVALUE | idx: u16 | Store to closure |
| 0x14 | LOAD_GLOBAL | idx: u32 | Load global |
| 0x15 | STORE_GLOBAL | idx: u32 | Store global |
| 0x20 | IADD | - | Integer add |
| 0x21 | ISUB | - | Integer subtract |
| 0x22 | IMUL | - | Integer multiply |
| 0x23 | IDIV | - | Integer divide |
| 0x24 | INEG | - | Integer negate |
| 0x30 | FADD | - | Float add |
| 0x31 | FSUB | - | Float subtract |
| 0x32 | FMUL | - | Float multiply |
| 0x33 | FDIV | - | Float divide |
| 0x34 | FNEG | - | Float negate |
| 0x35 | FSIN | - | Sine (scalar) |
| 0x36 | FCOS | - | Cosine (scalar) |
| 0x37 | FEXP | - | Exponential |
| 0x38 | FLOG | - | Natural log |
| 0x39 | FPOW | - | Power |
| 0x40 | IEQ | - | Integer equal |
| 0x41 | ILT | - | Integer less than |
| 0x50 | FEQ | - | Float equal |
| 0x51 | FLT | - | Float less than |
| 0x60 | NOT | - | Boolean not |
| 0x61 | AND | - | Boolean and |
| 0x62 | OR | - | Boolean or |

#### Control Context Only (0xA0-0xCF)

**Note: These opcodes panic if called in Audio context**

| Opcode | Name | Operands | Description |
|--------|------|----------|-------------|
| 0xA0 | BEHAVIOR_PURE | - | Lift pure value to Behavior |
| 0xA1 | BEHAVIOR_MAP | - | Map over Behavior |
| 0xA2 | BEHAVIOR_LIFT2 | - | Lift2 for Behaviors |
| 0xA3 | BEHAVIOR_SAMPLE | - | Sample Behavior at Event |
| 0xA4 | BEHAVIOR_INTEGRAL | - | Time integral |
| 0xA5 | BEHAVIOR_DERIVATIVE | - | Time derivative |
| 0xB0 | EVENT_NEVER | - | Empty event |
| 0xB1 | EVENT_MERGE | - | Merge two events |
| 0xB2 | EVENT_MAP | - | Map over Event |
| 0xB3 | EVENT_FILTER | - | Filter events |
| 0xB4 | EVENT_HOLD | - | Hold initial value |
| 0xB5 | EVENT_ACCUM | - | Accumulate events |
| 0xC0 | SF_ARR | - | Arrow arr: pure function as SF |
| 0xC1 | SF_COMPOSE | - | Arrow composition (>>>) |
| 0xC2 | SF_SPLIT | - | Arrow split (&&&) |
| 0xC3 | SF_LOOP | - | Arrow loop with feedback |
| 0xC4 | CLOSURE | fn_idx: u32, upvalues: u8 | Create closure |
| 0xC5 | CALL | argc: u8 | Call function |
| 0xC6 | TAILCALL | argc: u8 | Tail call optimization |
| 0xC7 | RETURN | - | Return from function |
| 0xC8 | JUMP | offset: i32 | Unconditional jump |
| 0xC9 | JUMP_IF | offset: i32 | Jump if true |
| 0xCA | JUMP_IF_NOT | offset: i32 | Jump if false |

#### Audio Context Only (0xD0-0xDF)

**Note: These opcodes panic if called in Control context**

| Opcode | Name | Operands | Description |
|--------|------|----------|-------------|
| 0xD0 | OSC_SIN | freq: local_idx | Sine oscillator, updates phase |
| 0xD1 | OSC_SAW | freq: local_idx | Sawtooth oscillator |
| 0xD2 | OSC_SQUARE | freq: local_idx, pwm: local_idx | Square with PWM |
| 0xD3 | OSC_TRI | freq: local_idx | Triangle oscillator |
| 0xD4 | ENV_ADSR | gate: local_idx, params: const_idx | ADSR envelope |
| 0xD5 | FILTER_BIQUAD | type: u8, freq: local_idx, q: local_idx | Biquad filter |
| 0xD6 | AUDIO_OUT | channels: u8 | Output to audio device |
| 0xD7 | AUDIO_IN | channels: u8 | Input from audio device |
| 0xD8 | DELAY_READ | line: local_idx, time: local_idx | Read from delay line |
| 0xD9 | DELAY_WRITE | line: local_idx, value: local_idx | Write to delay line |

#### SIMD Operations (0xE0-0xEF) - Both Contexts

| Opcode | Name | Description |
|--------|------|-------------|
| 0xE0 | VEC_F32x4_ADD | 4-wide float add |
| 0xE1 | VEC_F32x4_MUL | 4-wide float multiply |
| 0xE2 | VEC_F32x4_FMA | Fused multiply-add |
| 0xE3 | VEC_F32x8_ADD | 8-wide float add (AVX) |
| 0xE4 | VEC_F32x8_MUL | 8-wide float multiply |
| 0xE5 | VEC_LOAD | Load from audio buffer |
| 0xE6 | VEC_STORE | Store to audio buffer |

#### Exceptions & Control Flow (0xF0-0xFF) - Control Only

| Opcode | Name | Operands | Description |
|--------|------|----------|-------------|
| 0xF0 | TRY | catch_offset: i32 | Begin exception scope |
| 0xF1 | CATCH | - | Exception handler entry |
| 0xF2 | FINALLY | - | Cleanup block |
| 0xF3 | THROW | - | Raise exception |
| 0xF4 | PANIC | - | Unrecoverable error |

#### Linear Type Operations (0x90-0x93)

| Opcode | Name | Description |
|--------|------|-------------|
| 0x90 | LINEAR_MOVE | Verify and transfer ownership |
| 0x91 | LINEAR_CLONE | Explicit clone (requires Clone) |
| 0x92 | LINEAR_DROP | Explicit destroy |
| 0x93 | LINEAR_BORROW | Create temporary reference |

### 3.6 Audio Error Handling

```rust
/// Audio thread: no exceptions, only soft failure strategies
pub enum AudioErrorStrategy {
    /// Clamp to valid range (default for saturation)
    Clip,
    /// Zero output (safe fallback)
    Silence,
    /// Skip processing, keep previous sample
    Continue,
    /// Log to control thread, continue
    ReportAndContinue,
}

/// Runtime audio error handling
pub struct AudioErrorHandler {
    pub strategy: AudioErrorStrategy,
    pub last_error: Option<AudioError>,
    pub report_channel: crossbeam::channel::Sender<ThreadMessage>,
}

#[derive(Debug, Clone)]
pub enum AudioError {
    InvalidFrequency { requested: f32, max: f32 },
    FilterUnstable { cutoff: f32, q: f32 },
    BufferUnderrun,
    ClipDetected { channel: u8, sample: f32 },
}
```

### 3.7 Function Structure

```
Function Header:
┌─────────────┬────────────────────────────────────┐
│ arity       │ u8                                 │
│ num_locals  │ u16                                │
│ num_upvalues│ u8                                 │
│ context     │ ExecutionContext (1 byte)          │
│ code_size   │ u32                                │
│ constants   │ [Constant]                         │
│ code        │ [u8]                               │
└─────────────┴────────────────────────────────────┘
```

### 3.8 Constant Pool

```rust
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Function(Function),
    BehaviorTemplate(BehaviorTemplate),
    Array(Vec<Value>),
}
```

---

## 4. Module Format (.lnc)

The compiled module format uses a hybrid binary + debug symbols approach with hot reload support.

### 4.1 File Layout

```
Linen Compiled Module (.lnc)
┌─────────────────────────────────────────────────────────┐
│ Header                                                  │
│   Magic Number          │ "LNC\0" (4 bytes)             │
│   Version               │ u32 (major << 16 | minor)     │
│   Checksum              │ u64 (CRC64 of content)        │
│   Section Table Offset  │ u32                           │
├─────────────────────────────────────────────────────────┤
│ Section Table                                           │
│   - Number of sections  │ u32                           │
│   - Section entries     │ [(id, offset, size), ...]     │
├─────────────────────────────────────────────────────────┤
│ SECTION: CODE            │ Binary bytecode                │
├─────────────────────────────────────────────────────────┤
│ SECTION: CONSTANTS       │ Constant pool                  │
├─────────────────────────────────────────────────────────┤
│ SECTION: EXPORTS         │ Export table                   │
├─────────────────────────────────────────────────────────┤
│ SECTION: IMPORTS         │ Import dependencies            │
├─────────────────────────────────────────────────────────┤
│ SECTION: TYPES           │ Type signatures                │
├─────────────────────────────────────────────────────────┤
│ SECTION: STATEFUL_NODES  │ Hot reload state descriptors   │
├─────────────────────────────────────────────────────────┤
│ SECTION: SOURCE_MAP      │ For error reporting            │
├─────────────────────────────────────────────────────────┤
│ SECTION: DEBUG (optional)│ Debug symbols                  │
│   - Line number table    │                                │
│   - Variable names       │                                │
│   - Source text          │                                │
├─────────────────────────────────────────────────────────┤
│ SECTION: METADATA        │ Package info                   │
│   - Package name         │                                │
│   - Version              │                                │
│   - Dependencies         │                                │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Section IDs

```rust
pub enum SectionId {
    Code = 0x01,
    Constants = 0x02,
    Exports = 0x03,
    Imports = 0x04,
    Types = 0x05,
    // Hot reload support
    StatefulNodes = 0x30,
    SourceMap = 0x31,
    // Debug
    Debug = 0x10,
    Metadata = 0x20,
}
```

### 4.3 Stateful Node Descriptors (Hot Reload)

```rust
/// Stateful node descriptor for hot reload
#[derive(Debug, Clone)]
pub struct StatefulNode {
    /// Stable identifier (hash of AST path or explicit #[stable(id)])
    pub node_id: u64,
    /// Node classification for state migration
    pub node_type: StatefulType,
    /// Size of serialized state in bytes
    pub state_size: u32,
    /// Version for migration compatibility
    pub version: u16,
    /// Execution context
    pub context: ExecutionContext,
    /// Parameter indices that affect this node
    pub param_deps: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatefulType {
    Oscillator,       // Phase, frequency, waveform
    ADSR,            // Current phase, level, state machine, params
    DelayLine,       // Buffer content, write position, length
    Biquad,          // Filter state (z1, z2), coefficients
    Comb,            // Comb filter state
    Allpass,         // Allpass filter state
    Ref,             // User-defined mutable state
    Custom(u16),     // User-defined with custom serializer
}

/// State snapshot for hot reload
pub struct StateSnapshot {
    /// Module version when snapshot was taken
    pub module_version: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Serialized states for each node
    pub states: HashMap<u64, Vec<u8>>,
}

/// State migration between versions
pub trait StateMigration {
    fn can_migrate(from: u16, to: u16) -> bool;
    fn migrate(state: &[u8], from: u16, to: u16) -> Result<Vec<u8>, MigrationError>;
}
```

### 4.4 Export Table

```rust
pub struct ExportEntry {
    /// Exported name
    pub name: String,
    /// Export kind
    pub kind: ExportKind,
    /// Offset into code/constants
    pub offset: u32,
    /// Type signature index
    pub type_idx: u32,
    /// Execution context for functions
    pub context: ExecutionContext,
}

pub enum ExportKind {
    Value,
    Type,
    Constructor,
}
```

### 4.5 Import Table

```rust
pub struct ImportEntry {
    /// Module path (e.g., ["std", "audio"])
    pub module: Vec<String>,
    /// Imported name
    pub name: String,
    /// Expected type signature
    pub type_signature: Type,
    /// Required version constraint
    pub version_req: VersionReq,
}

pub struct VersionReq {
    pub min: Version,
    pub max: Option<Version>,
    pub exact: bool,
}

pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
```

### 4.6 Module Metadata

```rust
pub struct ModuleMetadata {
    /// Package name
    pub name: String,
    /// Package version (SemVer)
    pub version: Version,
    /// Linen compiler version
    pub linen_version: Version,
    /// Author information
    pub author: Option<String>,
    /// Dependencies
    pub dependencies: Vec<Dependency>,
    /// All exports
    pub exports: Vec<ExportEntry>,
    /// Target platforms
    pub targets: Vec<Target>,
}

pub struct Dependency {
    pub name: String,
    pub version_req: VersionReq,
    pub registry: Registry,
    pub optional: bool,
}

pub enum Registry {
    Default,  // crates.io equivalent
    Local(String),  // path
    Git(String),    // git url
}

pub enum Target {
    Linux,
    MacOS,
    Windows,
    Wasm,
}
```

### 4.7 Debug Information

```rust
pub struct DebugInfo {
    /// Source file path
    pub source_file: String,
    /// Line number table: bytecode_offset -> source_line
    pub line_table: Vec<(u32, u32)>,
    /// Variable locations
    pub variables: Vec<VariableInfo>,
    /// Source text (optional, for REPL)
    pub source_text: Option<String>,
}

pub struct VariableInfo {
    pub name: String,
    pub scope_start: u32,
    pub scope_end: u32,
    pub location: VarLocation,
}

pub enum VarLocation {
    Local(u16),
    Upvalue(u16),
    Global(u32),
}
```

### 4.8 Versioning

- **Module format version**: Separate from language version
- **Backward compatibility**: Loader can read older versions
- **Forward compatibility**: Unknown sections are skipped
- **Magic number changes**: Breaking change, new major version
- **State migration**: Stateful nodes include version for migration

---

## 5. FFI ABI

**Note**: For Linen v1.0, FFI is restricted to built-in bindings. User-defined FFI is planned for post-v1.0.

### 5.1 Built-in Bindings

The following libraries are bound at the VM level:

- `cpal`: Audio I/O
- `libsndfile`: WAV file support
- System audio backends: JACK, CoreAudio

### 5.2 Type Marshaling

| Linen Type | C Type | Notes |
|------------|--------|-------|
| Int | int64_t | |
| Float | double | |
| Bool | uint8_t | 0 = false, 1 = true |
| String | const char* | UTF-8, null-terminated |
| () | void | |
| Behavior a | LinenBehavior* | Opaque handle |
| Event a | LinenEvent* | Opaque handle |
| Lin τ | LinenLinear* | Linear handle, single owner |

### 5.3 Thread Safety

- FFI calls run in the control thread (not audio thread)
- No GC during FFI calls
- Stack overflow protection
- Automatic resource cleanup via RAII guards

---

## Appendix A: Opcode Quick Reference

### Ranges by Context

| Range | Context | Description |
|-------|---------|-------------|
| 0x00-0x0F | Universal | Stack operations |
| 0x10-0x1F | Universal | Variables |
| 0x20-0x2F | Universal | Integer arithmetic |
| 0x30-0x3F | Universal | Float arithmetic |
| 0x40-0x4F | Universal | Integer comparison |
| 0x50-0x5F | Universal | Float comparison |
| 0x60-0x6F | Universal | Logic |
| 0x70-0x7F | Universal | Jumps |
| 0x80-0x8F | Universal | Calls |
| 0x90-0x93 | Universal | Linear type operations |
| 0xA0-0xCF | Control | FRP operations, closures |
| 0xD0-0xDF | Audio | Oscillators, filters, I/O |
| 0xE0-0xEF | Universal | SIMD operations |
| 0xF0-0xFF | Control | Exceptions |

### Execution Context Verification

```rust
/// Runtime context verification
pub fn verify_context(opcode: u8, context: ExecutionContext) -> Result<(), ContextViolation> {
    let allowed = match opcode {
        0x00..=0x8F | 0x90..=0x93 | 0xE0..=0xEF => true,  // Universal
        0xA0..=0xCF | 0xF0..=0xFF => context == ExecutionContext::Control,
        0xD0..=0xDF => context == ExecutionContext::Audio,
        _ => false,
    };
    
    if allowed {
        Ok(())
    } else {
        Err(ContextViolation {
            opcode,
            attempted_context: context,
            allowed_context: if opcode >= 0xD0 && opcode <= 0xDF {
                ExecutionContext::Audio
            } else {
                ExecutionContext::Control
            },
        })
    }
}
```

## Appendix B: Type Serialization

Types are serialized using a compact binary format:

```
Type encoding:
0x00: Unit
0x01: Int
0x02: Float
0x03: Bool
0x04: String
0x05: Linear(type)
0x10: Var(idx)
0x11: Con(name, [types...])
0x12: Arrow(arg, ret)
0x13: Tuple([types...])
0x14: Forall([vars...], type)
0x15: Constraint(class, [types...], inner)
0x16: Ephemeral(context, type)
```

## Appendix C: Real-time Safety Checklist

### Audio Thread Requirements

- [ ] No heap allocation (use region allocators)
- [ ] No blocking operations (no mutex, no I/O)
- [ ] No exceptions (use error codes or soft failure)
- [ ] Bounded execution time (no unbounded loops)
- [ ] Lock-free data structures only
- [ ] SIMD operations for DSP
- [ ] Cache-friendly memory layout

### Control Thread Guarantees

- [ ] GC can run (not in audio thread)
- [ ] Full VM available
- [ ] Exception handling
- [ ] File I/O
- [ ] Network operations
- [ ] User interface updates

### Thread Communication

- [ ] Lock-free ring buffers
- [ ] Bounded message queues
- [ ] No shared mutable state
- [ ] Copy semantics for cross-thread values
