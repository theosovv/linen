yaml
Copy
---
name: rust-audio
description: Real-time audio programming in Rust with CPAL, lock-free data structures, and DSP
---

# Rust Audio Programming for Linen

## Core Principles

### Real-time Safety (Audio Thread)
- **NO allocation**: Pre-allocate all buffers at init
- **NO locks**: Use lock-free structures only (ringbuf, crossbeam)
- **NO blocking**: No I/O, no sleep, no mutex
- **Bounded execution**: Fixed iteration count, no recursion

### Thread Architecture
┌─────────────────┐     lock-free      ┌─────────────────┐
│  Control Thread │  ←──────────────→  │   Audio Thread  │
│  (VM, GC, I/O)  │   ring buffer      │  (DSP, no GC)   │
└─────────────────┘                    └─────────────────┘
plain
Copy

## CPAL Setup

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Default output device
let host = cpal::default_host();
let device = host.default_output_device()
    .expect("no output device");

// Config: 48kHz, 256 samples (~5ms latency)
let config = cpal::StreamConfig {
    channels: 2,
    sample_rate: cpal::SampleRate(48000),
    buffer_size: cpal::BufferSize::Fixed(256),
};

// Build stream with callback
let stream = device.build_output_stream(
    &config,
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        // REAL-TIME CALLBACK: no alloc, no locks
        for sample in data {
            *sample = generate_sample();
        }
    },
    |err| eprintln!("stream error: {}", err),
    None,
).unwrap();

stream.play().unwrap();
Lock-free Communication
Ring Buffer (Control → Audio)
rust
Copy
use ringbuf::{HeapRb, Producer, Consumer};

// Create SPSC queue
let (mut prod, mut cons) = HeapRb::<f32>::new(1024).split();

// Control thread: non-blocking push
if prod.push(value).is_err() {
    // Queue full, drop or handle
}

// Audio thread: non-blocking pop
while let Some(value) = cons.pop() {
    // Process value
}
Crossbeam Channels (Audio → Control)
rust
Copy
use crossbeam::channel::{bounded, Sender, Receiver};

// Metering data from audio to UI
let (tx, rx) = bounded::<MeterData>(16);

// Audio thread (non-blocking)
tx.try_send(MeterData { level: 0.5 }).ok();

// Control thread (blocking OK here)
let data = rx.recv().unwrap();
DSP Primitives
Oscillator (Phase Accumulator)
rust
Copy
pub struct SineOsc {
    phase: f32,      // [0, 1)
    phase_inc: f32,  // frequency / sample_rate
}

impl SineOsc {
    #[inline(always)]
    pub fn next(&mut self) -> f32 {
        let output = (self.phase * 2.0 * PI).sin();
        self.phase += self.phase_inc;
        if self.phase >= 1.0 { self.phase -= 1.0; }
        output
    }
    
    pub fn set_freq(&mut self, freq: f32, sr: f32) {
        self.phase_inc = freq / sr;
    }
}
Biquad Filter (Direct Form 2 Transposed)
rust
Copy
pub struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    z1: f32, z2: f32,
}

impl Biquad {
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = input * self.b0 + self.z1;
        self.z1 = input * self.b1 + self.z2 - self.a1 * output;
        self.z2 = input * self.b2 - self.a2 * output;
        output
    }
}
System Configuration (Linux)
bash
Copy
# Real-time priority
sudo chrt -f -p 80 $(pgrep linen)

# Lock memory
ulimit -l unlimited

# CPU isolation (boot param)
isolcpus=2,3  # Reserve cores 2,3 for audio

# JACK low-latency
jackd -d alsa -r 48000 -p 256 -n 2
Common Patterns
Table
Pattern	Structure	Use Case
Command queue	HeapRb<Command>	Control → Audio (parameters)
State snapshot	Arc<AtomicF32>	Shared atomic state
Buffer pool	Vec<AudioBuffer>	Reusable buffers
Lock-free list	crossbeam::segqueue	Multiple producers
Testing
rust
Copy
#[test]
fn test_oscillator() {
    let mut osc = SineOsc::new(440.0, 48000.0);
    let s1 = osc.next();
    let s2 = osc.next();
    assert!(s1 != s2); // Phase advances
    assert!(s1.abs() <= 1.0);
}

#[test]
fn test_biquad_dc() {
    let mut filter = Biquad::lowpass(1000.0, 48000.0, 0.7);
    // DC should pass through after settling
    let mut output = 0.0;
    for _ in 0..1000 {
        output = filter.process(1.0);
    }
    assert!(output > 0.99);
}
Resources
CPAL docs: https://docs.rs/cpal/
Rust Audio Discord: https://discord.gg/rust-audio
"Real-Time Audio Programming" by Ross Bencina