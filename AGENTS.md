# Linen Project Guide for AI Agents

This document provides essential information for AI coding agents working on the Linen project.

## Project Overview

**Linen** is a functional-reactive programming language designed for music and audio applications. It combines:
- **Functional programming**: Immutable data, first-class functions, Hindley-Milner type system
- **Reactive programming**: Time-varying values (Behaviors) and discrete events
- **Real-time audio**: Low-latency processing with hot reload without state loss

The project is implemented in Rust as a Cargo workspace. The language targets live coding scenarios where musicians modify code while audio is playing, preserving oscillator phases, envelope positions, and delay line contents across reloads.

## Technology Stack

- **Language**: Rust >= 1.75
- **Build System**: Cargo workspace
- **Parser**: chumsky (v1.0.0-alpha.7) - parser combinator library
- **Testing**: proptest (property-based), insta (snapshot), criterion (benchmarks)
- **Audio**: CPAL (cross-platform), JACK (Linux low-latency)
- **Concurrency**: crossbeam (lock-free data structures)

## Project Structure

```
linen/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── compiler/           # linen-compiler - Frontend
│   │   ├── src/
│   │   │   ├── lib.rs      # Main entry
│   │   │   ├── lexer.rs    # Tokenization
│   │   │   ├── parser.rs   # CST → AST
│   │   │   ├── typeck.rs   # Type checker (HM + Linear)
│   │   │   ├── ir.rs       # Intermediate representation
│   │   │   ├── codegen.rs  # Bytecode generation
│   │   │   └── error.rs    # Error types
│   │   └── Cargo.toml
│   ├── vm/                 # linen-vm - Runtime
│   │   ├── src/
│   │   │   ├── lib.rs      # Main entry
│   │   │   ├── vm.rs       # Stack-based VM
│   │   │   ├── bytecode.rs # Bytecode format
│   │   │   ├── audio.rs    # Audio thread (real-time)
│   │   │   ├── memory.rs   # Region-based allocation
│   │   │   └── jit.rs      # JIT compilation hooks
│   │   └── Cargo.toml
│   ├── stdlib/             # linen-stdlib - Standard library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── primitives.rs # Oscillators, filters
│   │   │   ├── frp.rs        # FRP combinators
│   │   │   ├── effects.rs    # Reverb, delay, compression
│   │   │   ├── time.rs       # BPM, transport
│   │   │   └── io.rs         # MIDI, file I/O
│   │   └── Cargo.toml
│   └── cli/                # linen-cli - Command-line interface
│       ├── src/
│       │   └── main.rs     # CLI entry point
│       └── Cargo.toml
├── docs/
│   ├── architecture.md     # Implementation details, bytecode spec
│   ├── language-spec.md    # Full language specification
│   ├── syntax-summary.md   # Quick reference
│   ├── roadmap.md          # Development roadmap (in Russian)
│   └── cross-reference.md  # Index
├── .kimi/skills/frp-linen/SKILL.md  # FRP patterns documentation
├── Cargo.lock
├── rustfmt.toml            # Formatting config
├── .pre-commit-config.yaml # Git hooks
└── .github/workflows/ci.yml # CI configuration
```

## Build Commands

```bash
# Build all crates
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check --all-features
```

## Test Commands

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p linen-compiler
cargo test -p linen-vm

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --workspace
```

## Code Quality Commands

```bash
# Format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
cargo doc --no-deps --all-features --workspace

# Run pre-commit hooks manually
pre-commit run --all-files
```

## Code Style Guidelines

- **Formatting**: Uses `rustfmt.toml` configuration
  - Edition: 2024
  - Max width: 100 characters
  - Tab spaces: 4 (spaces, not tabs)
  - Newline style: Unix
- **Documentation**: All public items must have doc comments
- **Naming**: Follow standard Rust conventions (snake_case for functions/variables, PascalCase for types)
- **Error handling**: Use `thiserror` for error types, `anyhow` for application errors
- **Traits**: Prefer explicit imports over glob imports

## Key Language Features

### FRP (Functional Reactive Programming)

- **Behavior a**: Continuous time-varying values
- **Event a**: Discrete occurrences
- **SF a b**: Signal functions (stateful transformations)

Example:
```linen
-- Sine wave with vibrato
let vibrato = sin (2.0 * pi * 5.0 * time) in
let freq = 440.0 + 5.0 * vibrato in
sinOsc freq
```

### Proc Notation

Arrow notation for signal functions:
```linen
synth = proc midi -> do
    freq  <- arr midiToFreq -< midi
    osc   <- sinOsc         -< freq
    env   <- adsr 0.1 0.3 0.7 1.0 -< gate midi
    returnA -< osc * env
```

### Linear Types

Resource safety for audio buffers:
```linen
allocBuffer : Int -> lin AudioBuffer
freeBuffer : lin AudioBuffer -> ()
```

### Hot Reload Attributes

State preservation across reloads:
```linen
#[stable("main-lfo")]
lfo = sinOsc 0.5
```

## VM Architecture

The VM has a **dual-context execution model**:

1. **Audio Context**: Hard real-time, no allocation, no blocking
   - Opcodes: 0xD0-0xDF (oscillators, filters, I/O)
   - Runs in audio callback at sample rate

2. **Control Context**: Soft real-time, GC allowed
   - Opcodes: 0xA0-0xCF, 0xF0-0xFF (FRP, closures, exceptions)
   - Handles UI, file I/O, compilation

3. **Universal Opcodes**: Both contexts (0x00-0x9F, 0xE0-0xEF)

Communication between contexts uses lock-free ring buffers with `ThreadMessage` types.

## Development Status

The project is in **early development** (Phase 0-1 of roadmap). Many components have scaffolding but are not fully implemented:

- Compiler: Structure defined, implementation pending
- VM: Structure defined, implementation pending
- CLI: Command structure exists, commands are stubs

Refer to `docs/roadmap.md` (in Russian) for detailed phase breakdown.

## Testing Strategy

- **Unit tests**: In each crate's `tests/` directory or inline
- **Property-based tests**: Using `proptest` for roundtrip testing
- **Snapshot tests**: Using `insta` for parser/output verification
- **Integration tests**: Will be in `tests/` at workspace root

## Dependencies Between Crates

```
linen-cli
├── linen-compiler
├── linen-vm
└── linen-stdlib
    └── linen-vm
```

## CI/CD Pipeline

GitHub Actions runs:
1. `cargo check --all-features`
2. `cargo fmt --all -- --check`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `cargo test --all-features --workspace` (Ubuntu, macOS)
5. `cargo doc --no-deps --all-features --workspace`
6. Code coverage with `cargo-tarpaulin`

## Documentation References

- **Language Spec**: `docs/language-spec.md` - Complete language reference
- **Architecture**: `docs/architecture.md` - Bytecode, AST, VM internals
- **FRP Patterns**: `.kimi/skills/frp-linen/SKILL.md` - FRP patterns and anti-patterns
- **Syntax Quick Ref**: `docs/syntax-summary.md` - Cheat sheet
- **Contributing**: `CONTRIBUTING.md` - Contribution guidelines

## Common Tasks

### Adding a New Opcode

1. Define in `crates/vm/src/bytecode.rs`
2. Add execution in `crates/vm/src/vm.rs`
3. Add codegen in `crates/compiler/src/codegen.rs`
4. Update `docs/architecture.md` §3

### Adding a Standard Library Function

1. Implement in appropriate module in `crates/stdlib/src/`
2. Add to VM primitives if needed
3. Document in language spec

### Working on Parser

The parser uses **chumsky** with the following pipeline:
```
Source → Tokenizer → Parser → AST Lowering → Typed AST
```

See `docs/architecture.md` §2 for AST node definitions.

## Security Considerations

- Audio thread must never allocate or block (real-time safety)
- FFI is restricted to built-in bindings (no user-defined FFI in v1.0)
- Linear types ensure resources are properly managed
- VM bytecode is validated before execution

## License

Dual-licensed under MIT OR Apache-2.0.
