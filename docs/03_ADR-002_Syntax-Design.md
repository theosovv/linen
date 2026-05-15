# ADR-002: Linen Syntax — ML-like with TypeScript Inspiration

## Status
**Accepted**

## Context
Need syntax that:
1. Naturally expresses FRP primitives (pipeline operators, pattern matching)
2. Supports HM type inference without explicit annotations
3. Familiar to developers (TS/JS, Rust, Haskell communities)
4. Unambiguous for parsing (no ambiguity)

## Decision
**ML-like syntax with TS inspiration for types and records.**

### Key Features:
- **Significant indentation** (offside rule) for blocks — like Haskell/Python
- **Pipeline operator** `|>` for signal composition (like Elm, F#)
- **Pattern matching** with `match ... with`
- **Type annotations** postfix via `:` (like TS), but inference-first
- **Records** JS-like `{ field: value }`, access via `.field`
- **Variants** Haskell-like: `type Option a = None | Some a`
- **Generics** via `<>`: `Signal<Float>`, `map<a, b>`
- **Comments** `-- line comment`, `{- block comment -}`

## Syntax Examples

### Basic Signal
```linen
let drone : Signal Float = 
  sin 220.0 
  |> gain 0.5 
  |> filter (lowpass 800.0 0.7)
```

### Pattern Matching
```linen
type Note = Rest | Tone { freq: Hz, vel: Float }

let playNote : Note -> Signal Float = fn note ->
  match note with
  | Rest -> silence
  | Tone { freq, vel } -> sin freq |> gain vel
```

### Events and Sampling
```linen
let metronome : Event Unit = every (seconds 0.5)
let notes : Event Float = metronome |> map (\_ -> randomChoice [440.0, 523.2, 659.2])
let melody : Signal Float = notes |> map (freq -> sin freq |> adsr 0.1 0.3 0.5 0.2) |> mixEvents
```

### Polymorphism
```linen
let map : (a -> b) -> Signal a -> Signal b = fn f sig -> ...
let sequenceA : [Signal a] -> Signal [a] = fn sigs -> ...
```

### Module Import
```linen
import "std/oscillators" as Osc
import "std/filters" as Filter

let patch = Osc.sin 440.0 |> Filter.lowpass 1000.0 0.5
```

## Consequences

### Positive
- FRP reads naturally: `sig |> map f |> zipWith g sig2`
- Pattern matching ideal for musical structures (notes, chords, scales)
- Type inference removes boilerplate
- Familiar to Haskell/Elm/OCaml developers

### Negative
- Significant indentation requires careful parser
- Pipeline operator `|>` — not standard in ML, requires explanation
- Indentation vs braces — controversial choice for some

## Alternatives Considered

### A. Lisp-like (S-expressions)
```lisp
(def drone (-> (sin 220.0) (gain 0.5) (lowpass 800.0 0.7)))
```
- **Pros**: Simple parser, macros
- **Cons**: Many parentheses, FRP less readable
- **Verdict**: Rejected — target audience prefers infix

### B. C-like (Rust/TS syntax)
```rust
let drone: Signal<f32> = sin(220.0).gain(0.5).filter(lowpass(800.0, 0.7));
```
- **Pros**: Familiar to wide audience
- **Cons**: Method chaining poorly fits FRP (signal as first argument), type inference harder
- **Verdict**: Rejected — ML pipeline better for signals

### C. TidalCycles-like (mini-notation)
```
d1 $ sound "bd sd*2 cp"
```
- **Pros**: Ideal for rhythms
- **Cons**: Too specific, not Turing-complete
- **Verdict**: Possible as embedded DSL in v0.2, but not main syntax

## Grammar (EBNF subset)
```
module     ::= import* declaration*
declaration ::= "let" ident [":" type] "=" expr
             | "fn" ident param+ [":" type] "->" expr

expr       ::= pipeline
pipeline   ::= apply ("|>" apply)*
apply      ::= primary primary*
primary    ::= number | string | ident | "(" expr ")" | record | list | lambda | match

lambda     ::= "\" param+ "->" expr
match      ::= "match" expr "with" ("|" pattern "->" expr)+
pattern    ::= ident | "_" | number | record-pattern | variant-pattern

record     ::= "{" [field-binding ("," field-binding)*] "}"
list       ::= "[" [expr ("," expr)*] "]"

variant    ::= ident ["(" type ")"] ("|" variant)*
type       ::= ident ["<" type ("," type)* ">"] | "(" type ")" | type "->" type
```

## References
- [Elm Syntax](https://elm-lang.org/docs/syntax)
- [F# Pipeline](https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/functions/#function-composition-and-pipelining)
- [Haskell Offside Rule](https://www.haskell.org/onlinereport/haskell2010/haskellch10.html#x17-17800010.3)
