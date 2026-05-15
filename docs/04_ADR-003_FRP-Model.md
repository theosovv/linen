# ADR-003: FRP Model for Linen

## Status
**Accepted**

## Context
Linen — FRP language for audio. Need to choose specific reactive programming model that:
1. Naturally expresses audio signals (continuous time)
2. Supports discrete events (note on/off, pattern triggers)
3. Compiles to efficient code without memory leaks
4. Statically typed

## Decision
**Signal-oriented FRP (Signal-Event-Behavior) with explicit time.**

### Core Primitives

#### 1. Signal<T> — Continuous Time
```linen
type Signal a = Time -> a
```
- Signal — pure function of time
- Compiles to DSP graph: each `Signal` — node computing `sample(t)`
- No implicit sampling rate — runtime determines SR at bounce

#### 2. Event<T> — Discrete Stream
```linen
type Event a = [(Time, a)]  -- conceptually; implementation: callback-based
```
- Events happen at specific times
- Sources: `every`, `random`, MIDI-in (future), UI events (future)
- No history — fire-and-forget

#### 3. Behavior<T> — Time-Varying Value
```linen
type Behavior a = Event a -> Signal a  -- hold/switch semantics
```
- Behavior created from Event via `hold` (sample-and-hold) or `switch` (signal switching)
- Used for parameters changing on events: `cutoff <- every 1.0 |> map (\_ -> random 200.0 2000.0)`

### Combinators

#### Signal Combinators
```linen
map      : (a -> b) -> Signal a -> Signal b
zipWith  : (a -> b -> c) -> Signal a -> Signal b -> Signal c
filter   : (a -> Bool) -> Signal a -> Signal a  -- drop samples (rarely used in audio)
sample   : Event Unit -> Signal a -> Event a      -- sample signal at event times
```

#### Event Combinators
```linen
map      : (a -> b) -> Event a -> Event b
filter   : (a -> Bool) -> Event a -> Event a
merge    : Event a -> Event a -> Event a
zip      : Event a -> Event b -> Event (a, b)
hold     : a -> Event a -> Signal a               -- sample-and-hold
switch   : Signal a -> Event (Signal a) -> Signal a  -- switch to new signal on event
```

#### Temporal Combinators
```linen
every    : Seconds -> Event Unit                  -- metronome
delay    : Seconds -> Event a -> Event a          -- delay events
throttle : Seconds -> Event a -> Event a          -- limit rate
cycle    : [a] -> Event a                         -- cyclic pattern
random   : Event a                                -- seeded random (deterministic)
```

### Semantics

#### Evaluation Model
- **Offline bounce**: Signal graph computed sample-by-sample for given duration
- **No push/pull distinction**: All signals pull-based (request sample from inputs)
- **Lazy evaluation**: Nodes computed only if output needed
- **Memoization**: Within one sample-frame node value cached

#### Time Representation
```linen
type Time = Samples   -- exact integer sample count
type Seconds = Float  -- converted to Samples at runtime based on SR
```
- Inside runtime time is always `Samples` (u64)
- `Seconds` converted at signal creation (compile-time constant or runtime parameter)

### Memory Model
- **No GC in audio thread**: All buffers pre-allocated
- **Signal graph — DAG**: Compiler checks absence of cycles (feedback forbidden at type level?)
- **Feedback**: Through explicit `delay` combinator: `delay (samples 1) signal` — creates feedback loop with delay

## Consequences

### Positive
- Pure semantics: `Signal` — function, easy to test
- Compiler can optimize entire graph: constant folding, dead code elimination
- Predictable memory: no surprise allocations

### Negative
- Feedback only through `delay` — limits some DSP patches
- `switch` can cause click if switching not at zero-crossing — requires care
- No implicit history — user must explicitly use `hold`/`delay`

## Alternatives Considered

### A. Arrowized FRP (Yampa)
- **Pros**: Feedback through arrows, type-safe signal functions
- **Cons**: Harder syntax, harder to optimize globally
- **Verdict**: Possible in v0.2 for feedback-heavy patches

### B. Push-Pull FRP (Reactive-banana)
- **Pros**: Efficient sparse event handling
- **Cons**: Harder implementation, not needed for offline bounce
- **Verdict**: Rejected — pull-based sufficient

### C. Temporal Logic (TidalCycles approach)
- **Pros**: Ideal for rhythms and patterns
- **Cons**: Not natural for continuous synthesis
- **Verdict**: Possible as DSL inside Event combinators

## Implementation

### Runtime Representation
```rust
// Pseudo-code for DSP runtime
trait SignalNode<T> {
    fn next_sample(&mut self, t: SampleTime) -> T;
}

struct SinOsc {
    freq: Box<dyn SignalNode<f32>>,
    phase: f32,
}

impl SignalNode<f32> for SinOsc {
    fn next_sample(&mut self, t: SampleTime) -> f32 {
        let f = self.freq.next_sample(t);
        self.phase += f * 2.0 * PI / SAMPLE_RATE;
        self.phase.sin()
    }
}
```

### Compilation Strategy
- `Signal<T>` -> `Box<dyn SignalNode<T>>` or static dispatch via generics
- `Event<T>` -> callback registration in event scheduler
- `Behavior<T>` -> `SignalNode` with mutable state (last value)

## References
- [Functional Reactive Programming](https://wiki.haskell.org/Functional_Reactive_Programming)
- [Yampa](https://wiki.haskell.org/Yampa) — Arrowized FRP
- [Reactive-banana](https://wiki.haskell.org/Reactive-banana) — Push-pull FRP
- [TidalCycles](https://tidalcycles.org) — Pattern FRP
