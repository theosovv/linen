# Linen — Product Requirements Document

## 1. Overview

**Linen** — statically typed, purely functional, reactive programming language for algorithmic music and audio track generation.

**Goal:** Allow a composer to describe an audio graph as a reactive system of signals and events, execute it efficiently via interpreter, and export a finished track (WAV/FLAC). Native compilation via LLVM is planned for v0.2.

**Target Audience:** Solo music software developers, algorithmic composers, audio SaaS creators.

---

## 2. Goals & Non-Goals

### Goals (MVP — v0.1)
1. Static typing with HM type inference (Hindley-Milner)
2. FRP primitives: `Signal<T>` (continuous), `Event<T>` (discrete), `Behavior<T>` (time-varying)
3. Purely functional semantics — all side-effects through explicit FRP constructs
4. Own DSP runtime in Rust (oscillators, filters, envelopes, effects)
5. Tree-walking interpreter for immediate execution
6. REPL for rapid prototyping
7. Export to WAV/FLAC (offline bounce)
8. ML-like syntax with TS inspiration

### Non-Goals (v0.1)
- Native code compilation (LLVM)
- Live coding / hot-swap code
- VST/AU/LV2 plugin format
- GUI patch editor
- Real-time performance guarantees (hard realtime)
- Distributed audio processing

### Future Goals (v0.2+)
- LLVM compilation to native code
- JIT compilation for REPL
- Hot-swap via LLVM ORC
- VST3/LV2 export
- Types for sample-rate (Hz@44100 vs Hz@48000) in type system
- Pattern-DSL for rhythms (like TidalCycles)

---

## 3. User Stories

### US-1: Composer wants to create a drone
```linen
let drone = sin 220.0 |> gain 0.5 |> filter (lowpass 800.0 0.7)
export drone "drone.wav" 10.0  -- 10 seconds
```

### US-2: Algorithmic melody from events
```linen
let notes = cycle [440.0, 523.2, 659.2, 783.9] |> every 0.25
let melody = notes |> map (freq -> sin freq |> adsr 0.1 0.2 0.5 0.3) |> mix
export melody "melody.wav" 8.0
```

### US-3: Polyphony and voice control
```linen
let chord = [sin 261.6, sin 329.6, sin 392.0] |> sequenceA |> map (voices -> mix voices)
let vibrato = sin 5.0 |> map (x -> x * 2.0)
let result = chord |> map (voice -> voice + vibrato)
```

### US-4: Modularity — import patches
```linen
import "kick.ln" as Kick
import "snare.ln" as Snare

let beat = sequence [Kick.hit, silence 0.25, Snare.hit, silence 0.25]
```

---

## 4. Functional Requirements

### FR-1: Type System
- HM type inference with let-polymorphism
- Primitive types: `Int`, `Float`, `Bool`, `String`, `Unit`
- Audio-specific types: `Signal<T>`, `Event<T>`, `Hz`, `Seconds`, `Samples`, `Amplitude`
- Composite types: tuples `(A, B)`, records `{field: T}`, variants `type Option a = None | Some a`
- Type aliases: `type Filter = Signal Float -> Signal Float`
- Generics: `map : (a -> b) -> Signal a -> Signal b`

### FR-2: FRP Primitives
- `Signal<T>`: `T = f(time)` — continuous time-varying value
- `Event<T>`: discrete stream of `T` at specific times
- `Behavior<T>`: piecewise-constant or interpolated between events
- Combinators: `map`, `zipWith`, `filter`, `fold`, `sample`, `switch`, `gate`
- `liftA2`, `sequenceA` for applicative style

### FR-3: DSP Primitives (Runtime Library)
- Oscillators: `sin`, `saw`, `square`, `triangle`, `noise` (white, pink)
- Filters: `lowpass`, `highpass`, `bandpass`, `notch`, `resonant`
- Envelopes: `adsr`, `ar`, `perc`
- Effects: `delay`, `reverb`, `chorus`, `distortion`, `compressor`
- Utilities: `gain`, `pan`, `mix`, `split`, `resample`

### FR-4: Temporal Primitives
- `seconds : Float -> Seconds`
- `hz : Float -> Hz`
- `every : Seconds -> Event Unit` — metronome
- `cycle : [a] -> Event a` — cyclic pattern
- `random : Event a` — random events (seeded)
- `sequence : [Signal a] -> Signal a` — sequential playback

### FR-5: Execution Pipeline (v0.1 — Interpreter)
1. **Lexing** -> tokens
2. **Parsing** -> AST (ML-like syntax)
3. **Type Checking** -> HM inference, annotated AST
4. **Desugaring** -> Core Language (ANF — A-Normal Form)
5. **FRP Lowering** -> Signal graph (SSA for signals)
6. **Interpretation** -> tree-walk over Core AST, sample-by-sample evaluation
7. **Bounce** -> offline render to WAV via DSP runtime

### FR-6: REPL
- Read expressions, type-checking, evaluation via tree-walking interpreter
- Commands: `:type expr`, `:compile module`, `:export signal filename duration`, `:help`
- History and basic autocompletion

---

## 5. Non-Functional Requirements

### NFR-1: Performance
- Interpreted code: <= 5x overhead from hand-written C++ DSP (acceptable for offline bounce)
- REPL interpretation: acceptable for preview (< 5s for 30s track with simple graph)
- DSP primitives native Rust (not interpreted)

### NFR-2: Portability
- Host: Linux (JACK/ALSA backend for preview)
- WAV export: platform-independent (no audio backend required)

### NFR-3: Extensibility
- FFI to Rust for custom DSP nodes
- Module system for patch libraries

### NFR-4: Documentation
- Self-documenting: function types as documentation
- Inline comments `-- comment`

---

## 6. Success Metrics
- [ ] Interpretation "Hello Sine" (220Hz sine wave) to WAV in < 2 seconds
- [ ] Polyphony 8 voices with filters without clicks/distortions
- [ ] Type inference for 100-line patch without explicit annotations
- [ ] REPL launches simple patch in < 1 second

---

## 7. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Interpreter too slow for complex patches | Medium | Medium | Profile and optimize hot paths; DSP stays native; LLVM in v0.2 |
| FRP semantics don't fit DSP | Medium | High | Prototype in Haskell/PureScript for validation |
| Type complexity scares users | Medium | Medium | Good error messages, type aliases |
| LLVM codegen harder than expected (v0.2) | High | High | Start with Cranelift as fallback |

---

## 8. Open Questions
- Need linear types for audio buffers (to prevent double-free in graph)?
- How to represent polyphony in types: `Signal [Float]` or `Polyphonic Signal Float`?
- Need effect system for IO (export, file read) or sufficient top-level declarations?

---

*Version: 0.1*  
*Date: 2026-05-15*  
*Author: Linen Team*
