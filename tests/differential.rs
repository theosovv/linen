#[path = "../test-support/reference/generator.rs"]
mod generator;
#[path = "../test-support/reference/mod.rs"]
mod reference;

use linen::bytecode::StateId;
use linen::vm::{VmConfig, VmRuntimeError};
use linen::{CompiledProgram, RenderConfig, compile, create_vm};
use reference::{ReferenceErrorKind, ReferenceInterpreter};

fn compile_program(source: &str, block_size: usize) -> CompiledProgram {
    compile(
        source,
        RenderConfig {
            block_size,
            ..RenderConfig::default()
        },
    )
    .expect("the test config is valid")
    .result
    .expect("the differential source should compile")
}

fn vm_samples(program: &CompiledProgram) -> Result<Vec<f32>, VmRuntimeError> {
    let mut vm = create_vm(program).expect("compiled bytecode must create a VM");
    let sample_count = program.typed.render.render_samples as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut scratch = vec![0.0; program.config.block_size];

    while samples.len() < sample_count {
        let length = scratch.len().min(sample_count - samples.len());
        vm.render_into(&mut scratch[..length])?;
        samples.extend_from_slice(&scratch[..length]);
    }

    Ok(samples)
}

fn reference_samples(program: &CompiledProgram) -> Vec<f32> {
    let mut reference = ReferenceInterpreter::new(&program.typed);
    (0..program.typed.render.render_samples)
        .map(|index| {
            reference
                .next_sample(index, program.config.sample_rate)
                .expect("the finite differential corpus must remain finite")
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

fn compare_at_block_size(source: &str, block_size: usize) {
    let program = compile_program(source, block_size);
    let actual = vm_samples(&program).expect("VM differential render must succeed");
    let expected = reference_samples(&program);

    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_close(*actual, *expected, index);
    }
}

#[test]
fn spec_7_5_static_corpus_matches_reference_at_every_required_block_size() {
    let corpus = [
        "let out = 0; render 1ms;",
        "let out = -sine(480) * 0.1; render 1ms;",
        include_str!("../fixtures/valid/stage-03-shared-sine.ln"),
        include_str!("../fixtures/valid/stage-03-block-partitions.ln"),
    ];

    for source in corpus {
        for block_size in [1, 3, 64, 128, 256] {
            compare_at_block_size(source, block_size);
        }
    }
}

#[test]
fn spec_7_5_generated_corpus_matches_independent_reference() {
    for seed in 0..16 {
        let source = generator::program(seed);
        compare_at_block_size(&source, 1 + seed as usize % 17);
    }
}

#[test]
fn spec_7_5_vm_preserves_reference_parity_after_naive_removal() {
    let program = compile_program(include_str!("../fixtures/valid/stage-03-shared-sine.ln"), 3);
    let actual = vm_samples(&program).expect("VM render must succeed");
    let expected = reference_samples(&program);

    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_close(*actual, *expected, index);
    }
}

#[test]
fn adr_0004_unused_binding_runtime_error_matches_reference() {
    let program = compile_program(
        include_str!("../fixtures/invalid/stage-03-unused-runtime-error.ln"),
        128,
    );
    let mut reference = ReferenceInterpreter::new(&program.typed);
    let reference_error = reference
        .next_sample(0, 48_000)
        .expect_err("the unused binding must fail in the reference");
    assert_eq!(reference_error.kind, ReferenceErrorKind::NonFiniteFrequency);
    assert_eq!(reference_error.sample_index, 0);

    let mut vm = create_vm(&program).expect("the runtime-error bytecode is structurally valid");
    let vm_error = vm
        .render_into(&mut [0.0; 1])
        .expect_err("VM roots must preserve the unused binding error");
    assert_eq!(
        vm_error,
        VmRuntimeError::NonFiniteFrequency {
            instruction: 4,
            state: StateId::new(1),
            sample_index: 0,
        }
    );
}

#[test]
fn adr_0005_vm_configuration_matches_direct_vm_for_differential_cases() {
    let program = compile_program("let out = sine(480); render 1ms;", 3);
    let mut direct = linen::vm::Vm::new(
        &program.bytecode,
        VmConfig {
            sample_rate: 48_000,
            block_size: 3,
        },
    )
    .expect("direct VM should be valid");
    let mut facade = create_vm(&program).expect("facade VM should be valid");
    let mut expected = [0.0; 3];
    let mut actual = [0.0; 3];
    direct.render_into(&mut expected).expect("direct VM works");
    facade.render_into(&mut actual).expect("facade VM works");
    assert_eq!(actual, expected);
}

#[test]
fn adr_0006_reference_and_vm_reset_repeat_the_same_sequence() {
    let program = compile_program("let out = sine(440) * 0.1; render 1ms;", 48);
    let mut reference = ReferenceInterpreter::new(&program.typed);
    let first: Vec<f32> = (0..48)
        .map(|index| {
            reference
                .next_sample(index, 48_000)
                .expect("the reset fixture is finite")
        })
        .collect();
    reference.reset();
    for (index, expected) in first.iter().enumerate() {
        let actual = reference
            .next_sample(index as u64, 48_000)
            .expect("the reset reference is finite");
        assert_eq!(actual.to_bits(), expected.to_bits(), "sample {index}");
    }

    let mut vm = create_vm(&program).expect("the reset bytecode is valid");
    let mut first_vm = [0.0; 48];
    vm.render_into(&mut first_vm)
        .expect("first VM render works");
    vm.reset();
    let mut second_vm = [0.0; 48];
    vm.render_into(&mut second_vm)
        .expect("reset VM render works");
    for (index, (actual, expected)) in second_vm.iter().zip(&first_vm).enumerate() {
        assert_eq!(actual.to_bits(), expected.to_bits(), "sample {index}");
    }
}

#[test]
fn spec_7_5_generated_function_block_pipeline_corpus_matches_reference() {
    for seed in 0..12 {
        let source = generator::function_program(seed);
        compare_at_block_size(&source, 1 + seed as usize % 17);
    }
}

#[test]
fn spec_7_5_generated_polyblep_corpus_matches_independent_reference() {
    for seed in 0..16 {
        let source = generator::stage5d_polyblep_program(seed);
        compare_at_block_size(&source, 1 + seed as usize % 17);
    }
}
