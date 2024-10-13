use std::{alloc::realloc, process::exit};

/// Reallocates memory with a new size.
///
/// # Safety
///
/// This function is unsafe because it deals with raw pointers and memory allocation.
/// The caller must ensure that:
/// - `ptr` is a valid pointer that was previously allocated with the global allocator.
/// - `old_size` matches the size that was used to allocate `ptr`.
/// - The memory referenced by `ptr` is no longer used after this function is called.
pub unsafe fn reallocate(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    if new_size == 0 {
        libc::free(ptr as *mut libc::c_void);
        return std::ptr::null_mut();
    }

    let new_ptr = libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8;
    if new_ptr.is_null() {
        std::process::exit(1);
    }
    new_ptr
}
