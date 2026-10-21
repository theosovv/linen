# Linen

Linen is a planned compile-time UI framework for the browser. It aims to use
single-file `.linen` components, compile them to direct DOM operations, and
statically analyze reactive dependencies.

> Reactive threads, woven at compile time.

The project is at the foundation stage. The compiler, runtime, CLI, and
toolchain are not implemented yet. This README describes the project direction
and should be updated as working capabilities become available; it does not
claim that planned features are already usable.

## Project specification

The product and language specification is the source of truth:

- [`docs/Linen-spec.md`](docs/Linen-spec.md)

It is currently written in Russian. Scope, syntax, architecture, and stage
acceptance criteria should be checked there before implementation decisions are
made.

## Development approach

The human author writes implementation code. The AI agent is used to draft and
maintain specifications, propose ADRs, and review code without modifying it.
See [`AGENTS.md`](AGENTS.md) for the repository's agent instructions.

## Roadmap

The first milestone is **Stage 0: Foundation**:

- Set up a Cargo workspace and the `packages/` directory.
- Add CI checks for formatting, Clippy, and tests.
- Create a fixture runner.
- Add the initial parser skeleton.
- Meet the Stage 0 definition of done in the specification: green CI and a
  parser skeleton.

Later milestones are tracked in the project specification. They are plans, not
implemented features.

## Planned technology

- Rust compiler in a Cargo workspace.
- Oxc for TypeScript parsing.
- Miette for compiler diagnostics.
- Salsa for incremental computation.
- TypeScript runtime package and Vite integration in later stages.

## License

No license has been selected yet. Do not assume the project is licensed for
reuse until a license file is added.
