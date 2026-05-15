# Linen

> A statically-typed, purely functional, reactive programming language for algorithmic music.

```linen
let drone = sin (hz 220.0) |> gain 0.5 |> filter (lowpass (hz 800.0) 0.7)
export drone "drone.wav" (seconds 10.0)
```

## Features

- **FRP for Audio**: Signals, events, behaviors — all first-class values
- **Static Typing**: Hindley-Milner inference, phantom types for Hz/Seconds
- **Interpreter + REPL**: Immediate feedback for prototyping, with WAV export
- **LLVM Compilation** (v0.2): Native code, SIMD, zero-allocation audio thread
- **REPL**: Interactive environment with JACK preview
- **Module System**: Import patches, standard library

## Quick Start

### Installation
```bash
cargo install linc linen-repl
```

### Hello Sine
```bash
cat > hello.ln << 'EOF'
let tone = sin (hz 440.0)
export tone "hello.wav" (seconds 2.0)
EOF

linc run hello.ln
# hello.wav generated!
```

### REPL
```bash
$ linen-repl
> let x = sin (hz 220.0)
> :type x
Signal Float
> :export x "preview.wav" (seconds 5.0)
Exported to preview.wav
```

## Documentation

- [Product Requirements](docs/01_PRD_Linen.md)
- [Architecture](docs/06_Architecture_C4-L3.md)
- [Examples](docs/09_Example_Programs.md)
- [API Reference](docs/api/)

## Roadmap

| Version | Status | Features |
|---------|--------|----------|
| v0.1 | In Progress | Interpreter, REPL, DSP runtime, WAV export |
| v0.2 | Planned | LLVM backend, hot-swap, MIDI, VST3, pattern DSL |
| v0.3 | Planned | Typeclasses, linear types, dependent types |
| v0.4 | Planned | Package manager, LSP, VSCode extension |

## Contributing

See [Coding Standards](docs/10_Coding_Standards.md) and [Roadmap](docs/07_Roadmap.md).

## License

MIT

---

*Built with Rust, LLVM, and love for algorithmic music.*
