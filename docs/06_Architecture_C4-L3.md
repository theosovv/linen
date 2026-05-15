# Linen — Software Architecture (C4 Level 3)

## 1. System Context (C4 Level 1)

```
+-------------------------------------------------------------+
|                    Linen Language Suite                      |
+-------------------------------------------------------------+
|                                                             |
|  +--------------+    +--------------+    +--------------+     |
|  |   Composer   |    |    REPL      |    |   Build Tool |  |
|  |  (writes .ln)|   |  (interactive)|   |  (linc CLI)  |  |
|  +------+-------+    +------+-------+    +------+-------+  |
|         |                   |                   |           |
|         +-------------------+-------------------+           |
|                             |                               |
|                    +----------v----------+                   |
|                    |  Linen              |                   |
|                    |  Runtime            |                   |
|                    |  (Rust binary)      |                   |
|                    +----------+----------+                   |
|                             |                               |
|                    +----------v----------+                   |
|                    |  WAV/FLAC File      |                   |
|                    +---------------------+                   |
|                                                             |
+-------------------------------------------------------------+
```

## 2. Container Diagram (C4 Level 2)

```
+---------------------------------------------------------------------+
|                         Linen Language Suite                         |
+---------------------------------------------------------------------+
|                                                                      |
|  +--------------------------------------------------------------+  |
|  |                     Linen CLI (Rust)                          |  |
|  |  +--------------+  +--------------+  +--------------+     |  |
|  |  |   linc       |  |   linen-repl   |  |   linen-run    |    |  |
|  |  |  (compiler/  |  |   (REPL)       |  |   (runner)     |    |  |
|  |  |   checker)   |  |                |  |                |    |  |
|  |  +------+---------+  +------+---------+  +------+---------+   |  |
|  |         |                   |                   |               |  |
|  |         +-------------------+-------------------+               |  |
|  |                             |                                   |  |
|  |              +--------------v--------------+                    |  |
|  |              |    Linen Core Library       |                    |  |
|  |              |         (Rust lib)          |                    |  |
|  |              +--------------+--------------+                    |  |
|  +-----------------------------+-----------------------------------+  |
|                                |                                      |
|  +-----------------------------v-----------------------------------+  |
|  |              DSP Runtime (Rust static lib)                     |  |
|  |  +----------+ +----------+ +----------+ +----------+          |  |
|  |  |Oscillators| |Filters  | |Envelopes | |Effects   |          |  |
|  |  +----------+ +----------+ +----------+ +----------+          |  |
|  +----------------------------------------------------------------+  |
|                                                                      |
|  External Dependencies:                                              |
|  +----------+ +----------+ +----------+ +----------+                 |
|  | logos    | | symphonia | | rustfft  | | clap     |                 |
|  | (lexer)  | | (audio IO)| | (FFT)    | | (CLI)    |                 |
|  +----------+ +----------+ +----------+ +----------+                 |
|                                                                      |
+---------------------------------------------------------------------+
```

## 3. Component Diagram — Interpreter Pipeline (C4 Level 3)

```
+---------------------------------------------------------------------+
|                    Linen Interpreter Core                            |
+---------------------------------------------------------------------+
|                                                                      |
|  Input: .ln file                                                     |
|       |                                                              |
|       v                                                              |
|  +--------------+     Tokens                                         |
|  |   Lexer      |----------------+                                   |
|  |  (logos)     |                |                                   |
|  +--------------+                v                                   |
|  +--------------+     AST       +--------------+                     |
|  |   Parser     |--------------->|   AST        |                     |
|  |  (hand-      |               |  (enum tree) |                     |
|  |   written)   |               +--------------+                     |
|  +--------------+                |                                   |
|                                  v                                   |
|  +--------------+     Typed AST +--------------+                     |
|  | Typechecker  |<-------------|   AST        |                     |
|  |  (HM W)      |   annotations|  (with types)|                     |
|  +--------------+              +--------------+                     |
|       |                                                              |
|       v                                                              |
|  +--------------+     Core AST   +--------------+                   |
|  |  Desugarer   |--------------->|   Core Lang  |                   |
|  |              |                |  (ANF form)  |                   |
|  +--------------+                +--------------+                   |
|       |                                                              |
|       v                                                              |
|  +--------------+     Signal Graph +--------------+                |
|  | FRP Lowerer  |----------------->|   SSA Graph  |                |
|  |              |                  |  (DAG nodes) |                |
|  +--------------+                  +--------------+                |
|       |                                                              |
|       v                                                              |
|  +--------------+     Values      +--------------+                |
|  | Interpreter  |---------------->|   Signal/    |                |
|  |  (tree-walk) |                 |   Event repr |                |
|  +--------------+                 +--------------+                |
|       |                                                              |
|       v                                                              |
|  +--------------+     WAV/FLAC                                    |
|  |   Bouncer    |---------------->  output.wav                    |
|  |  (DSP Runtime)|                                                  |
|  +--------------+                                                  |
|                                                                      |
+---------------------------------------------------------------------+
```

## 4. Component Diagram — REPL (C4 Level 3)

```
+---------------------------------------------------------------------+
|                    Linen REPL                                         |
+---------------------------------------------------------------------+
|                                                                      |
|  +--------------+                                                   |
|  |   Input      |  ":type sin 440.0"                                |
|  |   (rustyline)|----------+                                        |
|  +--------------+          |                                        |
|                            v                                        |
|  +--------------+     +--------------+                              |
|  |   Parser     |<----|   Command    |  -- dispatch :type, :export   |
|  |              |     |   Router     |                              |
|  +------+-------+     +--------------+                              |
|         |                                                           |
|         v                                                           |
|  +--------------+     +--------------+     +--------------+        |
|  |  Typechecker |---->|  Interpreter |---->|  Value       |        |
|  |  (same as    |      |  (tree-walk) |      |  (Signal/    |        |
|  |   pipeline)  |      |              |      |   Event repr)|        |
|  +--------------+      +------+-------+      +--------------+        |
|                              |                                      |
|                              v                                      |
|  +--------------+     +--------------+                              |
|  |   Preview    |<----|  DSP Runtime |  -- 5-second preview        |
|  |   (JACK/ALSA)|     |  (soft RT)   |     via audio backend       |
|  +--------------+     +--------------+                              |
|                                                                      |
|  State:                                                            |
|  +--------------+     +--------------+                              |
|  |   Env (vars) |     |   Type Env   |                              |
|  |   HashMap    |     |   HashMap    |                              |
|  +--------------+     +--------------+                              |
|                                                                      |
+---------------------------------------------------------------------+
```

## 5. Key Design Decisions

### 5.1 Interpreter-First Architecture
- **v0.1**: Tree-walking interpreter over Core AST for both REPL and `linc run`
- **v0.2+**: LLVM codegen will reuse the same pipeline stages (parser, typechecker, lowering)
- **Shared components**: AST, Typechecker, Parser, Lowerer — single crate `linen-core`
- **Benefit**: Working product in weeks, not months

### 5.2 DSP Runtime Architecture
```rust
// Runtime crate structure
linen-dsp/
├── src/
│   ├── lib.rs           -- public API
│   ├── oscillators.rs   -- SinOsc, SawOsc, etc.
│   ├── filters.rs       -- Biquad, StateVariable
│   ├── envelopes.rs     -- ADSR, AR
│   ├── effects.rs       -- Delay, Reverb
│   ├── graph.rs         -- SignalGraph execution engine
│   ├── io.rs            -- WAV export, JACK preview
│   └── ffi.rs           -- C ABI for future compiled code
```

### 5.3 Module System
- File = module (like in Rust)
- `import "path/to/module" as Alias`
- Module resolution: relative to entry file, then stdlib path
- Stdlib: `std/oscillators`, `std/filters`, `std/events`, `std/temporal`

### 5.4 Error Handling Strategy
- **Compile-time errors**: Type errors, parse errors, name resolution — via Result<T, Diagnostic>
- **Runtime errors**: Division by zero, out-of-bounds — via panic (since offline bounce, can restart)
- **Audio errors**: Clip detection (warning, not error), NaN detection (panic)

## 6. Data Flow — Example Interpretation

```
Input (.ln):
---------------
let drone = sin 220.0 |> gain 0.5
export drone "out.wav" 10.0

Phase 1: Lexing
---------------
[Let, Ident("drone"), Equals, Ident("sin"), Float(220.0), 
 Pipe, Ident("gain"), Float(0.5),
 Export, Ident("drone"), String("out.wav"), Float(10.0)]

Phase 2: Parsing
----------------
AST:
Let("drone", None, 
  Pipe(Apply(Ident("sin"), Float(220.0)), 
       Apply(Ident("gain"), Float(0.5))))
Export(Ident("drone"), String("out.wav"), Float(10.0))

Phase 3: Type Checking
----------------------
Env: sin : Hz -> Signal Float
     gain : Float -> Signal Float -> Signal Float
     drone : Signal Float  (inferred)

Phase 4: Desugaring to Core
---------------------------
let drone = gain(0.5, sin(hz(220.0)))
export(drone, "out.wav", seconds(10.0))

Phase 5: FRP Lowering
---------------------
SignalGraph:
  Node0: Const(220.0) -> Hz
  Node1: SinOsc(Node0) -> Signal Float
  Node2: Const(0.5) -> Float
  Node3: Gain(Node2, Node1) -> Signal Float
  Node4: Export(Node3, "out.wav", 10.0s) -> IO ()

Phase 6: Interpretation
-----------------------
- Allocate SinOsc state (phase accumulator)
- Allocate Gain node
- For t in 0..441000 (10s @ 44.1kHz):
    sample = gain.next_sample(sinosc.next_sample(t))
    write to WAV buffer
- Flush buffer to out.wav via symphonia
```

## 7. Technology Stack

| Component | Technology | Crate/Tool |
|-----------|-----------|------------|
| Language | Rust | — |
| Lexer | logos | `logos` |
| Parser | hand-written recursive descent | custom |
| Typechecker | HM Algorithm W | custom |
| Interpreter | tree-walking over Core AST | custom |
| LLVM Bindings | inkwell | `inkwell` (v0.2+) |
| Audio I/O | symphonia + JACK/ALSA | `symphonia`, `jack` |
| FFT | rustfft | `rustfft` |
| CLI | clap | `clap` |
| REPL | rustyline | `rustyline` |
| Math | libm | `libm` |

## 8. Interfaces

### 8.1 CLI
```bash
linc check input.ln                           # Typecheck only
linc run input.ln                             # Interpret and bounce to WAV
linc run input.ln -o out.wav -d 10.0          # Direct export
linc repl                                     # Start REPL
```

### 8.2 REPL Commands
```
:type <expr>           -- Show type of expression
:export <sig> <file> <dur>  -- Export signal to WAV
:load <file>           -- Load module into REPL
:info <ident>          -- Show documentation
:quit                  -- Exit REPL
```

## 9. Quality Attributes

| Attribute | Approach |
|-----------|----------|
| Performance | DSP primitives in native Rust; interpreter overhead only on graph topology |
| Correctness | Static typing, exhaustive pattern match checking, phantom types |
| Maintainability | Modular pipeline, shared core lib, ADR docs |
| Usability | REPL, good error messages, stdlib documentation |
| Portability | Linux first, WAV export platform-independent |
| Evolvability | Interpreter pipeline reusable for LLVM codegen in v0.2 |

---

*Version: 0.1*  
*Date: 2026-05-15*
