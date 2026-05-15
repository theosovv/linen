# ADR-004: Linen Type System

## Status
**Accepted**

## Context
Need type system that:
1. Supports HM inference (no explicit annotations in most cases)
2. Expresses audio-specific concepts (signals, events, frequencies, time)
3. Prevents runtime errors (e.g., feeding Events where Signal needed)
4. Compiles efficiently

## Decision
**Hindley-Milner with let-polymorphism + audio primitives + phantom types for units.**

### Type Hierarchy

#### Primitive Types
```linen
type Int        -- 64-bit signed integer
type Float      -- 64-bit float (f64)
type Bool       -- true | false
type String     -- UTF-8 string
type Unit       -- ()
type Time       -- sample count (u64)
```

#### Audio-Specific Types (Phantom Types)
```linen
type Hz         -- phantom type for frequency (Float underneath)
type Seconds    -- phantom type for time duration
type Samples    -- phantom type for sample count
type Amplitude  -- phantom type for gain (0.0..1.0)
type Bpm        -- phantom type for tempo
```
- Phantom types: runtime — ordinary Float, but typechecker prevents adding `Hz` and `Seconds`
- Constructors: `hz : Float -> Hz`, `seconds : Float -> Seconds`, `bpm : Float -> Bpm`

#### FRP Types
```linen
type Signal a   -- continuous signal of a
type Event a    -- discrete event stream of a
type Behavior a -- time-varying value (Event a -> Signal a)
```

#### Composite Types
```linen
type (a, b)           -- tuple
type [a]              -- list (homogeneous, fixed? or stream?)
type { field: a, ... } -- record
type Option a = None | Some a   -- variant/algebraic data type
type Result e a = Err e | Ok a
```

### Type Inference

#### HM Algorithm W
- Standard Algorithm W with let-polymorphism
- Generalization at let-bindings
- Instantiation at variable use

#### Special Cases
```linen
-- Polymorphism through let
let id x = x              -- id : a -> a
let f = id 5              -- f : Int
let g = id "hello"        -- g : String

-- Constraint through annotation
let freq : Hz = hz 440.0  -- freq : Hz, not Float
```

### Function Types
```linen
type Map = (a -> b) -> Signal a -> Signal b
type ZipWith = (a -> b -> c) -> Signal a -> Signal b -> Signal c
type Adsr = Seconds -> Seconds -> Float -> Seconds -> Signal Amplitude
type Sin = Hz -> Signal Float
```

### Type Constraints (Future v0.2)
- **Numeric typeclass**: `Num a` for `+`, `-`, `*`
- **Ordered typeclass**: `Ord a` for comparison
- **Signal typeclass**: `Signalizable a` for types that can be signals

## Consequences

### Positive
- Type inference removes boilerplate
- Phantom types prevent unit-of-measure errors
- ADT and pattern matching — powerful abstractions for music

### Negative
- HM doesn't support subtyping — `Signal Float` and `Signal Amplitude` — different types
- HM error messages can be cryptic — need good diagnostics
- Without typeclasses much duplication (`mapSignal`, `mapEvent`, `mapList`)

## Alternatives Considered

### A. Dependent Types (Idris/Agda-like)
- **Pros**: Types for sample-rate, buffer-size, track duration
- **Cons**: Harder inference, overkill for MVP
- **Verdict**: Possible in v0.3 for advanced users

### B. Linear Types
- **Pros**: Guarantee single use of audio buffers
- **Cons**: Harder programming, not needed for offline bounce
- **Verdict**: Rejected — add later for realtime version

### C. Refinement Types
- **Pros**: `Amplitude` as `{ x: Float | 0.0 <= x <= 1.0 }`
- **Cons**: Requires SMT solver, complicates compiler
- **Verdict**: Possible in v0.2 via runtime assertions

## Implementation

### Typechecker Architecture
```rust
// Rust pseudo-code
enum Type {
    Var(TypeVar),
    Con(TypeCon, Vec<Type>),  -- Signal(Float), Event(Hz)
    Arrow(Box<Type>, Box<Type>),  -- a -> b
    Tuple(Vec<Type>),
    Record(Vec<(String, Type)>),
}

struct TypeEnv {
    bindings: HashMap<String, Scheme>,  -- Scheme = forall vars. Type
}

fn infer(env: &TypeEnv, expr: &Expr) -> Result<Type, TypeError> {
    // Algorithm W implementation
}
```

### Phantom Types Implementation
```linen
-- Internal representation: all phantom types = Float
-- Typechecker tracks differences

type Hz = Float  -- phantom
type Seconds = Float  -- phantom

let addHz : Hz -> Hz -> Hz = fn a b -> a + b  -- OK
let bad : Hz -> Seconds -> Float = fn h s -> h + s  -- TYPE ERROR!
```

## References
- [Hindley-Milner Type Inference](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- [Phantom Types in Haskell](https://wiki.haskell.org/Phantom_type)
- [Type Inference for DSP](https://www.cs.indiana.edu/~sabry/papers/dsp-types.pdf)
