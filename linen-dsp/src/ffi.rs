//! C FFI for compiled code (v0.2+).
//!
//! These functions allow LLVM-generated code to call into the DSP runtime.

use crate::oscillators::SinOsc;
use crate::traits::SignalNode;

/// Create a new sine oscillator.
#[no_mangle]
pub extern "C" fn linen_dsp_sinosc_new(freq: f32) -> *mut SinOsc {
    let osc = Box::new(SinOsc::new(freq));
    Box::into_raw(osc)
}

/// Get next sample from a sine oscillator.
#[no_mangle]
pub extern "C" fn linen_dsp_sinosc_next(osc: *mut SinOsc, t: u64, sr: f32) -> f32 {
    let osc = unsafe { &mut *osc };
    osc.next_sample(t, sr)
}

/// Free a sine oscillator.
#[no_mangle]
pub extern "C" fn linen_dsp_sinosc_free(osc: *mut SinOsc) {
    if !osc.is_null() {
        unsafe {
            let _ = Box::from_raw(osc);
        }
    }
}
