use linen::bytecode::{
    AudioRegister, BinaryOp, Bytecode, BytecodeVerifyError, Instruction, Operand, OutputLayout,
    ScalarRegister, StateId,
};
use linen::vm::{Vm, VmConfig, VmConfigError, VmCreateError, VmRuntimeError};
use linen::{RenderConfig, compile, create_vm};

fn bytecode(
    instructions: Vec<Instruction>,
    scalar_register_count: usize,
    audio_register_count: usize,
    state_count: usize,
    output: usize,
) -> Bytecode {
    Bytecode {
        instructions,
        scalar_register_count,
        audio_register_count,
        state_count,
        output: OutputLayout::Mono(AudioRegister::new(output)),
        debug_origins: None,
    }
}

fn constant_program(value: f64) -> Bytecode {
    bytecode(
        vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(0),
            },
        ],
        1,
        2,
        0,
        1,
    )
}

fn sine_program(frequency: f64, scalar_input: bool) -> Bytecode {
    let mut instructions = vec![Instruction::ConstScalar {
        dst: ScalarRegister::new(0),
        value: frequency,
    }];
    let (frequency_operand, output_source) = if scalar_input {
        (Operand::Scalar(ScalarRegister::new(0)), 0)
    } else {
        instructions.push(Instruction::Lift {
            dst: AudioRegister::new(0),
            src: ScalarRegister::new(0),
        });
        (Operand::Audio(AudioRegister::new(0)), 1)
    };
    instructions.push(Instruction::Sine {
        dst: AudioRegister::new(output_source),
        freq: frequency_operand,
        state: StateId::new(0),
    });
    instructions.push(Instruction::SetOutput {
        src: AudioRegister::new(output_source),
    });

    bytecode(instructions, 1, output_source + 2, 1, output_source + 1)
}

fn render_with_partitions(
    bytecode: &Bytecode,
    maximum_block: usize,
    samples: usize,
    partitions: &[usize],
) -> Result<Vec<f32>, VmRuntimeError> {
    let mut vm = Vm::new(
        bytecode,
        VmConfig {
            sample_rate: 48_000,
            block_size: maximum_block,
        },
    )
    .expect("the manual bytecode and config should be valid");
    let mut rendered = Vec::with_capacity(samples);
    let mut block_index = 0;

    while rendered.len() < samples {
        let requested = partitions[block_index % partitions.len()];
        let length = requested.min(samples - rendered.len());
        let mut block = vec![0.0; length];
        vm.render_into(&mut block)?;
        rendered.extend_from_slice(&block);
        block_index += 1;
    }

    Ok(rendered)
}

fn assert_close(actual: f32, expected: f32, sample_index: usize) {
    let tolerance = 2e-5 + 2e-5 * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "sample {sample_index}: actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

fn independent_sine(frequency: f32, sample_index: usize) -> f32 {
    let periods = f64::from(frequency) * sample_index as f64 / 48_000.0;
    (std::f64::consts::TAU * periods.rem_euclid(1.0)).sin() as f32
}

#[test]
fn spec_6_4_vm_config_validates_sample_rate_and_maximum_block_size() {
    let default = VmConfig::default();
    assert_eq!(default.sample_rate, 48_000);
    assert_eq!(default.block_size, 128);
    assert_eq!(default.validate(), Ok(()));

    for block_size in [1, 3, 64, 128, 256, usize::MAX] {
        assert_eq!(
            VmConfig {
                block_size,
                ..default
            }
            .validate(),
            Ok(())
        );
    }
    for sample_rate in [8_000, 384_000] {
        assert_eq!(
            VmConfig {
                sample_rate,
                ..default
            }
            .validate(),
            Ok(())
        );
    }
    assert_eq!(
        VmConfig {
            sample_rate: 7_999,
            ..default
        }
        .validate(),
        Err(VmConfigError::InvalidSampleRate(7_999))
    );
    assert_eq!(
        VmConfig {
            block_size: 0,
            ..default
        }
        .validate(),
        Err(VmConfigError::ZeroBlockSize)
    );
}

#[test]
fn spec_5_7_constant_output_and_empty_blocks_preserve_position() {
    let program = constant_program(0.25);
    let mut vm = Vm::new(&program, VmConfig::default()).expect("the constant VM should be valid");
    vm.render_into(&mut []).expect("an empty block is a no-op");
    assert_eq!(vm.sample_index(), 0);

    let mut first = [0.0; 3];
    vm.render_into(&mut first)
        .expect("constant render must work");
    assert_eq!(first, [0.25; 3]);
    assert_eq!(vm.sample_index(), 3);

    vm.render_into(&mut [])
        .expect("empty calls do not advance state");
    let mut second = [0.0; 2];
    vm.render_into(&mut second)
        .expect("variable block must work");
    assert_eq!(second, [0.25; 2]);
    assert_eq!(vm.sample_index(), 5);
}

#[test]
fn spec_5_7_scalar_and_audio_arithmetic_use_independent_numeric_vectors() {
    let scalar = bytecode(
        vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 7.0,
            },
            Instruction::ConstScalar {
                dst: ScalarRegister::new(1),
                value: 2.0,
            },
            Instruction::BinaryScalar {
                op: BinaryOp::Divide,
                dst: ScalarRegister::new(2),
                lhs: ScalarRegister::new(0),
                rhs: ScalarRegister::new(1),
            },
            Instruction::NegScalar {
                dst: ScalarRegister::new(3),
                src: ScalarRegister::new(2),
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(3),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(0),
            },
        ],
        4,
        2,
        0,
        1,
    );
    let rendered = render_with_partitions(&scalar, 3, 4, &[3]).expect("scalar arithmetic works");
    assert_eq!(rendered, [-3.5; 4]);

    let mixed = bytecode(
        vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 2.0,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::ConstScalar {
                dst: ScalarRegister::new(1),
                value: 3.0,
            },
            Instruction::BinaryAudio {
                op: BinaryOp::Multiply,
                dst: AudioRegister::new(1),
                lhs: Operand::Audio(AudioRegister::new(0)),
                rhs: Operand::Scalar(ScalarRegister::new(1)),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(1),
            },
        ],
        2,
        3,
        0,
        2,
    );
    let rendered = render_with_partitions(&mixed, 2, 4, &[1, 2]).expect("mixed operands work");
    assert_eq!(rendered, [6.0; 4]);
}

#[test]
fn adr_0006_scalar_and_lifted_sine_paths_match_analytic_samples() {
    let scalar = render_with_partitions(&sine_program(480.0, true), 128, 101, &[128])
        .expect("scalar sine path works");
    let lifted = render_with_partitions(&sine_program(480.0, false), 128, 101, &[128])
        .expect("lifted sine path works");

    for index in 0..=100 {
        let expected = match index {
            0 | 50 | 100 => 0.0,
            25 => 1.0,
            75 => -1.0,
            _ => independent_sine(480.0, index),
        };
        assert_close(scalar[index], expected, index);
        assert_close(lifted[index], expected, index);
        assert_close(scalar[index], lifted[index], index);
    }
}

#[test]
fn spec_7_4_required_block_sizes_have_equivalent_samples() {
    let program = sine_program(480.0, false);
    let baseline = render_with_partitions(&program, 1, 257, &[1]).expect("block one works");

    for block_size in [3, 64, 128, 256] {
        let actual = render_with_partitions(&program, block_size, 257, &[block_size])
            .expect("required block size works");
        assert_eq!(actual.len(), baseline.len());
        for (index, (actual, expected)) in actual.iter().zip(&baseline).enumerate() {
            assert_close(*actual, *expected, index);
        }
    }
}

#[test]
fn spec_7_4_fixed_variable_partitions_match_whole_blocks() {
    let program = sine_program(440.0, false);
    let baseline = render_with_partitions(&program, 128, 480, &[128]).expect("baseline works");
    let partitions = [1, 7, 2, 128, 3, 31, 5];
    let actual =
        render_with_partitions(&program, 128, 480, &partitions).expect("variable partitions work");

    for (index, (actual, expected)) in actual.iter().zip(&baseline).enumerate() {
        assert_close(*actual, *expected, index);
    }
}

#[test]
fn spec_5_7_reset_restores_state_registers_and_sample_index() {
    let program = sine_program(440.0, false);
    let mut vm = Vm::new(
        &program,
        VmConfig {
            block_size: 96,
            ..VmConfig::default()
        },
    )
    .expect("the sine VM should be valid");
    let mut first = vec![0.0; 96];
    vm.render_into(&mut first).expect("first render works");
    assert_eq!(vm.sample_index(), 96);

    vm.reset();
    assert_eq!(vm.sample_index(), 0);
    let mut second = vec![0.0; 96];
    vm.render_into(&mut second).expect("reset render works");
    for (index, (actual, expected)) in second.iter().zip(&first).enumerate() {
        assert_eq!(actual.to_bits(), expected.to_bits(), "sample {index}");
    }
}

#[test]
fn spec_5_7_oversized_block_is_rejected_before_dispatch() {
    let program = sine_program(480.0, false);
    let mut vm = Vm::new(
        &program,
        VmConfig {
            block_size: 2,
            ..VmConfig::default()
        },
    )
    .expect("the VM config is valid");
    let error = vm
        .render_into(&mut [0.0; 3])
        .expect_err("a block above the configured maximum must fail");

    assert_eq!(
        error,
        VmRuntimeError::BlockTooLarge {
            requested: 3,
            maximum: 2,
        }
    );
    assert_eq!(vm.sample_index(), 0);
    let mut valid = [0.0; 2];
    vm.render_into(&mut valid).expect("the VM remains usable");
    assert_close(valid[0], 0.0, 0);
    assert_close(valid[1], independent_sine(480.0, 1), 1);
}

#[test]
fn adr_0006_nonfinite_frequency_has_structured_instruction_state_and_index() {
    let program = bytecode(
        vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 0.0,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::BinaryAudio {
                op: BinaryOp::Divide,
                dst: AudioRegister::new(1),
                lhs: Operand::Audio(AudioRegister::new(0)),
                rhs: Operand::Audio(AudioRegister::new(0)),
            },
            Instruction::Sine {
                dst: AudioRegister::new(2),
                freq: Operand::Audio(AudioRegister::new(1)),
                state: StateId::new(0),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(2),
            },
        ],
        1,
        4,
        1,
        3,
    );
    let mut vm = Vm::new(&program, VmConfig::default()).expect("dynamic division is valid");
    let error = vm
        .render_into(&mut [0.0; 3])
        .expect_err("zero divided by zero produces invalid frequency");

    assert_eq!(
        error,
        VmRuntimeError::NonFiniteFrequency {
            instruction: 3,
            state: StateId::new(0),
            sample_index: 0,
        }
    );
}

#[test]
fn spec_5_7_nonfinite_output_has_a_structured_sample_index() {
    let program = bytecode(
        vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 0.0,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::BinaryAudio {
                op: BinaryOp::Divide,
                dst: AudioRegister::new(1),
                lhs: Operand::Audio(AudioRegister::new(0)),
                rhs: Operand::Audio(AudioRegister::new(0)),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(1),
            },
        ],
        1,
        3,
        0,
        2,
    );
    let mut vm = Vm::new(&program, VmConfig::default()).expect("dynamic division is valid");
    let error = vm
        .render_into(&mut [0.0; 3])
        .expect_err("zero divided by zero produces a nonfinite output");

    assert_eq!(error, VmRuntimeError::NonFiniteOutput { sample_index: 0 });
    assert_eq!(vm.sample_index(), 0);
}

#[test]
fn spec_6_2_vm_runtime_error_uses_the_absolute_sample_index() {
    let program = compile(
        "let out = sine(sine(0) / (sine(480) + 1)); render 2ms;",
        RenderConfig::default(),
    )
    .expect("the default config is valid")
    .result
    .expect("the runtime-error source should compile");
    let mut vm = create_vm(&program).expect("compiled bytecode must create a VM");
    let mut error = None;
    for _ in 0..6 {
        let mut block = [0.0; 16];
        if let Err(runtime_error) = vm.render_into(&mut block) {
            error = Some(runtime_error);
            break;
        }
    }
    let error = error.expect("the 96-sample render must fail at sample 75");

    let VmRuntimeError::NonFiniteFrequency {
        instruction,
        state,
        sample_index,
    } = error
    else {
        panic!("the failing sample must be a frequency error");
    };
    assert_eq!(sample_index, 75);
    assert_eq!(state, StateId::new(2));
    assert!(matches!(
        program.bytecode.instructions[instruction],
        Instruction::Sine { state: actual, .. } if actual == state
    ));
}

#[test]
fn spec_5_7_invalid_bytecode_is_rejected_before_vm_dispatch() {
    let mut program = constant_program(1.0);
    program.instructions[1] = Instruction::Lift {
        dst: AudioRegister::new(0),
        src: ScalarRegister::new(9),
    };
    let error =
        Vm::new(&program, VmConfig::default()).expect_err("invalid bytecode must not be executed");

    assert_eq!(
        error,
        VmCreateError::InvalidBytecode(BytecodeVerifyError::UnknownScalarRegister {
            instruction: 1,
            register: ScalarRegister::new(9),
        })
    );
}

#[test]
fn adr_0005_debug_stripping_does_not_change_sound() {
    let mut regular = sine_program(480.0, false);
    regular.debug_origins = Some(Vec::new());
    let mut stripped = regular.clone();
    stripped.strip_debug_info();

    let expected = render_with_partitions(&regular, 128, 101, &[128])
        .expect("regular bytecode should execute without origin lookup");
    let actual = render_with_partitions(&stripped, 17, 101, &[17])
        .expect("stripped bytecode should execute");
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_close(*actual, *expected, index);
    }
}

#[test]
fn spec_5_8_facade_creates_vm_with_the_compiled_render_configuration() {
    let program = compile(
        "let out = sine(480); render 1ms;",
        RenderConfig {
            block_size: 3,
            ..RenderConfig::default()
        },
    )
    .expect("the custom config is valid")
    .result
    .expect("the source should compile");
    let mut vm = create_vm(&program).expect("the facade should create a VM");
    assert_eq!(vm.config().sample_rate, 48_000);
    assert_eq!(vm.config().block_size, 3);

    let mut output = [0.0; 3];
    vm.render_into(&mut output).expect("facade VM renders");
    for (index, sample) in output.into_iter().enumerate() {
        assert_close(sample, independent_sine(480.0, index), index);
    }
}

#[test]
fn spec_10_1_vm_rejects_buffer_size_overflow_before_allocation() {
    let program = constant_program(1.0);
    let error = Vm::new(
        &program,
        VmConfig {
            block_size: usize::MAX,
            ..VmConfig::default()
        },
    )
    .expect_err("two maximum-sized audio buffers must overflow the size calculation");
    assert_eq!(error, VmCreateError::SizeOverflow);

    for (scalar_count, audio_count, state_count) in
        [(usize::MAX, 2, 0), (1, usize::MAX, 0), (1, 2, usize::MAX)]
    {
        let mut program = constant_program(1.0);
        program.scalar_register_count = scalar_count;
        program.audio_register_count = audio_count;
        program.state_count = state_count;

        let _ = linen::bytecode::verify(&program);
        let error = Vm::new(&program, VmConfig::default())
            .expect_err("declared runtime sizes must be rejected before allocation");
        assert_eq!(error, VmCreateError::SizeOverflow);
    }
}
