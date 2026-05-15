# Linen — Prompts for Code Agent

## Prompt Template

```
You are implementing [COMPONENT] for Linen, a statically-typed functional reactive programming language for algorithmic music.

Context:
- Tech stack: Rust, interpreter (v0.1), LLVM via inkwell (v0.2+), DSP runtime
- Architecture: See docs/06_Architecture_C4-L3.md
- Coding standards: See docs/10_Coding_Standards.md
- Current phase: [PHASE]

Task: [SPECIFIC TASK]

Requirements:
1. [REQUIREMENT 1]
2. [REQUIREMENT 2]
3. [REQUIREMENT 3]

Constraints:
- No unsafe code without explicit justification
- All public functions must be documented
- Include unit tests
- Follow Rust naming conventions

Deliverables:
- [FILE 1]: [DESCRIPTION]
- [FILE 2]: [DESCRIPTION]
- Tests in [TEST FILE]

Reference implementation / prior art:
- [LINK OR DESCRIPTION]
```

---

## Phase 0: Foundation

### Prompt P0.1: Project Scaffolding
```
Create Cargo workspace for Linen with crates:
- linen-core: AST, types, errors, spans
- linen-lexer: tokenization (use logos crate)
- linen-parser: recursive descent parser
- linen-dsp: DSP runtime skeleton
- linc: CLI binary (use clap)
- linen-repl: REPL binary (use rustyline)

Each crate should have:
- lib.rs with module structure
- Cargo.toml with dependencies
- tests/ directory
- README stub

Use workspace-level Cargo.toml.
```

### Prompt P0.2: Lexer Implementation
```
Implement lexer for Linen using logos derive macro.

Tokens needed:
- Keywords: let, fn, type, match, with, import, export, as, if, then, else, true, false
- Literals: Int (i64), Float (f64), String (double-quoted)
- Operators: +, -, *, /, ==, !=, <, >, <=, >=, &&, ||, |> (pipeline), ->, =>, .
- Delimiters: (, ), [, ], {, }, ,, ;, =, :, |
- Identifiers: [a-zA-Z_][a-zA-Z0-9_]*
- Comments: -- line comment, {- block comment -}

Requirements:
- Proper error handling for invalid tokens
- Source span tracking (line, column)
- Skip whitespace and comments
- Test coverage for all token types

File: linen-lexer/src/lib.rs
Tests: linen-lexer/tests/lexer_tests.rs
```

### Prompt P0.3: Parser Implementation
```
Implement recursive descent parser for Linen.

Grammar (simplified):
module ::= decl*
decl ::= "let" ident [":" type] "=" expr
       | "fn" ident param+ [":" type] "->" expr
       | "type" ident [type-params] "=" variant*
expr ::= pipeline
pipeline ::= apply ("|>" apply)*
apply ::= primary+
primary ::= number | string | ident | "(" expr ")" | record | list | lambda | match | if
lambda ::= "\" param+ "->" expr
match ::= "match" expr "with" ("|" pattern "->" expr)+
pattern ::= ident | "_" | number | record-pattern | variant-pattern

Requirements:
- Left-associative binary operators with precedence
- Error recovery: report error, skip to next statement
- Source span for every AST node
- AST enum in linen-core/src/ast.rs

Files:
- linen-core/src/ast.rs (AST definitions)
- linen-parser/src/lib.rs (parser implementation)
- linen-parser/tests/parser_tests.rs
```

### Prompt P0.4: DSP Runtime Skeleton
```
Create DSP runtime skeleton in linen-dsp.

Core trait:
```rust
pub trait SignalNode {
    fn next_sample(&mut self, t: u64, sr: f32) -> f32;
}
```

Implement:
- SinOsc: sine wave oscillator
  - Fields: freq (can be constant or another SignalNode)
  - Phase accumulator
- Gain: amplitude scaling
- Mix: sum multiple signals

Requirements:
- All oscillators use f32 for samples
- Phase accumulator to avoid sin(t) per sample (use increment)
- Unit tests with golden reference (compare with known good implementation)

Files:
- linen-dsp/src/traits.rs
- linen-dsp/src/oscillators.rs
- linen-dsp/src/mix.rs
- linen-dsp/tests/oscillator_tests.rs
```

---

## Phase 1: Type System

### Prompt P1.1-P1.7: Typechecker
```
Implement Hindley-Milner type inference for Linen.

Types to support:
- Primitives: Int, Float, Bool, String, Unit
- Audio phantom types: Hz, Seconds, Samples, Amplitude (all Float underneath)
- FRP types: Signal<T>, Event<T>
- Composites: tuples (A, B), records {field: T}, variants Type = A | B
- Functions: A -> B
- Type variables: a, b, c (for polymorphism)

Algorithm:
1. Implement Type enum with TypeVar (unique IDs), TypeCon, Arrow, Tuple, Record
2. Unification: unify(Type, Type) -> Result<Subst, TypeError>
3. Algorithm W: infer(env, expr) -> Result<(Subst, Type), TypeError>
4. Let-polymorphism: generalize free variables, instantiate schemes
5. Error messages with source spans

Special cases:
- sin : Hz -> Signal Float
- gain : Float -> Signal Float -> Signal Float
- every : Seconds -> Event Unit
- map : (a -> b) -> Signal a -> Signal b

Files:
- linen-core/src/types.rs (Type definitions)
- linen-core/src/type_checker.rs (Algorithm W)
- linen-core/tests/typecheck_tests.rs
```

---

## Phase 2: Core Language

### Prompt P2.1-P2.4: Desugaring
```
Implement desugaring from surface AST to Core Language (ANF).

Core Language:
- Only let bindings, variables, literals, applications, lambdas
- No pattern matching (desugared to case chains)
- No pipeline operator (desugared to nested applications)
- No records (desugared to tuples with field indices)
- No type aliases (expanded)

ANF requirements:
- Every intermediate result is let-bound
- All function arguments are atoms (variables or literals)
- No nested applications

Example:
```linen
let x = sin 440.0 |> gain 0.5
```
-> Core:
```
let v0 = hz 440.0
let v1 = sin v0
let x = gain 0.5 v1
```

Files:
- linen-core/src/core_lang.rs (Core AST)
- linen-core/src/desugar.rs (Desugaring passes)
- linen-core/tests/desugar_tests.rs
```

---

## Phase 3: FRP Lowering

### Prompt P3.1-P3.4: Signal Graph
```
Implement FRP lowering from Core Language to Signal Graph (DAG).

SignalGraph representation:
```rust
struct SignalGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    outputs: Vec<NodeId>,
}

enum Node {
    Constant(f32),
    Oscillator(OscType, NodeId),  -- freq input
    Filter(FilterType, NodeId, NodeId),  -- input, params
    Effect(EffectType, Vec<NodeId>),
    EventSource(EventType),
    Behavior(BehaviorType, NodeId),  -- event input
}
```

Lowering rules:
- `sin expr` -> Oscillator(Sine, lower(expr))
- `gain amount signal` -> Filter(Gain, lower(signal), lower(amount))
- `lowpass freq q signal` -> Filter(Lowpass, lower(signal), lower(freq), lower(q))
- `event |> map f` -> EventMap(lower(event), lower(f))
- `event |> hold initial` -> Behavior(Hold, lower(initial), lower(event))

Validation:
- Detect cycles (error unless through delay node)
- Check all inputs connected
- Type consistency

Files:
- linen-core/src/signal_graph.rs
- linen-core/src/lowering.rs
- linen-core/tests/lowering_tests.rs
```

---

## Phase 4: Interpreter & Execution (v0.1)

### Prompt P4.1-P4.3: Tree-Walking Interpreter
```
Implement tree-walking interpreter for Linen Core AST.

Requirements:
1. Evaluate Core AST expressions to Values
2. Value enum:
   ```rust
   enum Value {
       Int(i64),
       Float(f64),
       Bool(bool),
       String(String),
       Unit,
       Signal(Box<dyn SignalNode>),
       Event(EventStream),
       Closure { env, params, body },
   }
   ```
3. Environment: HashMap<String, Value> with lexical scoping
4. Function application: evaluate args, bind to params, evaluate body
5. Signal graph execution: call `next_sample` on root node for each sample
6. Export: write samples to WAV via linen-dsp/io

Performance note:
- Only graph topology is interpreted; DSP primitives are native Rust
- Target: <30s bounce for 30s track with simple graph

Files:
- linen-core/src/interpreter.rs
- linen-core/src/value.rs
- linen-core/tests/interpreter_tests.rs
```

---

## Phase 5: DSP Runtime

### Prompt P5.1-P5.7: Full DSP Library
```
Implement complete DSP runtime for Linen.

Oscillators:
- SinOsc: phase accumulator, anti-aliased
- SawOsc: band-limited (BLEP or polyBLEP)
- SquareOsc: band-limited
- TriangleOsc: band-limited
- NoiseWhite: random [-1, 1]
- NoisePink: filtered white noise

Filters (Biquad):
- Lowpass, Highpass, Bandpass, Notch
- State variable filter (LP, HP, BP, BR simultaneously)

Envelopes:
- ADSR: attack, decay, sustain, release (exponential curves)
- AR: attack, release
- Perc: percussion envelope

Effects:
- Delay: circular buffer, feedback, mix
- Reverb: Schroeder algorithm (comb + allpass filters)
- Chorus: modulated delay lines
- Distortion: waveshaping
- Compressor: threshold, ratio, attack, release

Utilities:
- Gain: multiply by constant
- Pan: stereo balance
- Mix: sum with gain compensation
- Split: duplicate signal

WAV Export (symphonia):
- 16/24/32-bit integer
- 32/64-bit float
- 44.1/48/96/192 kHz
- Mono/Stereo

JACK Preview:
- Register as JACK client
- Process callback: fill buffer from signal graph
- Soft realtime (no hard guarantees)

Files:
- linen-dsp/src/oscillators.rs
- linen-dsp/src/filters.rs
- linen-dsp/src/envelopes.rs
- linen-dsp/src/effects.rs
- linen-dsp/src/graph.rs
- linen-dsp/src/io.rs
- linen-dsp/tests/*
```

---

## Phase 6: REPL

### Prompt P6.1-P6.6: Interactive Environment
```
Implement Linen REPL.

Features:
- Read input with history (rustyline)
- Parse and typecheck expressions
- Evaluate via tree-walking interpreter
- Play 5-second preview via JACK
- Commands:
  - :type <expr> -- show inferred type
  - :export <expr> <file> <dur> -- export to WAV
  - :load <file> -- load module
  - :info <ident> -- show documentation
  - :quit -- exit

Interpreter:
- Tree-walking over Core AST
- Environment: HashMap<String, Value>
- Value enum: Int, Float, Bool, String, Signal(Box<dyn SignalNode>), Event(...)
- Signal evaluation: sample-by-sample for preview

Preview:
- Create JACK client
- 5-second ring buffer
- Fill buffer from interpreter-evaluated signal
- Play once, then stop

Files:
- linen-repl/src/lib.rs
- linen-repl/src/interpreter.rs
- linen-repl/src/commands.rs
- linen-repl/src/preview.rs
- linen-repl/tests/repl_tests.rs
```

---

## Phase 7: Stdlib & Examples

### Prompt P7.1-P7.5: Standard Library
```
Implement standard library for Linen.

Module structure:
```
stdlib/
├── oscillators.ln    -- sin, saw, square, triangle, noise
├── filters.ln        -- lowpass, highpass, bandpass, notch
├── envelopes.ln      -- adsr, ar, perc
├── effects.ln        -- delay, reverb, chorus, distortion
├── events.ln         -- every, random, cycle, sequence
├── temporal.ln       -- seconds, hz, bpm, samples
└── utils.ln          -- gain, pan, mix, split, clip
```

Each module exports typed functions with documentation.

Examples to create:
- hello_sine.ln
- drone.ln
- melody.ln
- polysynth.ln
- rhythm.ln
- fm.ln
- granular.ln
- karplus_strong.ln

Documentation generator:
- Extract type signatures from .ln files
- Generate markdown docs
- Include doc comments

Files:
- stdlib/*.ln
- examples/*.ln
- linen-doc/src/lib.rs (doc generator)
```

---

## Future: Phase 8 — LLVM Backend (v0.2+)

### Prompt P8.1-P8.4: LLVM Codegen
```
Implement LLVM code generation using inkwell crate.

Requirements:
1. Setup inkwell Context, Module, Builder
2. Generate LLVM IR for:
   - Float constants
   - Float arithmetic (+, -, *, /)
   - Function calls (to DSP runtime FFI)
   - Structs for oscillator state
   - Main loop: for t in 0..samples { write next_sample(t) }
3. Link with liblinendsp.a (static library)
4. Compile to ELF binary using LLVM JIT or llc + ld

DSP FFI functions:
```c
void* linen_dsp_sinosc_new(float freq);
float linen_dsp_sinosc_next(void* osc, uint64_t t, float sr);
void linen_dsp_sinosc_free(void* osc);
```

Generated code structure:
```llvm
; Simplified
%SinOsc = type { float, float }  ; freq, phase

define float @drone_next(%SinOsc* %osc, i64 %t, float %sr) {
  ; load freq, update phase, compute sin, apply gain
  ret float %result
}

define i32 @main() {
  ; allocate osc
  ; loop 441000 times (10s @ 44.1kHz)
  ; call @drone_next
  ; write to WAV
  ret i32 0
}
```

Files:
- linen-compiler/src/codegen.rs
- linen-compiler/src/link.rs
- linen-compiler/tests/codegen_tests.rs
```

---

## General Tips for Code Agent

1. **Start small**: Implement minimal viable version, then iterate
2. **Test-driven**: Write test cases before implementation
3. **Reference implementations**: Look at rustc, GHC, Faust compiler for patterns
4. **Ask for clarification**: If spec is ambiguous, ask before implementing
5. **Document decisions**: Add comments for non-obvious choices
6. **Benchmark**: Include simple benchmarks for DSP operations
7. **Error messages**: Prioritize good error messages over features

---

*Version: 0.1*  
*Date: 2026-05-15*
