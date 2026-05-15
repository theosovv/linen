# Linen — Development Roadmap

## Philosophy
Each phase delivers a working product. No "big bang" releases. Ship the interpreter first, compile later.

---

## Phase 0: Foundation (Week 1)
**Goal:** Working lexer + parser + basic AST for simple expressions

### Tasks
- [ ] **P0.1** Project scaffolding: Cargo workspace, crates structure
  - `linen-core`: AST, types, errors
  - `linen-lexer`: tokenization
  - `linen-parser`: recursive descent parser
  - `linen-dsp`: runtime library skeleton
  - `linc`: CLI binary
  - `linen-repl`: REPL binary

- [ ] **P0.2** Lexer: tokens for literals, identifiers, operators, keywords
  - Keywords: `let`, `fn`, `type`, `match`, `with`, `import`, `export`, `as`, `if`, `then`, `else`
  - Operators: `|>`, `->`, `=>`, `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`
  - Literals: integers, floats, strings, booleans

- [ ] **P0.3** Parser: expressions, let bindings, function definitions
  - EBNF grammar implementation
  - Error recovery (skip to next statement)
  - AST enum definition

- [ ] **P0.4** DSP Runtime skeleton: basic oscillator trait
  ```rust
  trait SignalNode {
      fn next_sample(&mut self, t: u64, sr: f32) -> f32;
  }
  ```

### Deliverables
- `echo "let x = 5" | linc parse` -> AST output
- `cargo test` passes for lexer/parser

---

## Phase 1: Type System (Week 2)
**Goal:** HM type inference for basic types + audio primitives

### Tasks
- [ ] **P1.1** Type representation: Type enum, TypeVar, TypeCon
- [ ] **P1.2** Unification algorithm
- [ ] **P1.3** Algorithm W: inference for literals, variables, application, abstraction
- [ ] **P1.4** Let-polymorphism: generalization and instantiation
- [ ] **P1.5** Audio types: Signal<T>, Event<T>, Hz, Seconds, Amplitude (phantom types)
- [ ] **P1.6** Type aliases: `type Filter = Signal Float -> Signal Float`
- [ ] **P1.7** Error messages: pretty printing with source locations

### Deliverables
- `linc check file.ln` — typechecks and reports errors
- All test cases from `tests/typecheck/` pass

---

## Phase 2: Core Language & Desugaring (Week 3)
**Goal:** ANF (A-Normal Form) for simplifying execution

### Tasks
- [ ] **P2.1** Core AST: simpler than surface AST, explicit bindings
- [ ] **P2.2** Desugarer: pattern matching -> case chains, pipeline -> nested apply
- [ ] **P2.3** Alpha renaming (unique variable names)
- [ ] **P2.4** ANF conversion: all intermediates are let-bound

### Deliverables
- `linc desugar file.ln` -> Core AST output
- Property tests: desugar(interpret) == interpret(surface)

---

## Phase 3: Interpreter & FRP Lowering (Week 4)
**Goal:** Execute Linen programs via tree-walking interpreter

### Tasks
- [ ] **P3.1** SignalGraph representation: nodes, edges, types
- [ ] **P3.2** Lower `Signal<T>` expressions to graph nodes
- [ ] **P3.3** Lower `Event<T>` expressions to event callbacks
- [ ] **P3.4** Detect cycles (error) vs valid feedback through `delay`
- [ ] **P3.5** Tree-walking interpreter for Core AST
- [ ] **P3.6** Signal evaluation: sample-by-sample via `SignalNode.next_sample`

### Deliverables
- `linc run hello_sine.ln` -> `hello_sine.wav`
- Interpreter unit tests for arithmetic, functions, signals

---

## Phase 4: DSP Runtime & WAV Export (Week 5, parallel with P3)
**Goal:** Full audio library with offline bounce

### Tasks
- [ ] **P4.1** Oscillators: sin, saw, square, triangle, noise (white, pink)
- [ ] **P4.2** Filters: biquad (LP, HP, BP, notch), state variable
- [ ] **P4.3** Envelopes: ADSR, AR, perc (exponential, linear)
- [ ] **P4.4** Effects: delay (circular buffer), reverb (Schroeder), chorus
- [ ] **P4.5** Utilities: gain, pan, mix, split, resample
- [ ] **P4.6** WAV export: symphonia integration, 16/24/32-bit, 44.1/48/96kHz
- [ ] **P4.7** JACK preview: soft realtime playback for REPL

### Deliverables
- `tests/dsp/` — unit tests for each module
- `examples/sine_wave.ln` -> `sine_wave.wav` (verified)

---

## Phase 5: REPL (Week 5-6)
**Goal:** Interactive environment for prototyping

### Tasks
- [ ] **P5.1** rustyline integration: history, completion
- [ ] **P5.2** REPL commands: `:type`, `:export`, `:load`, `:info`, `:quit`
- [ ] **P5.3** Multi-line input for function definitions
- [ ] **P5.4** JACK playback: 5-second preview buffer

### Deliverables
- `linen-repl` binary
- `> let x = sin 440.0` -> evaluates, plays preview

---

## Phase 6: Stdlib & Examples (Week 6)
**Goal:** Patch library and documentation

### Tasks
- [ ] **P6.1** Module system: import/export, module resolution
- [ ] **P6.2** Stdlib modules: `std/oscillators`, `std/filters`, `std/envelopes`, `std/events`, `std/temporal`
- [ ] **P6.3** Examples: `hello_sine.ln`, `drone.ln`, `rhythm.ln`, `polysynth.ln`
- [ ] **P6.4** Documentation generator from type signatures
- [ ] **P6.5** README, tutorial, API reference

### Deliverables
- `examples/` directory with 10+ working examples
- Generated docs in `docs/api/`

---

## Phase 7: Polish & Release v0.1 (Week 7-8)
**Goal:** Stable MVP for public use

### Tasks
- [ ] **P7.1** Error message improvements: suggestions, colored output
- [ ] **P7.2** Performance benchmarks: bounce time for standard patches
- [ ] **P7.3** Memory profiling: no leaks in long bounces
- [ ] **P7.4** CI/CD: GitHub Actions, automated tests
- [ ] **P7.5** Release binaries: cargo publish, GitHub releases
- [ ] **P7.6** Announcement post, community setup (Discord/Reddit)

### Deliverables
- GitHub release v0.1.0
- Published crates on crates.io

---

## Future Roadmap (v0.2+)

### v0.2: LLVM Backend & Advanced FRP
- [ ] LLVM codegen via inkwell: compile Signal Graph to native code
- [ ] Hot-swap via LLVM ORC in REPL
- [ ] Feedback loops: `feedback : (Signal a -> Signal a) -> Signal a`
- [ ] Pattern DSL: TidalCycles-like mini-notation
- [ ] MIDI input/output
- [ ] VST3/LV2 plugin export

### v0.3: Type System Enhancements
- [ ] Typeclasses (Num, Ord, Signalizable)
- [ ] Linear types for audio buffers
- [ ] Dependent types: `Vector<n, Float>`, `Signal<SR=44100>`
- [ ] Refinement types: `Amplitude = {x: Float | 0 <= x <= 1}`

### v0.4: Ecosystem
- [ ] Package manager: `linen-pkg`
- [ ] LSP server: autocomplete, goto-definition, hover types
- [ ] VSCode extension
- [ ] Web playground (WASM compilation)

### v0.5: Performance
- [ ] SIMD vectorization in codegen
- [ ] GPU offload for FFT/convolution
- [ ] Multicore: parallel voice processing
- [ ] JIT for REPL (no interpreter)

---

## Timeline Summary

| Phase | Weeks | Key Deliverable |
|-------|-------|-----------------|
| 0: Foundation | 1 | Lexer + Parser |
| 1: Type System | 2 | Typechecker |
| 2: Core Lang | 3 | ANF desugaring |
| 3: Interpreter + FRP | 4 | Working execution |
| 4: DSP Runtime | 5 | Full audio library |
| 5: REPL | 5-6 | Interactive environment |
| 6: Stdlib | 6 | Examples + docs |
| 7: Release | 7-8 | v0.1.0 |

**Total MVP: 8 weeks (2 months) solo dev**

---

## Task Dependency Graph

```
P0 (Lexer/Parser)
  |
  +--> P1 (Typechecker)
  |      |
  |      +--> P2 (Desugaring)
  |      |      |
  |      |      +--> P3 (Interpreter + FRP Lowering)
  |      |      |      |
  |      |      |      +--> P5 (REPL)
  |      |      |
  |      |      +--> P4 (DSP Runtime) ----------> P3, P5
  |      |
  |      +--> P6 (Stdlib) -----------------------> P7 (Release)
```

---

## Resource Allocation

| Role | Time | Notes |
|------|------|-------|
| Architect (you) | 20% | ADRs, reviews, critical decisions |
| Code Agent (AI) | 50% | Implementation per specifications |
| You (coding) | 30% | DSP algorithms, debugging, integration |

---

*Version: 0.1*  
*Date: 2026-05-15*
