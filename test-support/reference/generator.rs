#![allow(dead_code)]

pub fn program(seed: u64) -> String {
    let mut rng = FixedRng::new(seed);
    let constant = float_expr(&mut rng, 2);
    let output = signal_expr(&mut rng, 3);

    format!(
        "let constant = {constant};\nlet wave = sine(constant);\nlet out = {output};\nrender 1ms;\n"
    )
}

struct FixedRng(u64);

impl FixedRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, upper: u64) -> u64 {
        self.next() % upper
    }
}

fn float_expr(rng: &mut FixedRng, depth: usize) -> String {
    if depth == 0 || rng.below(3) == 0 {
        return (rng.below(17) as i64 - 8).to_string();
    }

    let left = float_expr(rng, depth - 1);
    let right = float_expr(rng, depth - 1);
    match rng.below(4) {
        0 => format!("({left} + {right})"),
        1 => format!("({left} - {right})"),
        2 => format!("({left} * {right})"),
        _ => format!("({left} / {})", rng.below(4) + 1),
    }
}

fn signal_expr(rng: &mut FixedRng, depth: usize) -> String {
    if depth == 0 || rng.below(4) == 0 {
        return match rng.below(3) {
            0 => "wave".into(),
            1 => "sine(constant)".into(),
            _ => format!("sine({})", float_expr(rng, 1)),
        };
    }

    let left = signal_expr(rng, depth - 1);
    let right = signal_expr(rng, depth - 1);
    match rng.below(3) {
        0 => format!("({left} + {right})"),
        1 => format!("({left} - {right})"),
        _ => format!("({left} * {right})"),
    }
}

pub fn function_program(seed: u64) -> String {
    let mut rng = FixedRng::new(seed);
    let base = 120 + rng.below(9) * 20;
    let frequency = 240 + rng.below(9) * 30;
    let gain = 1 + rng.below(4);
    let calls = 1 + rng.below(4) as usize;
    let mut value = format!("sine({frequency})");
    for index in 0..calls {
        value = format!("{value} |> helper({})", index % 3 + 1);
    }

    format!(
        "let base = {base};\nlet helper = (x: Sig, gain: Sig): Sig => {{\n  let captured = sine(base);\n  let mixed = x * gain + captured * 0.25;\n  {{\n    let local = mixed * 0.5;\n    local\n  }}\n}};\nlet initial_gain = {gain};\nlet first = helper(sine(base), initial_gain);\nlet out = first + {value};\nrender 1ms;\n"
    )
}

pub fn stage5_mono_program(seed: u64) -> String {
    let mut rng = FixedRng::new(seed);
    let frequency = 80 + rng.below(9) * 20;
    let balance = rng.below(5);
    let waveform = match rng.below(4) {
        0 => format!("phasor({frequency})"),
        1 => format!("saw({frequency})"),
        2 => format!("square({frequency})"),
        _ => format!("triangle({frequency})"),
    };
    let second = match rng.below(3) {
        0 => format!("saw({frequency})"),
        1 => format!("triangle({frequency})"),
        _ => format!("phasor({frequency})"),
    };

    format!(
        "let mixed = mix({waveform}, {second}, 0.{balance});\nlet out = gain(mixed, 0.1);\nrender 1ms;\n"
    )
}

pub fn stage5_stereo_program(seed: u64) -> String {
    let mut rng = FixedRng::new(seed);
    let frequency = 120 + rng.below(8) * 30;
    let position = rng.below(5);
    format!(
        "let wave = mix(saw({frequency}), triangle({frequency}), 0.25);\nlet out = pan(gain(wave, 0.1), -0.{position});\nrender 1ms;\n"
    )
}

pub fn stage5_noise_program(seed: u64) -> String {
    format!(
        "let left = noise({seed});\nlet right = noise({});\nlet out = stereo(gain(left, 0.1), gain(right, 0.1));\nrender 1ms;\n",
        seed + 1
    )
}
