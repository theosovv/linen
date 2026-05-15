# Linen — Example Programs

## Example 1: Hello Sine (Minimal)
```linen
-- hello_sine.ln
-- The simplest Linen program

let tone = sin (hz 440.0)
export tone "hello_sine.wav" (seconds 2.0)
```

**Expected output:** 2-second 440Hz sine wave, -6dBFS amplitude.

---

## Example 2: Drone with Filter
```linen
-- drone.ln
-- Evolving drone with LFO-modulated filter

let baseFreq = hz 110.0
let lfoFreq = hz 0.25

let lfo = sin lfoFreq |> map (x -> x * 0.5 + 0.5)  -- unipolar 0..1

let drone = 
  sin baseFreq 
  |> mix (sin (hz 112.0) |> gain 0.3)  -- slight detune for beating
  |> filter (lowpass (lfo |> map (x -> x * 800.0 + 200.0)) 0.7)
  |> gain 0.4

export drone "drone.wav" (seconds 10.0)
```

---

## Example 3: Algorithmic Melody
```linen
-- melody.ln
-- Random melody from pentatonic scale

let pentatonic = [261.6, 293.7, 329.6, 392.0, 440.0, 523.2]

let noteEvent = every (seconds 0.25)
let randomNote = noteEvent |> map (\_ -> randomChoice pentatonic)

let melody = 
  randomNote 
  |> map (freq -> sin (hz freq) |> adsr 0.05 0.1 0.3 0.2)
  |> mixEvents
  |> gain 0.5

export melody "melody.wav" (seconds 8.0)
```

---

## Example 4: Polyphonic Synth
```linen
-- polysynth.ln
-- 4-voice polyphony with detune

let voice : Hz -> Signal Float = fn freq ->
  let detune = random (hz (-2.0)) (hz 2.0)
  let osc = sin (freq + detune) |> gain 0.25
  let filtered = osc |> filter (lowpass (hz 2000.0) 0.5)
  filtered |> adsr 0.1 0.2 0.6 0.3

let chord = [hz 261.6, hz 329.6, hz 392.0, hz 523.2]
let voices = chord |> map voice |> sequenceA |> map mix

export voices "polysynth.wav" (seconds 4.0)
```

---

## Example 5: Rhythmic Pattern
```linen
-- rhythm.ln
-- Kick + snare pattern with hihats

let kick = 
  every (seconds 0.5) 
  |> map (\_ -> sin (hz 60.0) |> adsr 0.01 0.1 0.0 0.2 |> gain 0.8)

let snare = 
  every (seconds 1.0) 
  |> offset (seconds 0.5)
  |> map (\_ -> noise White |> filter (bandpass (hz 1000.0) (hz 200.0)) |> adsr 0.01 0.1 0.0 0.15 |> gain 0.6)

let hihat = 
  every (seconds 0.25)
  |> map (\_ -> noise White |> filter (highpass (hz 8000.0)) |> adsr 0.005 0.02 0.0 0.05 |> gain 0.3)

let beat = [kick, snare, hihat] |> sequenceA |> map mix

export beat "rhythm.wav" (seconds 4.0)
```

---

## Example 6: FM Synthesis
```linen
-- fm.ln
-- Classic FM synthesis (DX7-style)

let carrierFreq = hz 440.0
let modulatorFreq = hz 220.0
let modulationIndex = 5.0

let modulator = sin modulatorFreq |> map (x -> x * modulationIndex)
let fm = sin (carrierFreq + modulator)

let patch = fm |> filter (lowpass (hz 4000.0) 0.3) |> adsr 0.1 0.3 0.5 0.4 |> gain 0.5

export patch "fm_bell.wav" (seconds 3.0)
```

---

## Example 7: Modular Import
```linen
-- main.ln
-- Using imported modules

import "std/oscillators" as Osc
import "std/filters" as Filter
import "std/envelopes" as Env
import "std/effects" as Fx

let bass = Osc.saw (hz 55.0) |> Filter.lowpass (hz 400.0) 0.8 |> Env.adsr 0.05 0.1 0.7 0.2 |> gain 0.6

let pad = 
  [Osc.sin (hz 220.0), Osc.sin (hz 277.2), Osc.sin (hz 329.6)]
  |> sequenceA
  |> map mix
  |> Fx.chorus 0.5 0.3
  |> Fx.reverb 0.4 3.0
  |> gain 0.3

let track = [bass, pad] |> sequenceA |> map mix

export track "track.wav" (seconds 16.0)
```

---

*Note: Examples with random/delay use features planned for v0.2*

*Version: 0.1*  
*Date: 2026-05-15*
