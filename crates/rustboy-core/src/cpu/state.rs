use std::fmt;

/// The CPU's current execution state, separate from IME and fetch suppression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    Running,
    Halted,
    /// STOP currently idles and wakes like HALT; hardware STOP behavior is incomplete.
    Stopped,
    /// The current fallback idles and can wake on an enabled interrupt request.
    /// This does not yet model the hardware's permanent illegal-opcode lockup.
    IllegalOpcode,
}

/// An event emitted by the step that encounters a CPU execution problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuDiagnostic {
    IllegalOpcode { address: u16, opcode: u8 },
}

impl fmt::Display for CpuDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllegalOpcode { address, opcode } => {
                write!(f, "Illegal opcode 0x{opcode:02X} at 0x{address:04X}")
            }
        }
    }
}
