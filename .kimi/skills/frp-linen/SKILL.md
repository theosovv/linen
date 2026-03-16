```yaml
---
name: frp-linen
description: Functional Reactive Programming patterns for Linen - Behaviors, Events, Signal Functions
---

# FRP in Linen

## Core Abstractions

### Behavior (Continuous Time)
```linen
-- Time-varying value
sineWave : Behavior Float
sineWave = sinOsc 440.0

-- Automatic lifting: operators work on Behaviors
modulated : Behavior Float
modulated = sineWave * (0.5 + 0.5 * lfo 5.0)
Event (Discrete Time)
linen
Copy
-- Never occurs
never : Event a

-- Periodic trigger
metro : Float -> Event ()
metro bpm = every (60.0 / bpm)

-- MIDI input as events
noteOn : Event (Note, Velocity)
Signal Function (Stateful)
linen
Copy
-- Type: SF input output
filter : SF Float Float
filter = proc input -> do
    filtered <- lowPass 1000.0 0.7 -< input
    returnA -< filtered
Combinators
Behavior
Table
Function	Type	Description
liftA2	(a→b→c) → Behavior a → Behavior b → Behavior c	Combine two Behaviors
sample	Behavior a → Event b → Event a	Sample at event times
integral	Behavior Float → Behavior Float	Time integration
derivative	Behavior Float → Behavior Float	Time derivative
Event
Table
Function	Type	Description
merge	Event a → Event a → Event a	Union (left bias)
mapE	(a→b) → Event a → Event b	Transform values
filterE	(a→Bool) → Event a → Event a	Conditional pass
hold	a → Event a → Behavior a	Event → Behavior
accum	a → Event (a→a) → Behavior a	Accumulate state
Signal Function (Arrows)
Table
Function	Type	Description
arr	(a→b) → SF a b	Lift pure function
(>>>)	SF a b → SF b c → SF a c	Sequential composition
(&&&)	SF a b → SF a c → SF a (b,c)	Parallel split
first	SF a b → SF (a,c) (b,c)	Apply to first component
loop	SF (a,c) (b,c) → SF a b	Feedback loop
Proc Notation
Syntax
linen
Copy
proc input -> do
    -- Bind: result <- sf -< input
    freq   <- arr midiToFreq  -< input
    osc    <- sinOsc          -< freq
    env    <- adsr 0.1 0.3 0.7 1.0 -< gate
    
    -- Recursive definition with 'rec'
    rec
        let delayed = delay 0.1 -< mixed
        let mixed = input + delayed * 0.5
    
    -- Return result
    returnA -< osc * env
Desugaring
linen
Copy
-- Proc notation
proc x -> do
    y <- f -< x
    z <- g -< y
    returnA -< z

-- Desugars to arrow composition
arr x -> (f >>> g)
Common Patterns
LFO with Range
linen
Copy
lfoRange : Float -> Float -> Float -> Behavior Float
lfoRange freq min max = 
    let sine = sin (2.0 * pi * freq * time)
    in min + (sine + 1.0) / 2.0 * (max - min)
Triggered Envelope
linen
Copy
triggered : Event () -> Behavior Float
triggered trig = 
    let gate = hold 0.0 (mapE (\_ -> 1.0) trig)
    in adsr 0.01 0.3 0.5 0.5 gate (mapE (\_ -> 0.0) trig)
Sample and Hold
linen
Copy
sampleAndHold : Behavior a -> Event () -> Behavior a
sampleAndHold source trigger = 
    hold 0.0 (sample source trigger)
Switching
linen
Copy
-- Switch between behaviors on event
switcher : Behavior a -> Event (Behavior a) -> Behavior a

-- Example: change waveform on beat
changingOsc : Behavior Float
changingOsc = 
    let waveforms = cycle [sinOsc 440, sawOsc 440, squareOsc 440]
    in switcher (sinOsc 440) (mapE (\_ -> head waveforms) beat)
Hot Reload with FRP
State Preservation
linen
Copy
-- #[stable] preserves state across reloads
#[stable("main-lfo")]
lfo = sinOsc 0.5

-- Phase continues from previous value after reload
Migration Rules
Table
Old State	New Code	Result
sinOsc 440	sinOsc 880	Phase preserved, freq changed
sinOsc 440	sawOsc 440	State reset (different type)
adsr 0.1 0.2 0.5 0.5	adsr 0.2 0.3 0.6 0.4	Phase/level preserved, params updated
Anti-patterns
Don't: Nested time
linen
Copy
-- BAD: time inside Behavior
bad = behavior (\t -> behavior (\t' -> t + t'))
-- Result: time-of-time, confusing semantics
Don't: Unbounded recursion in proc
linen
Copy
-- BAD: infinite recursion without delay
bad = proc x -> do
    rec y <- bad -< x  -- Stack overflow!
    returnA -< y
Do: Use delay for feedback
linen
Copy
-- GOOD: explicit delay
good = proc x -> do
    rec y <- delay 0.001 -< x + y * 0.5
    returnA -< y
Resources
"Functional Reactive Programming" by Blackheath and Jones
Yampa paper: "Functional Reactive Programming, Continued"
Elm documentation on Signals (historical)
