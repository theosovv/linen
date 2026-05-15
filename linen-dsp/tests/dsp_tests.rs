use linen_dsp::oscillators::SinOsc;
use linen_dsp::traits::SignalNode;

#[test]
fn sine_wave_zero_crossings() {
    let mut osc = SinOsc::new(440.0);
    let sr = 44100.0;

    let samples: Vec<f32> = (0..441).map(|t| osc.next_sample(t, sr)).collect();

    let zero_crossings = samples.windows(2).filter(|w| w[0] * w[1] < 0.0).count();

    // 440Hz at 44.1kHz: period = 100.23 samples
    // In 441 samples we expect ~8 zero crossings (2 per period, ~4.4 periods)
    assert!(zero_crossings >= 7 && zero_crossings <= 10);
}

#[test]
fn sine_wave_range() {
    let mut osc = SinOsc::new(220.0);
    let sr = 48000.0;

    for t in 0..1000 {
        let sample = osc.next_sample(t, sr);
        assert!(sample >= -1.01 && sample <= 1.01);
    }
}
