# Linen Syntax Quick Reference

A concise cheat sheet for Linen language syntax.

---

## Basics

```linen
-- Variables
x = 42
name = "Linen"

-- Type annotation
age : Int
age = 25

-- Functions
add x y = x + y
add : Int -> Int -> Int

-- Lambda
\x -> x + 1
\x y -> x + y
```

---

## Control Flow

```linen
-- If-then-else
if x > 0 then "positive" else "negative"

-- Pattern matching
match maybe with
| Just v  -> v
| Nothing -> 0

-- Guards
match n with
| n | n > 0 -> "positive"
  | n < 0 -> "negative"
  | otherwise -> "zero"
```

---

## Let Bindings

```linen
-- Simple
let x = 5 in x + 1

-- Multiple
let x = 5 in
let y = 3 in
x + y

-- Pattern
let (a, b) = (1, 2) in a + b

-- Mutable reference (for state)
let counter = ref 0 in
    counter := !counter + 1;
    !counter
```

---

## Data Types

```linen
-- Type alias
type Hz = Float

-- Data type
type Maybe a =
    | Nothing
    | Just a

-- Record
type Person = { name : String, age : Int }

-- Create
alice = { name = "Alice", age = 30 }

-- Access
alice.name

-- Update
bob = { alice | name = "Bob" }
```

---

## Lists and Comprehensions

```linen
-- List literal
[1, 2, 3, 4]

-- Range (if implemented)
[1..10]

-- Comprehension
[ x * 2 | x <- [1, 2, 3, 4] ]
[ x * 2 | x <- [1, 2, 3, 4], x > 2 ]
```

---

## Pipes (Important!)

```linen
-- Forward pipe (left to right)
x |> f |> g    -- g (f x)

-- Backward pipe
f <| x         -- f x

-- Example
sinOsc 440.0 |> gain 0.5 |> audioOut
```

---

## FRP (Functional Reactive Programming)

### Behaviors (Time-varying values)

```linen
-- Constant
constFive : Behavior Int
constFive = 5

-- Time
ramp = integral 1.0

-- Operations lift automatically
vibrato = 440.0 + 5.0 * sin (2.0 * pi * 5.0 * time)
```

### Events (Discrete occurrences)

```linen
-- Never
noEvent : Event a
noEvent = never

-- Periodic
metro : Hz -> Event ()
metro bpm = every (60.0 / bpm)

-- Merge
merged = merge event1 event2

-- Transform
mapE f event
filterE predicate event
```

### Signal Functions (Stateful)

```linen
-- Composition
sf1 >>> sf2

-- Parallel
sf1 &&& sf2

-- Proc notation
synth = proc midi -> do
    freq  <- arr midiToFreq -< midi
    osc   <- sinOsc         -< freq
    env   <- adsr 0.1 0.3 0.7 1.0 -< gate midi
    returnA -< osc * env
```

---

## Audio

```linen
-- Types
Behavior Hz      -- Frequency
Behavior Float   -- Audio signal
Event ()         -- Trigger

-- Oscillators
sinOsc 440.0
sawOsc 220.0
squareOsc 110.0
triOsc 55.0

-- Envelope
adsr attack decay sustain release gateOn gateOff

-- Filters
lpf frequency q input
hpf frequency q input
bpf frequency q input

-- Effects
gain level signal
delay time feedback signal
```

---

## Effects (Audio Monad)

```linen
-- Type
main : Audio ()

-- Do notation
main = do
    sample <- readWav "kick.wav"
    audioOut (play sample)

-- Return pure value
pure value
return value
```

---

## Modules

```linen
-- Declaration
module MyModule where

-- Exports (explicit)
module MyModule (public1, public2) where

-- Imports
import Std.Audio                    -- Qualified
import Std.Audio (sinOsc, gain)     -- Specific
import Std.Audio as A               -- Alias
import Std.Audio open               -- Unqualified
```

---

## Attributes

```linen
-- Stable ID for hot reload
#[stable("lfo")]
myLfo = sinOsc 0.5

-- JIT hint
#[jit]
heavyFunction x = ...

-- Inline
#[inline]
smallFunction x = x + 1
```

---

## Physical Units (Type-Safe)

```linen
type Hz = Float      -- Hertz (frequency)
type Ms = Float      -- Milliseconds (time)
type Sec = Float     -- Seconds
type Db = Float      -- Decibels

-- Safe usage
let freq : Hz = 440.0 in
let delay : Ms = 100.0 in
-- freq + delay  -- Error: different units!
```

---

## Linear Types (Resource Safety)

```linen
-- Linear type annotation
allocBuffer : Int -> lin AudioBuffer

-- Explicit operations
clone : lin a -> a      -- Copy (if Clone trait)
drop : lin a -> ()      -- Destroy

-- Usage
let buf = allocBuffer 1024 in
let copy = clone buf in
freeBuffer buf;          -- Consume original
process copy             -- Use copy
```

---

## Common Idioms

### Chaining operations

```linen
-- Without pipe (nested)
audioOut (gain 0.5 (sinOsc 440.0))

-- With pipe (linear)
sinOsc 440.0 |> gain 0.5 |> audioOut
```

### Handling Maybe

```linen
-- Pattern match
match maybe with
| Just x  -> x
| Nothing -> default

-- Default value
fromMaybe default maybe
```

### State with ref

```linen
#[stable("counter")]
counter = ref 0

increment = do
    counter := !counter + 1
    !counter
```

### Live coding pattern

```linen
#[stable("main-synth")]
synth = proc _ -> do
    -- Change these values live!
    let freq : Hz = 440.0
    let modAmt = 100.0
    
    lfo <- sinOsc -< 5.0
    osc <- sinOsc -< freq + lfo * modAmt
    
    returnA -< osc * 0.3

main = audioOut (synth ())
```

---

## Operator Precedence (High to Low)

1. `.` — Field access
2. `!` — Dereference
3. `* / %` — Multiplicative
4. `+ -` — Additive
5. `|>` — Pipe
6. `< <= > >=` — Comparison
7. `== !=` — Equality
8. `&&` — Logical AND
9. `||` — Logical OR
10. `-<` — Arrow application
11. `>>>` — Arrow composition
12. `->` — Function arrow

---

## Keywords

```
as      behavior  do       else      event     foreign   if
import  in        let      lin       match     module    proc
pure    rec       ref      return    returnA   then      type
where   with
```

---

## Escape Sequences

```
\n     -- Newline
\t     -- Tab
\r     -- Carriage return
\\     -- Backslash
\"     -- Quote
\xHH   -- Hex byte
\u{HHHH} -- Unicode
```

---

## Comments

```linen
-- Single line

{- Multi-line
   comment -}

{- Nested:
   {- inner -}
-}

-- | Documentation
```

---

## File Extensions

- `.ln` — Linen source file
- `.lnc` — Compiled Linen bytecode

---

## See Also

- [language-spec.md](language-spec.md) — Full language specification
- [architecture.md](architecture.md) — Implementation details
- [roadmap.md](roadmap.md) — Development roadmap
