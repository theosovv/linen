use linen::dsp::Sine;
use linen::render::NaiveInterpreter;
use linen::{
    CompiledProgram, ErrorCode, RenderError, SourceId, SourceMap, SourceSpan, compile, render,
};

fn compile_program(source: &str) -> CompiledProgram {
    compile(source, Default::default())
        .expect("the default render config is valid")
        .result
        .expect("the source should compile")
}

fn collect(program: &CompiledProgram, count: usize) -> Vec<f32> {
    let mut interpreter = NaiveInterpreter::new(&program.typed);
    (0..count)
        .map(|index| {
            interpreter
                .next_sample(index as u64, program.config.sample_rate)
                .expect("the analytic input should produce finite samples")
        })
        .collect()
}

fn assert_close(actual: f32, expected: f32, sample_index: usize) {
    let tolerance = 2e-5 + 2e-5 * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "sample {sample_index}: actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

fn source_id() -> SourceId {
    let mut sources = SourceMap::new();
    sources.add_source("expected.ln", "")
}

#[test]
fn spec_4_8_sine_matches_analytic_quarter_period_and_gain() {
    let program = compile_program("let out = sine(480) * 0.1; render 3ms;");
    let samples = collect(&program, 101);

    for (index, expected) in [(0, 0.0), (25, 0.1), (50, 0.0), (75, -0.1), (100, 0.0)] {
        assert_close(samples[index], expected, index);
    }
}

#[test]
fn spec_4_8_zero_negative_and_above_nyquist_frequencies_follow_phase_formula() {
    let zero = compile_program("let out = sine(0); render 1ms;");
    assert!(collect(&zero, 16).iter().all(|sample| *sample == 0.0));

    let negative = compile_program("let out = sine(-480); render 1ms;");
    let negative_samples = collect(&negative, 26);
    assert_close(negative_samples[25], -1.0, 25);

    let aliased = compile_program("let out = sine(96000); render 1ms;");
    assert!(
        collect(&aliased, 16)
            .iter()
            .all(|sample| sample.abs() <= 2e-5)
    );
}

#[test]
fn spec_4_8_reset_repeats_the_same_samples() {
    let program = compile_program("let out = sine(440) * 0.1; render 2ms;");
    let mut interpreter = NaiveInterpreter::new(&program.typed);
    let first: Vec<f32> = (0..96)
        .map(|index| {
            interpreter
                .next_sample(index, 48_000)
                .expect("the first render should be finite")
        })
        .collect();

    interpreter.reset();
    for (index, expected) in first.into_iter().enumerate() {
        let actual = interpreter
            .next_sample(index as u64, 48_000)
            .expect("the reset render should be finite");
        assert_close(actual, expected, index);
    }
}

#[test]
fn spec_4_8_phase_stays_finite_and_wrapped_on_long_render() {
    let mut sine = Sine::new();
    for _ in 0..1_000_000 {
        sine.next_sample(48_000.0, 48_000)
            .expect("an integral number of periods remains finite");
    }

    assert!(sine.phase().is_finite());
    assert!((0.0..1.0).contains(&sine.phase()));
}

#[test]
fn spec_4_8_negative_frequency_uses_euclidean_modulo() {
    let mut sine = Sine::new();
    assert_eq!(
        sine.next_sample(-480.0, 48_000)
            .expect("negative finite frequency is valid"),
        0.0
    );
    assert_close(sine.phase() as f32, 0.99, 0);
}

#[test]
fn spec_6_2_nonfinite_output_has_exact_span_and_sample_index() {
    let source = "let out = sine(1) / 0; render 1ms;";
    let program = compile_program(source);
    let error = render(&program, &mut Vec::new()).expect_err("NaN output must fail");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("a nonfinite output must be a source diagnostic");
    };

    assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
    assert_eq!(diagnostic.sample_index(), Some(0));
    assert_eq!(
        diagnostic.message(),
        "output signal contains NaN or infinity"
    );
    assert_eq!(
        diagnostic.primary().message(),
        "this out expression produces the invalid sample"
    );
    assert_eq!(
        diagnostic.primary().span(),
        SourceSpan::new(source_id(), 10, 21).expect("the expected span is valid")
    );
}

#[test]
fn spec_6_2_nonfinite_sine_frequency_has_argument_span_and_sample_index() {
    let source = "let out = sine(sine(0) / 0); render 1ms;";
    let program = compile_program(source);
    let mut wav = Vec::new();
    let error = render(&program, &mut wav).expect_err("NaN frequency must fail");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("a nonfinite frequency must be a source diagnostic");
    };

    assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
    assert_eq!(diagnostic.sample_index(), Some(0));
    assert_eq!(
        diagnostic.message(),
        "sine frequency contains NaN or infinity"
    );
    assert_eq!(
        diagnostic.primary().message(),
        "this argument produces the invalid frequency"
    );
    assert_eq!(
        diagnostic.primary().span(),
        SourceSpan::new(source_id(), 15, 26).expect("the expected span is valid")
    );
}

#[test]
fn spec_6_2_runtime_error_uses_the_actual_failing_sample_index() {
    let source = "let out = sine(0) / (sine(480) + 1); render 2ms;";
    let program = compile_program(source);
    let error = render(&program, &mut Vec::new()).expect_err("division must fail at sample 75");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("a nonfinite output must be a source diagnostic");
    };

    assert_eq!(diagnostic.sample_index(), Some(75));
    assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
    let span = diagnostic.primary().span();
    assert_eq!(
        (span.source().index(), span.start(), span.end()),
        (0, 10, 35)
    );
}

#[test]
fn spec_4_8_nonfinite_frequency_does_not_change_sine_state() {
    let mut sine = Sine::new();
    sine.next_sample(480.0, 48_000)
        .expect("the finite setup sample should succeed");
    let phase = sine.phase();

    for frequency in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        sine.next_sample(frequency, 48_000)
            .expect_err("nonfinite frequency must be rejected");
        assert_eq!(sine.phase().to_bits(), phase.to_bits());
    }
}
