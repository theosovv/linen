# Cross-Reference: Architecture vs Language Specification

This document maps concepts between implementation details (`architecture.md`) and user-facing syntax (`language-spec.md`).

## AST / Syntax Correspondence

| architecture.md | language-spec.md | Notes |
|-----------------|------------------|-------|
| `Expr::Lambda` | `\x -> e` | Backslash lambda syntax |
| `Expr::Let` | `let x = e1 in e2` | OCaml-style binding |
| `Expr::LetRec` | `let rec \| f = ...` | Mutual recursion |
| `Expr::If` | `if c then t else e` | Standard if-then-else |
| `Expr::Match` | `match x with \| P -> e` | Pattern matching |
| `Expr::App` | `f x y` | Left-associative application |
| `Expr::BinOp` | `x + y`, `x |> f` | Operators with precedence |
| `Expr::Proc` | `proc x -> do ...` | Arrow notation |
| `Expr::MacroInvoke` | `name!(args)` | Reserved for v1.0 |
| `Expr::Quote` | `quote! { ... }` | Reserved for v1.0 |
| `Expr::Unquote` | `#expr`, `#(list)` | Reserved for v1.0 |

## Type System

| architecture.md | language-spec.md | Notes |
|-----------------|------------------|-------|
| `Type::Var` | Implicit (e.g., `a`, `b`) | Polymorphic variables |
| `Type::Con` | `Int`, `Float`, `Behavior Float` | Type constructors |
| `Type::Arrow` | `a -> b` | Function type |
| `Type::Linear` | `lin AudioBuffer` | Linear types |
| `Type::Ephemeral` | (internal) | Audio-thread only |
| `TypeFlags::is_linear` | `lin` keyword | Resource safety |
| `TypeFlags::is_ephemeral` | (inferred) | Thread-bound |

## FRP Types

| architecture.md | language-spec.md | Notes |
|-----------------|------------------|-------|
| `Behavior<T>` | `Behavior Float` | Time-varying values |
| `Event<T>` | `Event ()` | Discrete events |
| `SF<A, B>` | `SF Float Float` | Signal functions |
| `arr` | `arr f` | Lift pure function |
| `>>>` | `sf1 >>> sf2` | Arrow composition |
| `&&&` | `sf1 &&& sf2` | Arrow split |
| `-<` | `f -< x` | Arrow application |
| `returnA` | `returnA -< x` | Arrow return |

## Bytecode / Runtime

| architecture.md | language-spec.md | Context |
|-----------------|------------------|---------|
| `ExecutionContext::Audio` | (implicit) | Audio callback only |
| `ExecutionContext::Control` | (implicit) | Control thread |
| `OSC_SIN` (0xD0) | `sinOsc` | Audio context |
| `BEHAVIOR_MAP` (0xA1) | `map` on Behavior | Control context |
| `LINEAR_MOVE` (0x90) | Implicit move | Both contexts |
| `LINEAR_CLONE` (0x91) | `clone` function | Both contexts |
| `LINEAR_DROP` (0x92) | `drop` function or implicit | Both contexts |
| `StatefulType::Oscillator` | `sinOsc`, `sawOsc`, etc. | State preservation |
| `StatefulType::ADSR` | `adsr` function | Envelope state |
| `StatefulType::DelayLine` | `delay` function | Buffer content |
| `StatefulType::Ref` | `ref`, `!`, `:=` | Mutable state |

## Module System

| architecture.md | language-spec.md | Notes |
|-----------------|------------------|-------|
| `Module` | `module Name where` | Module declaration |
| `Import` | `import Module` | Qualified import |
| `Import` with items | `import Module (a, b)` | Selective import |
| `Import` with alias | `import Module as Alias` | Renaming |
| Export list | `module Name (a, b) where` | Explicit exports |
| (default) | `module Name where` | Everything exported |

## Effects

| architecture.md | language-spec.md | Notes |
|-----------------|------------------|-------|
| `Audio<T>` monad | `Audio ()` | Effect type |
| `readWav` | `readWav :: String -> Audio ...` | File I/O |
| `audioOut` | `audioOut :: Behavior ... -> Audio ()` | Audio output |
| `pure` / `return` | `pure`, `return` | Lift to Audio |
| Thread messages | (internal) | Control-Audio communication |

## Attributes

| architecture.md | language-spec.md | Purpose |
|-----------------|------------------|---------|
| `#[stable(id)]` | `#[stable("name")]` | Hot reload ID |
| `#[jit]` | `#[jit]` | JIT compilation hint |
| `#[inline]` | `#[inline]` | Inlining hint |

## FFI

| architecture.md | language-spec.md | Notes |
|-----------------|------------------|-------|
| `ForeignDeclaration` | `foreign "C" name :: Type` | Built-in FFI |
| `Abi::C` | `"C"` | C ABI |
| `Abi::Internal` | `"rust"` | Rust ABI |

## Status Summary

### ✅ Implemented in Spec

| Feature | architecture.md | language-spec.md | Status |
|---------|-----------------|------------------|--------|
| Lambda | `Expr::Lambda` | `\x -> e` | ✅ |
| Let | `Expr::Let` | `let x = e in e2` | ✅ |
| Pattern match | `Expr::Match` | `match ... with` | ✅ |
| Types | `Type` enum | `type` syntax | ✅ |
| Linear types | `Type::Linear` | `lin` keyword | ✅ |
| FRP | `Behavior`, `Event` | `Behavior`, `Event` | ✅ |
| Arrow notation | `Expr::Proc` | `proc ... do` | ✅ |
| Pipe operator | (defined as function) | `\|>` | ✅ |
| Ref | `StatefulType::Ref` | `ref`, `!`, `:=` | ✅ |
| Units | (type aliases) | `Hz`, `Ms`, etc. | ✅ |
| Modules | `Module`, `Import` | `module`, `import` | ✅ |
| FFI | `ForeignDeclaration` | `foreign` | ✅ |

### ⚠️ Reserved for Future

| Feature | architecture.md | language-spec.md | Planned |
|---------|-----------------|------------------|---------|
| Macros | `Expr::MacroInvoke` | `name!(args)` | post-v1.0 |
| Quote | `Expr::Quote` | `quote! { ... }` | post-v1.0 |
| Unquote | `Expr::Unquote` | `#expr` | post-v1.0 |
| Type classes | (not in AST yet) | `class`, `instance` | post-v1.0 |

### 🔧 Implementation-Only

These exist in `architecture.md` but are not user-facing:

| Feature | Purpose |
|---------|---------|
| `ExecutionContext` | Audio vs Control thread safety |
| `NaN boxing` | Value representation optimization |
| `Opcode ranges` | Bytecode organization |
| `SectionId` | Module file format |
| `LinearChecker` | Debug-only verification |

## Quick Lookup

### I want to... → Look in...

| Task | Document | Section |
|------|----------|---------|
| Learn syntax basics | language-spec.md | §1-§4 |
| Write FRP code | language-spec.md | §5 |
| Understand audio effects | language-spec.md | §8 |
| Quick syntax lookup | syntax-summary.md | (all) |
| Implement lexer/parser | architecture.md | §2 |
| Implement type checker | architecture.md | §2.4 |
| Implement VM | architecture.md | §3 |
| Implement hot reload | architecture.md | §4.3 |
