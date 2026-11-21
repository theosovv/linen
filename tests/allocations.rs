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
    let program = compile(
        include_str!("../fixtures/valid/stage-03-block-partitions.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid")
    .result
    .expect("the allocation fixture must compile");
    let mut vm = create_vm(&program).expect("compiled bytecode must create a VM");
    let mut block = vec![0.0; program.config.block_size];

    vm.render_into(&mut block)
        .expect("the initialization warm-up must succeed");

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for length in [1, 7, 2, 128, 3, 0] {
        vm.render_into(&mut block[..length])
            .expect("steady-state rendering must succeed");
    }
    vm.reset();
    let after = ALLOCATIONS.load(Ordering::Relaxed);

    assert_eq!(
        after, before,
        "render_into and reset must not allocate heap memory"
    );
}
