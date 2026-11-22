#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

use linen::{RenderConfig, compile, create_vm};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { std::alloc::System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { std::alloc::System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { std::alloc::System.realloc(pointer, layout, size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { std::alloc::System.dealloc(pointer, layout) }
    }
}

#[test]
fn spec_10_1_render_into_and_reset_have_zero_allocations_after_initialization() {
    for source in [
        include_str!("../fixtures/valid/stage-03-block-partitions.ln"),
        include_str!("../fixtures/valid/stage-04-blocks-pipeline.ln"),
        include_str!("../fixtures/valid/stage-05-oscillators.ln"),
        include_str!("../fixtures/valid/stage-05-stereo.ln"),
        include_str!("../fixtures/valid/stage-05-pan.ln"),
        include_str!("../fixtures/valid/stage-05-noise.ln"),
    ] {
        let program = compile(source, RenderConfig::default())
            .expect("the default config is valid")
            .result
            .expect("the allocation fixture must compile");
        let channels = program.bytecode.output.channels();
        let mut vm = create_vm(&program).expect("compiled bytecode must create a VM");
        let mut block = vec![0.0; program.config.block_size * channels];

        vm.render_into(&mut block)
            .expect("the initialization warm-up must succeed");

        let before = ALLOCATIONS.load(Ordering::Relaxed);
        for frames in [1, 7, 2, 128, 3, 0] {
            vm.render_into(&mut block[..frames * channels])
                .expect("steady-state rendering must succeed");
        }
        vm.reset();
        let after = ALLOCATIONS.load(Ordering::Relaxed);

        assert_eq!(
            after, before,
            "render_into and reset must not allocate heap memory"
        );
    }
    assert_stage5_stereo_render_and_reset_have_zero_allocations();
}

fn assert_stage5_stereo_render_and_reset_have_zero_allocations() {
    use linen::bytecode::{AudioRegister, Bytecode, Instruction, OutputLayout, ScalarRegister};
    use linen::vm::{Vm, VmConfig};

    let bytecode = Bytecode {
        instructions: vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 0.25,
            },
            Instruction::ConstScalar {
                dst: ScalarRegister::new(1),
                value: -0.5,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::Lift {
                dst: AudioRegister::new(1),
                src: ScalarRegister::new(1),
            },
            Instruction::MakeStereo {
                dst_left: AudioRegister::new(2),
                dst_right: AudioRegister::new(3),
                src_left: AudioRegister::new(0),
                src_right: AudioRegister::new(1),
            },
            Instruction::SetStereoOutput {
                src_left: AudioRegister::new(2),
                src_right: AudioRegister::new(3),
            },
        ],
        scalar_register_count: 2,
        audio_register_count: 6,
        state_count: 0,
        output: OutputLayout::Stereo {
            left: AudioRegister::new(4),
            right: AudioRegister::new(5),
        },
        debug_origins: None,
    };
    let mut vm = Vm::new(&bytecode, VmConfig::default()).unwrap();
    let mut block = vec![0.0; 256];
    vm.render_into(&mut block[..2]).unwrap();

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for length in [2, 6, 4, 256, 0] {
        vm.render_into(&mut block[..length]).unwrap();
    }
    vm.reset();
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after, before, "stereo steady-state must not allocate");
}
