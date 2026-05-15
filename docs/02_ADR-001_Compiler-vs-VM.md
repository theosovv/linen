# ADR-001: Compiler vs Self-Written VM for Linen

## Status
**Accepted** (Amended for v0.1 simplicity)

## Context
Linen — statically typed FRP language for algorithmic music. Key architectural decision: how to execute code — via compilation to native code or via self-written virtual machine?

## Decision
**Evolutionary approach: Interpreter for v0.1, LLVM backend for v0.2+.**

### v0.1 — Interpreter + REPL (Current)
1. **LinenInterpreter** — tree-walking interpreter over Core AST for immediate execution
2. **LinenRepl** — interactive environment with JACK preview
3. **DSP Runtime** — static Rust library (oscillators, filters, etc.)
4. **WAV Export** — offline bounce via interpreted signal graph

### v0.2 — LLVM Backend (Future)
5. **LinenCompiler** — HM-typechecker -> ANF -> LLVM IR (inkwell) -> ELF binary
6. **JIT for REPL** — LLVM ORC for hot-swap without interpreter fallback

## Rationale
After reviewing the complexity of a full LLVM backend for MVP, we decided to ship an interpreter first to get an end-to-end working product faster. The interpreter is sufficient for offline bounce and REPL prototyping. LLVM is retained as the long-term performance target.

## Consequences

### Positive
- **Faster MVP**: Interpreter works in days, not weeks
- **Simpler debugging**: No codegen bugs, no linking issues
- **Unified runtime**: REPL and `linc run` use the same execution path
- **Path to LLVM**: Core AST and ANF are reusable for LLVM codegen later
- **Rust ecosystem**: symphonia (WAV export), rustfft — ready crates

### Negative
- **Performance**: Interpreter is 10-50x slower than native code; complex patches may take minutes to bounce
- **No FRP fusion**: Cannot fuse `map . map` at codegen level yet
- **GC in interpreter**: Audio thread runs interpreted, not zero-allocation

## Mitigations for v0.1 Performance
- DSP primitives are still native Rust (not interpreted)
- Signal graph executes native `next_sample` calls, only graph topology is interpreted
- Bounce time target: <30s for 30s track with simple graph

## Alternatives Considered

### A. Pure VM (bytecode interpreter)
- **Pros**: Fast start, easy hot-swap, simple serialization
- **Cons**: 10-100x slower on DSP, hard to optimize signal graph, GC problems
- **Verdict**: Rejected — insufficient performance for complex patches

### B. VM + JIT (self-written)
- **Pros**: Control over codegen, learning experience
- **Cons**: Rabbit hole of optimizations, won't reach LLVM in reasonable time
- **Verdict**: Rejected — unjustified complexity for solo dev

### C. LLVM from day one (original plan)
- **Pros**: Best performance from the start
- **Cons**: 3+ months to working product, high risk of getting stuck in codegen
- **Verdict**: Deferred to v0.2 — too complex for MVP

### D. Cranelift instead of LLVM
- **Pros**: Faster compilation, simpler API, written in Rust
- **Cons**: Fewer optimizations, fewer platforms, smaller community
- **Verdict**: Possible alternative for v0.2 if inkwell proves too complex

## Implementation Notes
- Phase 1: REPL interpreter (1-2 weeks)
- Phase 2: DSP runtime + WAV export via interpreter (2 weeks)
- Phase 3: ANF + LLVM codegen for simple signals (v0.2, 3-4 weeks)
- Phase 4: Full compiler with DSP runtime linking (v0.2, 2-3 weeks)

## References
- [Inkwell](https://github.com/TheDan64/inkwell) — Safe Rust LLVM bindings
- [Cranelift](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift) — Codegen backend
- [Faust](https://faust.grame.fr) — Reference functional audio language
- [TidalCycles](https://tidalcycles.org) — Reference FRP for patterns
