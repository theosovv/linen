#[path = "../test-support/reference/mod.rs"]
mod reference;

use linen::render::NaiveInterpreter;
use linen::sema::{
    BindingId, BuiltinId, Type, TypedBinding, TypedExpr, TypedExprKind, TypedProgram, TypedRender,
};
use linen::syntax::{BinaryOp, DurationUnit, Identifier};
use linen::{SourceMap, SourceSpan, compile};
use reference::ReferenceInterpreter;

fn program_with_frequency(frequency: f64) -> TypedProgram {
    let mut sources = SourceMap::new();
    let source = sources.add_source("reference.ln", "");
    let span = SourceSpan::empty(source, 0);
    let number = |value: f64| TypedExpr {
        kind: TypedExprKind::Number(value),
        ty: Type::Float,
        span,
    };
    let lift = |expression| TypedExpr {
        kind: TypedExprKind::LiftFloatToSig(Box::new(expression)),
        ty: Type::Sig,
        span,
    };
    let oscillator = TypedExpr {
        kind: TypedExprKind::Call {
            builtin: BuiltinId::SINE,
            callee: Identifier::new("sine", span),
            arguments: vec![lift(number(frequency))],
        },
        ty: Type::Sig,
        span,
    };
    let output = TypedExpr {
        kind: TypedExprKind::Binary {
            operator: BinaryOp::Multiply,
            left: Box::new(oscillator),
            right: Box::new(lift(number(0.1))),
        },
        ty: Type::Sig,
        span,
    };
    let id = BindingId::new(0);

    TypedProgram {
        bindings: vec![TypedBinding {
            id,
            name: Identifier::new("out", span),
            annotation: None,
            value: output,
            span,
        }],
        output: id,
        render: TypedRender {
            duration: linen::syntax::DurationLiteral {
                value: 1.0,
                unit: DurationUnit::Milliseconds,
                span,
            },
            duration_seconds: 0.001,
            render_samples: 48,
            span,
        },
        span,
    }
}

fn assert_close(actual: f32, expected: f32, sample_index: usize) {
    let tolerance = 2e-5 + 2e-5 * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "sample {sample_index}: actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

fn compile_program(source: &str) -> linen::CompiledProgram {
    compile(source, Default::default())
        .expect("the default render config is valid")
        .result
        .expect("the source should compile")
}

#[test]
fn adr_0009_reference_matches_analytic_quarter_period_and_gain() {
    let program = program_with_frequency(480.0);
    let mut reference = ReferenceInterpreter::new(&program);
    for index in 0..=100 {
        let actual = reference
            .next_sample(index, 48_000)
            .expect("the analytic reference input is finite");
        let expected = match index {
            0 | 50 | 100 => Some(0.0),
            25 => Some(0.1),
            75 => Some(-0.1),
            _ => None,
        };
        if let Some(expected) = expected {
            assert_close(actual, expected, index as usize);
        }
    }
}

#[test]
fn adr_0006_reference_reset_repeats_state() {
    let program = program_with_frequency(440.0);
    let mut reference = ReferenceInterpreter::new(&program);
    let first: Vec<f32> = (0..100)
        .map(|index| {
            reference
                .next_sample(index, 48_000)
                .expect("the reference input is finite")
        })
        .collect();

    reference.reset();
    for (index, expected) in first.into_iter().enumerate() {
        let actual = reference
            .next_sample(index as u64, 48_000)
            .expect("the repeated reference input is finite");
        assert_eq!(actual.to_bits(), expected.to_bits(), "sample {index}");
    }
}

#[test]
fn adr_0009_reference_matches_production_without_sharing_dsp() {
    let program = compile_program("let out = sine(480) * 0.1; render 1ms;");
    let mut production = NaiveInterpreter::new(&program.typed);
    let mut reference = ReferenceInterpreter::new(&program.typed);

    for index in 0..program.typed.render.render_samples {
        let actual = production
            .next_sample(index, 48_000)
            .expect("production sample should be finite");
        let expected = reference
            .next_sample(index, 48_000)
            .expect("reference sample should be finite");
        assert_close(actual, expected, index as usize);
    }
}
