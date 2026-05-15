# Linen — Glossary

## Audio Terms

| Term | Definition |
|------|------------|
| **Sample** | One amplitude value at a point in time (usually f32) |
| **Sample Rate (SR)** | Samples per second (44100, 48000, 96000 Hz) |
| **Buffer** | Block of samples processed in one callback (64, 128, 512, 1024) |
| **DSP** | Digital Signal Processing |
| **Oscillator** | Periodic signal generator (sin, saw, square) |
| **Filter** | Device that modifies signal spectrum (LP, HP, BP) |
| **Envelope** | Amplitude function over time (ADSR: Attack, Decay, Sustain, Release) |
| **Bounce** | Offline track rendering to file (vs realtime playback) |
| **Polyphony** | Simultaneous playback of multiple voices |
| **Zero-crossing** | Moment when signal passes through 0 (important for clickless switching) |
| **Aliasing** | Distortion from insufficient sample rate |
| **FFT** | Fast Fourier Transform — time to frequency conversion |
| **Wet/Dry** | Ratio of processed/unprocessed signal in an effect |
| **Gain** | Amplitude amplification/attenuation |
| **Pan** | Signal distribution between left/right channels |
| **MIDI** | Musical Instrument Digital Interface — protocol for musical events |
| **DAW** | Digital Audio Workstation — music production software |

## Programming Terms

| Term | Definition |
|------|------------|
| **FRP** | Functional Reactive Programming |
| **Signal** | Continuous stream of values over time (in FRP) |
| **Event** | Discrete occurrence at a specific time (in FRP) |
| **Behavior** | Time-varying value (in FRP) |
| **HM** | Hindley-Milner — type inference algorithm |
| **Phantom Type** | Type parameter not used in constructor, for type-level checks |
| **ANF** | A-Normal Form — IR where all arguments are atoms |
| **SSA** | Static Single Assignment — IR where each variable assigned once |
| **DAG** | Directed Acyclic Graph |
| **LLVM IR** | LLVM Intermediate Representation |
| **JIT** | Just-In-Time compilation |
| **FFI** | Foreign Function Interface — calling functions from other languages |
| **ADT** | Algebraic Data Type — sum type (variant) or product type (record) |
| **Typeclass** | Haskell concept: interface for types (analogous to Rust trait) |
| **Polymorphism** | Ability of function to work with different types |
| **Currying** | Transforming n-argument function into chain of 1-argument functions |
| **Pipeline** | Operator `|>` for sequential function composition |

## Compiler Terms

| Term | Definition |
|------|------------|
| **Lexer** | Breaking text into tokens |
| **Parser** | Building AST from tokens |
| **AST** | Abstract Syntax Tree |
| **Typechecker** | Type checking and inference |
| **Desugarer** | Transforming syntactic sugar to core language |
| **Codegen** | Target code generation (LLVM IR) |
| **Linker** | Joining object files into executable |
| **REPL** | Read-Eval-Print Loop — interactive environment |
| **LSP** | Language Server Protocol — IDE integration protocol |

## Linen-Specific Terms

| Term | Definition |
|------|------------|
| **Aural** | File with `.ln` extension — Linen source code |
| **Patch** | Module/file defining an audio graph |
| **Voice** | One polyphony voice — one signal path |
| **Graph** | Signal graph — DAG of processing nodes |
| **Node** | Graph node — oscillator, filter, effect, etc. |
| **Connection** | Graph edge — signal transfer between nodes |
| **Export** | Signal rendering command to audio file |
| **Preview** | Signal playback via JACK in REPL |
| **Stdlib** | Standard library: oscillators, filters, events |
| **Module** | Compilation unit — one `.ln` file |

---

*Version: 0.1*  
*Date: 2026-05-15*
