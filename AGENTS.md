# Linen: Functional-Reactive Music Programming Language

## Project Overview

**Linen** is a functional-reactive programming language designed specifically for music and audio applications. The project is currently at the specification/design stage — it consists of comprehensive language documentation but no implementation code yet.

The language is inspired by Haskell and OCaml syntax, with Functional Reactive Programming (FRP) as its core paradigm. Key features include:

- **Hot reload without state loss** — Incremental FRP (IFRP) allows code modification without losing audio state (oscillator phases, envelope positions, delay lines)
- **Strong static typing** — Hindley-Milner type system with extensions (type classes, linear types, limited dependent types)
- **Low-latency audio** — Specialized VM + JIT compilation targeting real-time audio processing
- **Live coding oriented** — Designed for interactive music performance

## Project Structure

```
/home/theosov/projects/linen/
├── AGENTS.md          # This file — agent instructions
├── docs/
│   └── linen.md       # Complete language specification (Russian)
└── .git/              # Git repository (empty, no commits yet)
```

The project currently contains only documentation. The specification in `docs/linen.md` covers:
- Language architecture and type system
- Audio primitives (oscillators, ADSR envelopes, filters)
- VM and JIT design
- Standard library design
- DAW integration plans
- Development workflow

## Technology Stack (Planned)

Based on the specification, the implementation is planned to use:

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Bootstrap compiler | OCaml (≥ 4.14) or Haskell | Initial implementation |
| Build system | dune (OCaml) or cabal (Haskell) | Compilation and packaging |
| Parser generator | menhir | Language parser |
| Audio I/O | PortAudio, JACK, CoreAudio | Cross-platform audio |
| File I/O | libsndfile | WAV file support |
| JIT compilation | Cranelift / LLVM | Hot code optimization |

## Build Commands (Planned)

Once implemented, the workflow will be:

```bash
# Install dependencies (OCaml path)
opam install . --deps-only

# Build the compiler/VM
dune build

# Run tests
dune test

# Interactive REPL
linen repl

# Run a Linen program
linen run examples/sine_wave.ln

# Compile to bytecode
linen compile -o output.bc source.ln

# Run with JIT optimization
linen run --jit source.ln
```

## Code Style Guidelines

### Language Syntax

Linen uses Haskell/OCaml-inspired syntax:

```linen
-- Function definition with type annotation
let sineWave : Behavior Float =
  let phase = integral (freq / sampleRate) in
  sin (2.0 * pi * phase)

-- Recursive functions
let rec fib n =
  if n < 2 then n else fib (n-1) + fib (n-2)

-- Pattern matching
match event with
| NoteOn (note, vel) -> triggerEnvelope
| NoteOff -> releaseEnvelope
```

### Naming Conventions (from spec)

- Functions and variables: `camelCase` or `snake_case`
- Type constructors: `PascalCase`
- Type variables: lowercase (`a`, `b`, `c`)
- Module names: `PascalCase`

### Type Annotations

Always include type annotations for top-level definitions:

```linen
let main : Audio () =
  audioOut (sineWave * 0.5)
```

## Key Language Concepts

### FRP Abstractions

| Concept | Type | Description |
|---------|------|-------------|
| `Behavior a` | `Time → a` | Continuous time-varying value |
| `Event a` | `[(Time, a)]` | Discrete occurrences with values |
| `SF a b` | Signal Function | Stateful signal transformation |

### Audio Primitives

| Primitive | Description |
|-----------|-------------|
| `sinOsc`, `sawOsc`, `squareOsc`, `triOsc` | Basic oscillators |
| `adsr` | ADSR envelope generator |
| `lpf`, `hpf`, `bpf` | Biquad filters |
| `audioOut` | Output to audio interface |
| `midiIn` / `midiOut` | MIDI I/O |

### Effects Monad

All I/O operations use the `Audio` monad:

```haskell
newtype Audio a = Audio { runAudio :: AudioContext -> IO a }
```

## Testing Strategy

From the specification, testing approaches include:

1. **Unit tests** for audio primitives
2. **Property-based testing** for signal functions
3. **Integration tests** with audio file comparison
4. **Real-time performance tests** — measuring deadline compliance

## Development Notes

### Important Considerations for Implementation

1. **No GC in audio thread** — Use region-based allocation, object pools, linear types
2. **Lock-free structures** — For communication between audio and control threads
3. **Deterministic execution** — Strict evaluation for timing-critical code
4. **State preservation** — IFRP requires term labeling and state migration

### Target Audio Backends

| Backend | Platform | Priority |
|---------|----------|----------|
| JACK | Linux, macOS | Primary |
| CoreAudio | macOS | Secondary |
| ALSA | Linux | Fallback |
| WASAPI | Windows | Windows support |
| PortAudio | All | Development/testing |

## Next Steps for Development

Based on the specification, implementation should proceed in this order:

1. **Lexer and Parser** — Implement language frontend
2. **Type checker** — Hindley-Milner with extensions
3. **Bytecode VM** — Stack-based VM for audio primitives
4. **Audio backend abstraction** — PortAudio integration
5. **JIT compiler** — Cranelift-based compilation
6. **Hot reload mechanism** — IFRP state migration
7. **Standard library** — Filters, effects, I/O
8. **DAW plugins** — VST3/AU integration

## References

- Main specification: `docs/linen.md` (in Russian)
- FRP theory: Yampa, Fran, Reactive-banana
- Audio programming: JACK, SuperCollider, Max/MSP

---

*Note: This project is currently in specification phase. No implementation code exists yet. The `docs/linen.md` file contains the complete technical specification for the language.*
