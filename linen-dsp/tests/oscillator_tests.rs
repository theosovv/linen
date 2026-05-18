use linen_dsp::mix::{Gain, Mix};
use linen_dsp::oscillators::{SawOsc, SinOsc, SquareOsc, TriangleOsc};
use linen_dsp::traits::{ConstNode, SignalNode, SAMPLE_RATE_44K};

// ---------- Golden reference tests ----------

#[test]
fn sinosc_matches_direct_sin() {
    let freq = 440.0;
    let sr = SAMPLE_RATE_44K;
    let mut osc = SinOsc::new(freq);

    for t in 0..100 {
        let expected = (t as f32 * freq * 2.0 * std::f32::consts::PI / sr).sin();
        let actual = osc.next_sample(t as u64, sr);
        assert!(
            (actual - expected).abs() < 1e-5,
            "sample {}: expected {}, got {}",
            t,
            expected,
            actual
        );
    }
}

#[test]
fn sinosc_periodicity() {
    // Pick a frequency that divides the sample rate exactly
    let freq = 100.0;
    let sr = SAMPLE_RATE_44K;
    let mut osc = SinOsc::new(freq);
    let period_samples = (sr / freq) as usize; // 441

    let first_period: Vec<f32> = (0..period_samples)
        .map(|t| osc.next_sample(t as u64, sr))
        .collect();

    let second_period: Vec<f32> = (period_samples..(period_samples * 2))
        .map(|t| osc.next_sample(t as u64, sr))
        .collect();

    for (a, b) in first_period.iter().zip(second_period.iter()) {
        assert!((a - b).abs() < 1e-4, "period mismatch: {} vs {}", a, b);
    }
}

#[test]
fn sinosc_fm_synthesis() {
    let carrier = 440.0;
    let modulator_freq = 5.0;
    let modulator_index = 10.0;
    let sr = SAMPLE_RATE_44K;

    let modulator = Box::new(SinOsc::new(modulator_freq));
    let mut fm_osc = SinOsc::with_modulator(Box::new(Gain::new(modulator, modulator_index)));

    // Just run a few samples and ensure it doesn't NaN or explode
    for t in 0..1000 {
        let sample = fm_osc.next_sample(t, sr);
        assert!(sample.is_finite(), "FM sample {} is not finite", sample);
        assert!(
            sample.abs() <= 1.01,
            "FM sample {} out of expected range",
            sample
        );
    }
}

// ---------- Gain ----------

#[test]
fn gain_scales_amplitude() {
    let freq = 440.0;
    let sr = SAMPLE_RATE_44K;
    let gain = 0.5;

    let mut sin = SinOsc::new(freq);
    let mut gained = Gain::new(Box::new(SinOsc::new(freq)), gain);

    for t in 0..100 {
        let raw = sin.next_sample(t, sr);
        let scaled = gained.next_sample(t, sr);
        assert!((scaled - raw * gain).abs() < 1e-5);
    }
}

#[test]
fn gain_zero_is_silent() {
    let sr = SAMPLE_RATE_44K;
    let mut silent = Gain::new(Box::new(SinOsc::new(440.0)), 0.0);

    for t in 0..100 {
        assert_eq!(silent.next_sample(t, sr), 0.0);
    }
}

// ---------- Mix ----------

#[test]
fn mix_sums_signals() {
    let freq1 = 220.0;
    let freq2 = 330.0;
    let sr = SAMPLE_RATE_44K;

    let mut osc1 = SinOsc::new(freq1);
    let mut osc2 = SinOsc::new(freq2);
    let mut mixed = Mix::new(vec![
        Box::new(SinOsc::new(freq1)),
        Box::new(SinOsc::new(freq2)),
    ]);

    for t in 0..100 {
        let expected = osc1.next_sample(t, sr) + osc2.next_sample(t, sr);
        let actual = mixed.next_sample(t, sr);
        assert!((actual - expected).abs() < 1e-5);
    }
}

#[test]
fn mix_empty_is_zero() {
    let sr = SAMPLE_RATE_44K;
    let mut mixed = Mix::new(vec![]);
    assert_eq!(mixed.next_sample(0, sr), 0.0);
}

// ---------- Other oscillators ----------

#[test]
fn sawosc_range() {
    let mut osc = SawOsc::new(100.0);
    let sr = SAMPLE_RATE_44K;
    for t in 0..1000 {
        let sample = osc.next_sample(t, sr);
        assert!(sample >= -1.01 && sample <= 1.01);
    }
}

#[test]
fn squareosc_range() {
    let mut osc = SquareOsc::new(100.0);
    let sr = SAMPLE_RATE_44K;
    for t in 0..1000 {
        let sample = osc.next_sample(t, sr);
        assert!(sample == 1.0 || sample == -1.0);
    }
}

#[test]
fn triangleosc_range() {
    let mut osc = TriangleOsc::new(100.0);
    let sr = SAMPLE_RATE_44K;
    for t in 0..1000 {
        let sample = osc.next_sample(t, sr);
        assert!(sample >= -1.01 && sample <= 1.01);
    }
}

// ---------- ConstNode ----------

#[test]
fn const_node_returns_fixed_value() {
    let mut node = ConstNode(3.14);
    assert_eq!(node.next_sample(0, 44100.0), 3.14);
    assert_eq!(node.next_sample(999, 48000.0), 3.14);
}
