# Linen — Coding Standards for Code Agent

## 1. Rust Code Style

### General Rules
- **Formatter**: `rustfmt` with default config
- **Linter**: `clippy` with `-- -D warnings`
- **Edition**: 2021
- **MSRV**: 1.70.0 (stable)

### Naming
```rust
// Types: PascalCase
type SignalNode<T> = ...;
struct SinOsc { ... }
enum OscillatorType { Sine, Saw, Square }

// Functions: snake_case
fn next_sample(&mut self, t: SampleTime) -> f32;
fn create_oscillator(freq: Hz) -> Box<dyn SignalNode<f32>>;

// Constants: SCREAMING_SNAKE_CASE
const SAMPLE_RATE: f32 = 44100.0;
const MAX_POLYPHONY: usize = 32;

// Modules: snake_case
mod oscillators;
mod signal_graph;
mod type_checker;

// Variables: snake_case
let sample_rate = 44100.0;
let voice_count = 8;
```

### Error Handling
```rust
// Use Result<T, E> for recoverable errors
fn parse(input: &str) -> Result<Ast, ParseError>;

// Use thiserror for error types
#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}

// Panic only for unrecoverable invariants
assert!(graph.is_acyclic(), "Signal graph must be a DAG");
```

### Documentation
```rust
/// Generates the next audio sample for this signal node.
/// 
/// # Arguments
/// * `t` - Current sample time (absolute, monotonically increasing)
/// * `sr` - Sample rate in Hz
/// 
/// # Returns
/// The sample value for time `t`
/// 
/// # Panics
/// Panics if `t` overflows internal phase counter (extremely unlikely)
pub fn next_sample(&mut self, t: SampleTime, sr: f32) -> f32 {
    ...
}
```

## 2. Module Organization

```
linen-core/
├── src/
│   ├── lib.rs              -- Re-exports
│   ├── ast.rs              -- AST definitions
│   ├── types.rs            -- Type system
│   ├── lexer.rs            -- Token definitions
│   ├── parser.rs           -- Parser implementation
│   ├── type_checker.rs     -- HM inference
│   ├── desugar.rs          -- Desugaring passes
│   ├── core_lang.rs        -- Core language AST
│   ├── interpreter.rs      -- Tree-walking interpreter (v0.1)
│   ├── value.rs            -- Runtime values
│   ├── signal_graph.rs     -- FRP graph lowering
│   ├── error.rs            -- Error types and diagnostics
│   └── span.rs             -- Source location tracking

linen-compiler/
├── src/
│   ├── lib.rs
│   ├── lowering.rs         -- FRP graph lowering
│   ├── codegen.rs          -- LLVM codegen (v0.2+)
│   ├── link.rs             -- Linking (v0.2+)
│   └── runner.rs           -- Binary execution (v0.2+)

linen-dsp/
├── src/
│   ├── lib.rs
│   ├── traits.rs           -- SignalNode trait
│   ├── oscillators.rs      -- SinOsc, SawOsc, etc.
│   ├── filters.rs          -- Biquad, etc.
│   ├── envelopes.rs        -- ADSR, etc.
│   ├── effects.rs          -- Delay, Reverb
│   ├── graph.rs            -- SignalGraph execution
│   ├── io.rs               -- WAV export, JACK
│   └── ffi.rs              -- C ABI

linen-repl/
├── src/
│   ├── lib.rs
│   ├── interpreter.rs      -- Tree-walking eval
│   ├── commands.rs         -- REPL commands
│   └── preview.rs          -- JACK preview

linc/
├── src/
│   └── main.rs             -- CLI entry point
```

## 3. Testing Standards

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_wave_produces_correct_frequency() {
        let mut osc = SinOsc::new(hz(440.0));
        let sr = 44100.0;

        let samples: Vec<f32> = (0..441)
            .map(|t| osc.next_sample(t, sr))
            .collect();

        let zero_crossings = samples.windows(2)
            .filter(|w| w[0] * w[1] < 0.0)
            .count();

        assert_eq!(zero_crossings, 2, "440Hz sine at 44.1kHz should have 2 zero crossings per period");
    }
}
```

### Integration Tests
```rust
// tests/integration_tests.rs
#[test]
fn compile_hello_sine() {
    let result = compile_file("examples/hello_sine.ln");
    assert!(result.is_ok());

    let output = std::fs::read("hello_sine.wav").unwrap();
    assert!(output.len() > 44); // WAV header is 44 bytes
}
```

## 4. Code Agent Instructions

### When Implementing...

1. **Always start with types**: Define data structures before algorithms
2. **Use pattern matching**: Prefer `match` over `if/else` for ADT
3. **Avoid unwrap/expect in library code**: Return `Result`
4. **Document public API**: Every `pub` item must have doc comment
5. **Write tests first**: TDD for DSP algorithms (golden tests for audio)
6. **Profile before optimize**: Use `cargo flamegraph` for hotspots
7. **Keep functions small**: < 50 lines, single responsibility
8. **Use iterators**: Instead of manual loops where possible
9. **Explicit types in complex expressions**: `let x: f32 = ...`
10. **No unsafe without discussion**: Only after review

### Code Review Checklist
- [ ] Compiles without warnings
- [ ] Clippy clean
- [ ] Tests pass
- [ ] Documentation updated
- [ ] CHANGELOG updated
- [ ] ADR created/updated if architectural change

## 5. Git Workflow

### Branch Naming
```
feature/lexer-implementation
bugfix/type-inference-polymorphism
docs/frp-model-update
refactor/signal-graph-optimization
```

### Commit Messages
```
feat(lexer): add string literal tokenization

- Support single and double quoted strings
- Escape sequences: \\, \", \n, \t
- Error on unterminated strings

Refs: #42
```

## 6. Performance Guidelines

### DSP Hot Path
```rust
// GOOD: No allocation in audio thread
impl SignalNode for SinOsc {
    #[inline]
    fn next_sample(&mut self, t: SampleTime, sr: f32) -> f32 {
        self.phase += self.freq * 2.0 * PI / sr;
        self.phase.sin()
    }
}

// BAD: Allocation in audio thread
impl SignalNode for BadOsc {
    fn next_sample(&mut self, t: SampleTime, sr: f32) -> f32 {
        let buffer = vec![0.0; 1024]; // allocates!
        ...
    }
}
```

## 7. FFI Boundaries

```rust
// C ABI for compiled code
#[no_mangle]
pub extern "C" fn linen_dsp_sine_next(osc: *mut SinOsc, t: u64, sr: f32) -> f32 {
    let osc = unsafe { &mut *osc };
    osc.next_sample(t, sr)
}
```

---

*Version: 0.1*  
*Date: 2026-05-15*
