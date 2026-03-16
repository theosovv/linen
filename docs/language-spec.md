# Linen Language Specification

**Version:** 0.1.0 (Draft)  
**Status:** Work in Progress

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Types](#3-types)
4. [Expressions](#4-expressions)
5. [FRP (Functional Reactive Programming)](#5-frp-functional-reactive-programming)
6. [Pattern Matching](#6-pattern-matching)
7. [Modules](#7-modules)
8. [Audio and Effects](#8-audio-and-effects)
9. [FFI (Foreign Function Interface)](#9-ffi-foreign-function-interface)
10. [Attributes and Metadata](#10-attributes-and-metadata)
11. [Complete Examples](#11-complete-examples)

---

## 1. Introduction

Linen is a functional-reactive programming language designed for music and audio applications. It combines:

- **Functional programming**: Immutable data, first-class functions, strong static typing
- **Reactive programming**: Time-varying values (Behaviors) and discrete events
- **Real-time audio**: Low-latency processing with hot reload

### Design Philosophy

- **Composability**: Small, reusable functions compose into complex systems
- **Safety**: Type system prevents common audio programming errors
- **Live coding**: Modify programs while they run without audio glitches

### Hello Sine

```linen
module Main where

import Std.Audio (sinOsc, audioOut, gain)

-- A pure sine wave at 440 Hz
mySine : Behavior Hz
mySine = sinOsc 440.0

-- Main entry point with audio output
main : Audio ()
main = sinOsc 440.0 |> gain 0.5 |> audioOut
```

---

## 2. Lexical Structure

### 2.1 Comments

```linen
-- This is a single-line comment

{- This is a
   multi-line comment -}

{- Comments can be nested:
   {- inner comment -}
-}
```

### 2.2 Identifiers

- **Variables and functions**: `camelCase` or `snake_case`
  - Valid: `frequency`, `myOsc`, `attack_time`
  - Invalid: `MyOsc` (reserved for constructors), `123abc` (starts with number)

- **Type constructors**: `PascalCase`
  - Valid: `Behavior`, `Maybe`, `AdsrEnvelope`

- **Type variables**: Lowercase, single letters preferred
  - Valid: `a`, `b`, `freq`, `phase`

### 2.3 Keywords

Reserved words that cannot be used as identifiers:

```
as      behavior  do       else      event     foreign   if
import  in        let      lin       match     module    proc
pure    rec       ref      return    returnA   then      type
where   with
```

### 2.4 Literals

#### Numbers

```linen
-- Integers
42
-17
0xFF        -- hexadecimal
0b1010      -- binary
1_000_000   -- underscores for readability

-- Floats
3.14
-0.5
1.0e-3      -- scientific notation
440.0
3.141_592   -- underscores

-- Optional suffixes
440.0f32    -- 32-bit float
3.14f64     -- 64-bit float (default)
```

#### Strings

```linen
"Hello, World!"
"Line 1\nLine 2"        -- escape sequences
"Quote: \"text\""        -- escaped quotes
"Tab\there"             -- \t
"Carriage\r\n"          -- \r
"Hex\x41"               -- hex byte (A)
"Unicode\u{2764}"       -- Unicode (❤)

-- String interpolation
let name = "Linen" in
"Hello, {name}!"        -- = "Hello, Linen!"
```

Escape sequences: `\n` (newline), `\t` (tab), `\r` (carriage return), `\\` (backslash), `\"` (quote), `\xHH` (hex byte), `\u{HHHH}` (Unicode).

#### Other

```linen
true   -- Boolean
false  -- Boolean
()     -- Unit (empty tuple)
```

### 2.5 Operators

#### Arithmetic
```
+  -  *  /  %
```

#### Comparison
```
==  !=  <  <=  >  >=
```

#### Logical
```
&&  ||  not
```

#### Function Composition and Application
```
|>      -- Forward pipe: x |> f = f x
<|      -- Backward pipe: f <| x = f x
>>>     -- Sequential composition (arrows)
&&&     -- Parallel composition (split)
***     -- First and second
```

#### Assignment and Dereference
```
:=      -- Mutable assignment (for ref)
!       -- Dereference (for ref)
```

#### Other
```
|       -- Pattern separator
->      -- Function arrow, pattern result, lambda
=>      -- Type constraint arrow
-<      -- Arrow application (in proc)
_       -- Wildcard pattern
```

### 2.6 Layout and Indentation

Linen uses **off-side rule** (like Haskell, Python, F#). Indentation determines block structure:

```linen
-- Correct: body of 'let' is indented
let x = 5 in
    x + 1

-- Correct: alternatives in 'match' aligned
match x with
| Some v -> v
| None   -> 0

-- Incorrect: inconsistent indentation
match x with
| Some v -> v
  | None -> 0   -- Error: not aligned
```

**Rules:**
- Indent with spaces (recommended: 4)
- Opening keyword (`let`, `where`, `do`, `match`) starts a block
- Subsequent lines must be indented more than the keyword
- Lines at the same indentation level belong to the same block

---

## 3. Types

### 3.1 Basic Types

```linen
Int         -- 64-bit signed integer
Float       -- 64-bit floating point
Bool        -- true or false
String      -- UTF-8 text
()          -- Unit (empty tuple, void)
```

### 3.2 Unit Types (Type-Safe Physical Units)

```linen
type Hz = Float      -- Frequency in Hertz
type Ms = Float      -- Time in milliseconds
type Sec = Float     -- Time in seconds
type Db = Float      -- Decibels
type Samples = Int   -- Sample count
```

Unit types are **distinct** — the compiler prevents mixing them:

```linen
let freq : Hz = 440.0 in
let delay : Ms = 100.0 in

-- freq + delay  -- Error: Cannot add Hz and Ms
freq + 100.0     -- OK: 100.0 is Hz (inferred)
freq + (delay |> msToHz)  -- OK: explicit conversion
```

### 3.3 Function Types

```linen
-- Function type: argument -> result
Int -> Int              -- Integer to integer
Float -> Float -> Float -- Curried: takes two floats

-- Type annotation
add : Int -> Int -> Int
add x y = x + y
```

### 3.4 Type Constructors

```linen
type Maybe a =
    | Nothing
    | Just a

type List a =
    | Nil
    | Cons a (List a)

type Result e a =
    | Ok a
    | Err e
```

**Syntax:**
- `type` keyword
- Name with optional type parameters
- `=` followed by constructor alternatives
- Each alternative starts with `|` and PascalCase name
- Constructors can have fields (space-separated)

### 3.5 Type Aliases

```linen
type Stereo a = (a, a)
type MidiNote = Int
type Velocity = Int
type Channel = Int
```

Type aliases are purely syntactic — `MidiNote` and `Int` are interchangeable.

### 3.6 FRP Types

```linen
-- Time-varying continuous value
Behavior Float        -- Signal that changes over time
Behavior Hz           -- Can use type aliases

-- Discrete event stream
Event ()              -- Trigger event
Event (Note, Velocity) -- Event carrying MIDI data

-- Signal function (stateful transformation)
SF Float Float        -- Input Float, output Float
```

### 3.7 Polymorphism

```linen
-- Type variables are implicitly forall
identity : a -> a
identity x = x

-- Multiple type variables
map : (a -> b) -> List a -> List b
map f Nil = Nil
map f (Cons x xs) = Cons (f x) (map f xs)

-- Type constraints (type classes)
sum : Num a => List a -> a
sum Nil = 0
sum (Cons x xs) = x + sum xs
```

### 3.8 Linear Types (Resource Safety)

Resources that must be used exactly once:

```linen
-- Linear type annotation (prefix with 'lin')
allocBuffer : Int -> lin AudioBuffer
freeBuffer : lin AudioBuffer -> ()

-- Explicit operations
clone : lin a -> a      -- Create non-linear copy (if Clone trait)
drop : lin a -> ()      -- Explicit destroy (usually automatic)

-- Type error: buffer not freed
bad = 
    let buf = allocBuffer 1024 in
    ()  -- Error: linear resource 'buf' not consumed

-- Correct: buffer freed
good = 
    let buf = allocBuffer 1024 in
    freeBuffer buf

-- Correct: cloned then freed
alsoGood =
    let buf = allocBuffer 1024 in
    let copy = clone buf in
    freeBuffer buf;
    process copy  -- 'copy' is regular value, 'buf' is consumed
```

**Note:** Linear types are primarily used for audio resources (buffers, streams). Most user code uses regular types.

### 3.9 Ephemeral Types (Audio Thread Only)

Types valid only in audio context:

```linen
-- Ephemeral type (valid only in audio callback)
type Ephemeral a = ...

-- Example: Direct hardware access
readHardware : Ephemeral Samples

-- Cannot escape audio thread:
bad = proc _ -> do
    samples <- readHardware -< ()
    returnA -< samples  -- Error: Ephemeral cannot leave audio context
```

---

## 4. Expressions

### 4.1 Let Bindings

```linen
-- Simple binding
let x = 5 in x + 1

-- With type annotation
let y : Float = 3.14 in y * 2.0

-- Multiple bindings (sequential)
let x = 5 in
let y = 3 in
x + y

-- Pattern binding
let (a, b) = (1, 2) in a + b

-- Mutable reference (for hot reload state)
let counter = ref 0 in
    counter := !counter + 1;
    !counter

-- Where clause (alternative syntax)
g x = y + z
    where
    | y = x * 2
    | z = x + 1
```

### 4.2 Mutable References

```linen
-- Create mutable reference
ref : a -> lin (Ref a)

-- Read (dereference)
! : Ref a -> a

-- Write
:= : Ref a -> a -> ()

-- Example: counter
tick : Ref Int -> Int
tick counter =
    counter := !counter + 1;
    !counter

-- Usage in hot reload
#[stable("sequencer-step")]
stepRef = ref 0

sequencer = proc trigger -> do
    step <- hold 0 (mapE (\_ -> tick stepRef) trigger)
    returnA -< step
```

### 4.3 Functions

#### Lambda Expressions

```linen
-- Anonymous function (backslash syntax)
\x -> x + 1

-- Multiple parameters
\x y -> x + y

-- With pattern
\(a, b) -> a + b
```

#### Function Definitions

```linen
-- Top-level definition
add x y = x + y

-- With type annotation
square : Float -> Float
square x = x * x

-- Pattern matching in parameters
fst (a, _) = a
snd (_, b) = b

-- Recursive definition
factorial n =
    if n <= 1 then 1
    else n * factorial (n - 1)

-- Mutual recursion
let rec 
    | even n = if n == 0 then true else odd (n - 1)
    | odd n = if n == 0 then false else even (n - 1)
in even 42
```

### 4.4 Pipe Operators

```linen
-- Forward pipe (most common)
(|>) : a -> (a -> b) -> b
x |> f = f x

-- Backward pipe
(<|) : (a -> b) -> a -> b
f <| x = f x

-- Examples
-- Without pipe (hard to read inside-out)
audioOut (gain 0.5 (sinOsc 440.0))

-- With pipe (left-to-right data flow)
sinOsc 440.0 |> gain 0.5 |> audioOut

-- Mixed with regular application
[1, 2, 3, 4] 
    |> filter (\x -> x > 2)
    |> map (\x -> x * 2)
    |> sum
```

### 4.5 Conditionals

```linen
-- If-then-else
if x > 0 then "positive" else "non-positive"

-- Nested
if x < 0 then "negative"
else if x > 0 then "positive"
else "zero"
```

### 4.6 Operators

```linen
-- Infix application
1 + 2 * 3       -- Standard precedence

-- Sectioning (partial application)
(+ 1)           -- \x -> x + 1
(2 *)           -- \x -> 2 * x

-- Custom operators (defined as functions)
(|>) x f = f x          -- Pipe forward

-- Arrow composition
(>>>) : SF a b -> SF b c -> SF a c
```

### 4.7 Tuples

```linen
-- Construction
(1, 2)
("hello", 42, true)

-- Deconstruction (pattern matching)
let (x, y) = point in ...

-- Unit (empty tuple)
()
```

### 4.8 Records

```linen
-- Record type
type Person = { name : String, age : Int }

-- Construction
alice = { name = "Alice", age = 30 }

-- Field access
alice.name

-- Update (F#-style)
bob = { alice | name = "Bob" }

-- Functional update
growOlder p = { p | age = p.age + 1 }
```

### 4.9 List Comprehensions

```linen
-- Basic comprehension
[ x * 2 | x <- [1, 2, 3, 4] ]
-- = [2, 4, 6, 8]

-- With filter
[ x * 2 | x <- [1, 2, 3, 4], x > 2 ]
-- = [6, 8]

-- Multiple generators
[ (x, y) | x <- [1, 2], y <- [10, 20] ]
-- = [(1, 10), (1, 20), (2, 10), (2, 20)]

-- With let
[ y | x <- [1, 2, 3], let y = x * x, y > 3 ]
-- = [4, 9]
```

---

## 5. FRP (Functional Reactive Programming)

### 5.1 Behaviors

Behaviors are time-varying continuous values.

```linen
-- Constant behavior (pure lifts automatically)
constFive : Behavior Int
constFive = 5

-- Lifted operations
ramp : Behavior Float
ramp = integral 1.0     -- Increases by 1 per second

-- Time-varying frequency
vibrato : Behavior Hz
vibrato = 440.0 + 5.0 * sin (2.0 * pi * 5.0 * time)
```

**Note:** Most arithmetic operators lift automatically to Behaviors.

### 5.2 Events

Events are discrete occurrences in time.

```linen
-- Never occurs
noEvent : Event a
noEvent = never

-- Periodic trigger
metro : Hz -> Event ()
metro bpm = every (60.0 / bpm)

-- Mouse clicks (conceptual)
clicks : Event (Float, Float)
```

### 5.3 Event Combinators

```linen
-- Merge two events (left-biased for simultaneous)
merged : Event a -> Event a -> Event a
merged = merge

-- Transform event values
withValue : Event a -> (a -> b) -> Event b
withValue ev f = mapE f ev

-- Sample behavior at event occurrences
snapshot : Event a -> Behavior b -> Event (a, b)

-- Gate: filter events when condition is true
when : Event a -> Behavior Bool -> Event a
```

### 5.4 Switching

Convert between Events and Behaviors:

```linen
-- Hold: event values become behavior
hold : a -> Event a -> Behavior a

-- Accumulate: stateful accumulation
accum : a -> Event (a -> a) -> Behavior a

-- Step: discrete changes
step : Event a -> Behavior a

-- Switch: change behavior on event
switch : Behavior a -> Event (Behavior a) -> Behavior a
```

### 5.5 Signal Functions (Arrows)

Signal functions are stateful transformations with explicit input/output.

#### Basic Combinators

```linen
-- Identity
identity : SF a a

-- Lifting pure functions
arr : (a -> b) -> SF a b

-- Composition
(>>>) : SF a b -> SF b c -> SF a c

-- Parallel composition
(&&&) : SF a b -> SF a c -> SF a (b, c)

-- First/second
first : SF a b -> SF (a, c) (b, c)
second : SF a b -> SF (c, a) (c, b)
```

#### Proc Notation

For complex signal function composition:

```linen
-- Without proc notation (hard to read)
synth = arr midiToFreq >>> sinOsc >>> arr (* 0.5)

-- With proc notation (clear data flow)
synth = proc midiMsg -> do
    freq   <- arr midiToFreq  -< midiMsg
    osc    <- sinOsc          -< freq
    env    <- adsr 0.1 0.3 0.7 1.0 -< gateFromMidi midiMsg
    returnA -< osc * env
```

**Syntax:**
- `proc param -> do` introduces arrow notation
- `result <- function -< input` applies function to input
- `returnA -< value` produces output
- `rec` for recursive definitions

#### Example: Recursive Delay

```linen
echo = proc input -> do
    rec
        -- Mix input with delayed feedback
        let mixed = input + feedback * 0.5
        -- Delay line
        delayed <- delay 0.3 -< mixed
        let feedback = delayed
    returnA -< mixed
```

### 5.6 Common Patterns

```linen
-- Low-frequency oscillator (LFO)
lfo : Hz -> Behavior Float
lfo freq = sin (2.0 * pi * freq * time)

-- Amplitude modulation
am : Behavior Float -> Behavior Float -> Behavior Float
am carrier modulator = carrier * (0.5 + 0.5 * modulator)

-- Frequency modulation
fm : Hz -> Behavior Float -> Float -> Behavior Float
fm freq modulator index = 
    sinOsc (freq + index * modulator)
```

---

## 6. Pattern Matching

### 6.1 Match Expression

```linen
match value with
| pattern1 -> result1
| pattern2 -> result2
| _        -> defaultResult
```

### 6.2 Patterns

```linen
-- Variable pattern (binds value)
| x -> ...

-- Wildcard (ignores value)
| _ -> ...

-- Literal pattern
| 0 -> "zero"
| 1 -> "one"

-- Constructor pattern
| Nothing -> 0
| Just x  -> x

-- Nested pattern
| Just (x, y) -> x + y

-- List-like patterns
| Nil -> 0
| Cons x xs -> x

-- As-pattern
| list@(Cons x _) -> ...  -- 'list' is whole value, 'x' is head

-- Guard clauses
| n | n > 0 -> "positive"
  | n < 0 -> "negative"
  | otherwise -> "zero"
```

### 6.3 Exhaustiveness Checking

The compiler checks that all cases are covered:

```linen
-- Error: missing Nothing case
bad : Maybe Int -> Int
bad x = match x with | Just n -> n

-- Correct
good : Maybe Int -> Int
good x = match x with
    | Just n -> n
    | Nothing -> 0
```

---

## 7. Modules

### 7.1 Module Declaration

```linen
-- File: MyModule.ln
module MyModule where

-- exports go here
```

### 7.2 Explicit Exports

By default, **all** top-level definitions are exported. Explicit exports restrict the public API:

```linen
-- Explicit exports (recommended)
module MySynth (myOsc, filter) where

myOsc = ...      -- exported
filter = ...     -- exported
secret = ...     -- NOT exported (private)
```

**Rules:**
- Names in export list are public
- Names not in export list are private to the module
- Type constructors must be exported explicitly: `Type(Cons1, Cons2)`

### 7.3 Imports

```linen
-- Import everything (qualified by default)
import Std.Audio
-- Use: Std.Audio.sinOsc

-- Import specific items
import Std.Audio (sinOsc, gain, audioOut)
-- Use: sinOsc, gain, audioOut directly

-- Import with alias
import Std.Audio as A
-- Use: A.sinOsc

-- Import specific items as open (no qualification)
import Std.Audio (sinOsc, gain) open
-- Can use: sinOsc, gain directly

-- Import type with constructors
import Std.Maybe (Maybe(Nothing, Just))
```

### 7.4 Module Hierarchy

```linen
-- Std/Audio/Oscillators.ln
module Std.Audio.Oscillators where

-- Import parent
import Std.Audio

-- Submodules can be imported individually
import Std.Audio.Oscillators (sinOsc, sawOsc)
```

---

## 8. Audio and Effects

### 8.1 The Audio Monad

Audio operations (I/O, file reading, device control) use the `Audio` monad:

```linen
-- Type signature indicating audio effects
main : Audio ()

-- Do notation for sequencing effects
main = do
    -- Read audio file
    sample <- readWav "drum.wav"
    
    -- Get MIDI input
    midiEvents <- midiIn
    
    -- Setup audio output
    audioOut (process sample midiEvents)

-- Return pure value in Audio context
pure : a -> Audio a
return : a -> Audio a  -- alias for pure
```

### 8.2 Audio Primitives

```linen
-- Oscillators (pure, lift to Behavior automatically)
sinOsc : Hz -> Behavior Float
sawOsc : Hz -> Behavior Float
squareOsc : Hz -> Behavior Float
triOsc : Hz -> Behavior Float

-- Envelopes
adsr : Ms -> Ms -> Float -> Ms -> Event () -> Event () -> Behavior Float
--       A     D      S      R       gate-on       gate-off

-- Filters
lpf : Hz -> Float -> Behavior Float -> Behavior Float
--     freq   Q       input signal         output
hpf : Hz -> Float -> Behavior Float -> Behavior Float
bpf : Hz -> Float -> Behavior Float -> Behavior Float
```

### 8.3 Audio I/O

```linen
-- Output to speakers (stereo)
audioOut : Behavior (Float, Float) -> Audio ()

-- Output mono (duplicated to both channels)
audioOutMono : Behavior Float -> Audio ()

-- Read audio file
readWav : String -> Audio (Behavior [Float])

-- Write audio file
writeWav : String -> Behavior [Float] -> Audio ()

-- MIDI input
midiIn : Audio (Event MidiMessage)

-- MIDI note extractor
midiNote : Channel -> Event (Note, Velocity)
```

### 8.4 Time and Tempo

```linen
-- Current time in seconds
time : Behavior Sec

-- Sample rate (constant for session)
sampleRate : Samples

-- Set BPM (affects tempo-dependent functions)
setBPM : Hz -> Audio ()

-- Current transport position
beat : Behavior Float
bar : Behavior Int
```

---

## 9. FFI (Foreign Function Interface)

**Note:** For Linen v1.0, FFI is restricted to built-in bindings. User-defined FFI is planned for post-v1.0.

### 9.1 Built-in FFI Syntax

```linen
-- Import C function
foreign "C" sin : Float -> Float
foreign "C" cos : Float -> Float
foreign "C" exp : Float -> Float

-- Import with custom name
foreign "C" "sqrtf" sqrtF32 : Float -> Float

-- Audio library bindings (built-in)
foreign "rust" readWav : String -> Audio (Behavior [Float])
```

### 9.2 FFI Restrictions

- FFI calls run in the **control thread** (not audio thread)
- No allocation in audio callback
- Automatic marshaling of primitive types
- Complex types passed by pointer (opaque handles)

---

## 10. Attributes and Metadata

### 10.1 Stable Identifiers (for Hot Reload)

```linen
-- Explicit stable ID for state preservation
#[stable("main-lfo")]
lfo = sinOsc 0.5

-- Hash-based automatic ID (default)
oscillator = sinOsc 440.0
```

### 10.2 JIT Hints

```linen
-- Force JIT compilation
#[jit]
heavyProcessing : Behavior Float -> Behavior Float

-- Inline function
#[inline]
smallFunc x = x + 1
```

### 10.3 Documentation

```linen
-- Documentation comment (appears in generated docs)
-- | Calculate ADSR envelope
-- | Attack: time to reach peak (seconds)
-- | Decay: time to fall to sustain level
-- | Sustain: level to hold (0-1)
-- | Release: time to fall to zero
adsr : Float -> Float -> Float -> Float -> ...
```

---

## 11. Complete Examples

### 11.1 Simple Sine Wave

```linen
module Main where

import Std.Audio (sinOsc, gain, audioOut)

main : Audio ()
main = sinOsc 440.0 |> gain 0.3 |> audioOutMono
```

### 11.2 FM Synthesis with Unit Types

```linen
module Main where

import Std.Audio
import Std.Time

-- FM bell sound with type-safe units
bell : Behavior Float
bell = proc trigger -> do
    -- Envelope (times in milliseconds)
    env <- adsr 10.0 300.0 0.0 2000.0 trigger never -< ()
    
    -- Modulator envelope
    modEnv <- adsr 10.0 500.0 0.0 1000.0 trigger never -< ()
    let modulator = sinOsc 220.0 * modEnv * 1000.0
    
    -- Carrier with FM (frequency in Hz)
    carrier <- sinOsc -< 440.0 + modulator
    
    returnA -< carrier * env

main : Audio ()
main = do
    trigger <- midiIn |> mapE (\_ -> ())
    audioOut (bell trigger)
```

### 11.3 Drum Machine with Mutable State

```linen
module DrumMachine where

import Std.Audio
import Std.Time
import Std.Sequencer

-- Mutable sequencer state
#[stable("sequencer-step")]
stepCounter = ref 0

-- Sequencer pattern (16 steps)
kickPattern : [Bool]
kickPattern = [true, false, false, false, true, false, false, false,
               true, false, false, false, true, false, true, false]

-- Trigger events from pattern
trigger : Event () -> Event String
trigger clock = 
    mapE (\_ -> tickAndGet) clock
    where
    | tickAndGet =
        let step = !stepCounter in
        stepCounter := (step + 1) % 16;
        if kickPattern !! step then "kick" else "rest"

main : Audio ()
main = do
    clock <- metro 120.0  -- 120 BPM
    let trig = trigger clock
    
    kick <- readWav "kick.wav"
    
    let pattern = mapE (\t -> if t == "kick" then kick else silence) trig
    
    audioOut (gain 0.7 pattern)
```

### 11.4 Live Coding Example

```linen
module Live where

import Std.Audio

-- This can be modified live - state (phase) preserved
#[stable("main-drone")]
drone = proc _ -> do
    -- Change these values live!
    let freq : Hz = 55.0       -- Try: 55, 110, 73.42
    let harmonics = 3          -- Try: 1, 3, 5, 8
    let detune : Hz = 0.5      -- Try: 0.0 to 2.0
    
    -- Additive synthesis
    osc1 <- sinOsc -< freq
    osc2 <- sinOsc -< freq * 2.0 + detune
    osc3 <- sinOsc -< freq * 3.0 - detune
    
    let sum = case harmonics of
        | 1 -> osc1
        | 2 -> osc1 + osc2 * 0.5
        | _ -> osc1 + osc2 * 0.5 + osc3 * 0.33
    
    -- Filter sweep
    lfo <- sinOsc -< 0.1
    filtered <- lpf -< (2000.0 + lfo * 1000.0, 2.0, sum)
    
    returnA -< filtered * 0.3

main : Audio ()
main = audioOut (drone ())
```

### 11.5 Using Linear Types

```linen
module LinearExample where

import Std.Audio
import Std.IO

-- Process audio buffer (linear resource)
processBuffer : lin AudioBuffer -> Audio Float
processBuffer buf = do
    -- Process the buffer
    let rms = calculateRMS buf
    -- Explicitly free (or use 'clone' to keep)
    freeBuffer buf;
    pure rms

-- Safe resource usage
analyzeFile : String -> Audio Float
analyzeFile path = do
    buf <- readFileLinear path    -- Returns lin AudioBuffer
    processBuffer buf             -- Consumes the buffer
    -- 'buf' is no longer accessible here (linear type safety)
```

---

## Appendix A: Syntax Quick Reference

### Expressions

```linen
-- Literals
42, 3.14, "hello", true, ()

-- Variables and application
x, f x, f x y

-- Lambda
\x -> x + 1
\x y -> x + y

-- Let
let x = 5 in x + 1
let x = 5 in let y = 3 in x + y

-- If
if x then y else z

-- Match
match x with | A -> 1 | B -> 2

-- Pipe
x |> f |> g
f <| x

-- Ref
ref 0, !counter, counter := 5
```

### Types

```linen
-- Basic
Int, Float, Bool, String, ()

-- Units
Hz, Ms, Sec, Db

-- Functions
Int -> Int, a -> b -> c

-- Constructors
type Maybe a = Nothing | Just a

-- Linear
lin AudioBuffer

-- FRP
Behavior Float, Event (), SF a b
```

### FRP

```linen
-- Behavior
sinOsc 440.0, time, integral 1.0

-- Event
never, merge e1 e2, mapE f e

-- Arrow
arr f, sf1 >>> sf2, proc x -> do ...
```

### Modules

```linen
module Name where
module Name (export1, export2) where
import Module
import Module (name)
import Module as Alias
import Module open
```

---

## Appendix B: Reserved Future Syntax

The following are reserved for future versions:

```linen
-- Type classes (planned)
class Eq a where
    (==) : a -> a -> Bool

instance Eq Int where
    x == y = intEq x y

-- Macros (planned)
macro! { ... }

-- Row polymorphism (planned)
type Rec r = (x : Int | r)

-- Effects tracking (planned)
process : FilePath -> IO + Audio Float
```

---

## Appendix C: Operator Precedence

From highest to lowest:

1. Function application (left): `f x y` = `((f x) y)`
2. Field access: `.` (left)
3. Dereference: `!` (prefix)
4. Multiplicative: `*`, `/`, `%` (left)
5. Additive: `+`, `-` (left)
6. Pipe: `|>` (left)
7. Comparison: `<`, `<=`, `>`, `>=` (non-associative)
8. Equality: `==`, `!=` (non-associative)
9. Logical AND: `&&` (right)
10. Logical OR: `||` (right)
11. Arrow application: `-<` (in proc)
12. Arrow composition: `>>>` (right)
13. Comma (tuple): `,` (right)
14. Type annotation: `:` (non-associative)
15. Function arrow: `->` (right)
16. Assignment: `=` (non-associative)
17. Let/match: lowest

---

## Appendix D: Glossary

- **Behavior**: Time-varying continuous value
- **Event**: Discrete occurrence in time
- **Signal Function (SF)**: Stateful signal transformation
- **Hot Reload**: Updating code without stopping audio
- **Linear Type**: Type ensuring single use (resource safety)
- **FRP**: Functional Reactive Programming
- **ADSR**: Attack-Decay-Sustain-Release envelope
- **Hz**: Hertz (frequency unit)
- **Ms**: Milliseconds (time unit)
- **Unit Type**: Type alias for physical units (Hz, Ms, Db)
