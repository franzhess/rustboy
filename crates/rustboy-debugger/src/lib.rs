//! Debugger use cases will live here.
//!
//! The crate deliberately depends only on `rustboy-core`; terminal and graphical
//! user interfaces belong in debugger adapter crates.

pub use rustboy_core::{
    AudioBuffer, CpuDiagnostic, CpuState, Frame, Machine, RegisterValues, StepResult,
};
