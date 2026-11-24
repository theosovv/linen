#![allow(dead_code)]

use linen::sema::{
    BindingId, BuiltinId, Type, TypedBinding, TypedExpr, TypedExprKind, TypedProgram, TypedRender,
    TypedTopDecl,
};
use linen::syntax::{DurationLiteral, DurationUnit, Identifier};
use linen::{CompiledProgram, RenderConfig, SourceMap, SourceSpan, compile, create_vm};

pub fn compile_program(source: &str, block_size: usize) -> CompiledProgram {
    compile(
        source,
        RenderConfig {
            block_size,
            ..RenderConfig::default()
        },
    )
    .expect("the test render config is valid")
    .result
    .expect("the stage 5 source should compile")
}

pub fn span() -> SourceSpan {
    let mut sources = SourceMap::new();
    let source = sources.add_source("stage-05-test.ln", "");
    SourceSpan::empty(source, 0)
}

pub fn number(value: f64) -> TypedExpr {
    TypedExpr {
        kind: TypedExprKind::Number(value),
        ty: Type::Float,
        span: span(),
    }
}

pub fn lift(value: TypedExpr) -> TypedExpr {
    TypedExpr {
        kind: TypedExprKind::LiftFloatToSig(Box::new(value)),
        ty: Type::Sig,
        span: span(),
    }
}

pub fn builtin(builtin: BuiltinId, arguments: Vec<TypedExpr>, ty: Type) -> TypedExpr {
    let name = linen::sema::builtin_descriptor(builtin)
        .expect("the test builtin must exist")
        .name;
    TypedExpr {
        kind: TypedExprKind::BuiltinCall {
            builtin,
            callee: Identifier::new(name, span()),
            arguments,
        },
        ty,
        span: span(),
    }
}

pub fn typed_program(value: TypedExpr, frames: u64) -> TypedProgram {
    let id = BindingId::new(0);
    TypedProgram {
        declarations: vec![TypedTopDecl::Binding(TypedBinding {
            id,
            name: Identifier::new("out", span()),
            annotation: None,
            value,
            span: span(),
        })],
        output: id,
        render: TypedRender {
            duration: DurationLiteral {
                value: 1.0,
                unit: DurationUnit::Milliseconds,
                span: span(),
            },
            duration_seconds: 0.001,
            render_samples: frames,
            span: span(),
        },
        span: span(),
    }
}

pub fn assert_close(actual: f32, expected: f32, sample_index: usize) {
    let tolerance = 2e-5 + 2e-5 * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "sample {sample_index}: actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

pub fn assert_primary(source: &str, diagnostic: &linen::Diagnostic, needle: &str) {
    let start = source
        .find(needle)
        .expect("the expected primary text exists");
    assert_eq!(
        (
            diagnostic.primary().span().start(),
            diagnostic.primary().span().end()
        ),
        (start, start + needle.len())
    );
}

pub fn vm_values(program: &CompiledProgram) -> Vec<f32> {
    let channels = program.bytecode.output.channels();
    let frames = program.typed.render.render_samples as usize;
    let target_values = frames * channels;
    let mut vm = create_vm(program).expect("verified bytecode must create a VM");
    let mut values = Vec::with_capacity(target_values);
    let mut scratch = vec![0.0; program.config.block_size * channels];

    while values.len() < target_values {
        let remaining_frames = (target_values - values.len()) / channels;
        let frames = remaining_frames.min(program.config.block_size);
        let length = frames * channels;
        vm.render_into(&mut scratch[..length])
            .expect("the stage 5 render must succeed");
        values.extend_from_slice(&scratch[..length]);
    }

    values
}

pub fn assert_reset_repeats(program: &CompiledProgram, expected: &[f32]) {
    let channels = program.bytecode.output.channels();
    let block_values = program.config.block_size * channels;
    let mut vm = create_vm(program).expect("verified bytecode must create a VM");
    let mut first = vec![0.0; expected.len()];
    for block in first.chunks_mut(block_values) {
        vm.render_into(block)
            .expect("the first render must succeed");
    }
    assert_eq!(first, expected);

    vm.reset();

    let mut second = vec![0.0; expected.len()];
    for block in second.chunks_mut(block_values) {
        vm.render_into(block)
            .expect("the render after reset must succeed");
    }
    assert_eq!(second, expected);
}
