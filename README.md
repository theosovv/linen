# Linen

[![CI](https://github.com/linen-lang/linen/actions/workflows/ci.yml/badge.svg)](https://github.com/linen-lang/linen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

**Linen** is a functional-reactive programming language designed for music and audio applications.

## Features

- 🎵 **Functional Reactive Programming (FRP)** — Model audio as time-varying behaviors and discrete events
- 🔄 **Hot Reload without State Loss** — Modify code live while preserving oscillator phases, envelope positions, and delay lines
- 🎛️ **Low-Latency Audio** — Real-time processing with specialized VM and JIT compilation
- 📝 **Strong Static Typing** — Hindley-Milner type system with extensions
- 🎹 **Live Coding Oriented** — Designed for interactive music performance

## Quick Start

```bash
# Clone the repository
git clone https://github.com/linen-lang/linen.git
cd linen

# Build
cargo build --release

# Run a program
./target/release/linen run examples/sine_wave.ln

# Start REPL
./target/release/linen repl
```

## Example

```linen
-- Simple sine wave synthesizer
let sampleRate = 48000.0
let freq = 440.0

let sineWave : Behavior Float =
  let phase = integral (freq / sampleRate) in
  sin (2.0 * pi * phase)

let main : Audio () =
  audioOut (sineWave * 0.5)
```

## Project Structure

This is a Cargo workspace with the following crates:

- `linen-compiler` — Lexer, parser, type checker, and code generation
- `linen-vm` — Virtual machine with real-time audio processing
- `linen-stdlib` — Standard library (oscillators, filters, effects)
- `linen-cli` — Command-line interface and REPL

## Development

### Prerequisites

- Rust >= 1.75
- For audio: JACK (Linux), CoreAudio (macOS)

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test --workspace
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features

# Run pre-commit hooks
pre-commit run --all-files
```

## License

This project is licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md) for the development plan.
