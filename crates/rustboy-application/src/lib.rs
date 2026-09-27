use rustboy_core::mbc::Cartridge;
use rustboy_core::{
    AudioBuffer, ButtonEvent, CpuDiagnostic, Frame, Machine, StepResult, CPU_FREQUENCY,
};
use std::error::Error;
use std::fmt;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub trait FrameSink {
    type Error: Error + 'static;

    /// Reads the frame during this call. Sinks retaining data must own a copy.
    fn present_frame(&mut self, frame: &Frame) -> Result<(), Self::Error>;
}

pub trait AudioSink {
    type Error: Error + 'static;

    /// Transfers samples during this call; queued playback must not retain the borrow.
    fn queue_audio(&mut self, samples: &AudioBuffer) -> Result<(), Self::Error>;
}

pub trait InputSource {
    type Error: Error + 'static;

    fn poll_input(&mut self) -> Result<RunState, Self::Error>;
    fn drain_input(&mut self) -> Vec<ButtonEvent>;
}

pub trait RomSource {
    /// The adapter's structured error, including any underlying cause.
    type Error: std::error::Error;

    fn load_cartridge(&self) -> Result<Cartridge, Self::Error>;
}

pub trait Platform: InputSource + FrameSink + AudioSink {}

impl<T: InputSource + FrameSink + AudioSink> Platform for T {}

/// The failed application operation, retaining the adapter's original error.
#[derive(Debug)]
pub enum RunError {
    Input(Box<dyn Error>),
    Frame(Box<dyn Error>),
    Audio(Box<dyn Error>),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(error) => write!(f, "could not poll input: {error}"),
            Self::Frame(error) => write!(f, "could not present frame: {error}"),
            Self::Audio(error) => write!(f, "could not queue audio: {error}"),
        }
    }
}

impl Error for RunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Input(error) | Self::Frame(error) | Self::Audio(error) => Some(error.as_ref()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Running,
    Quit,
}

pub struct Session {
    machine: Machine,
    time_slice: TimeSliceBudget,
}

impl Session {
    pub fn new(machine: Machine) -> Self {
        Self {
            machine,
            time_slice: TimeSliceBudget::default(),
        }
    }

    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn machine_mut(&mut self) -> &mut Machine {
        &mut self.machine
    }

    /// Executes an unpaced step, independently of time-slice accounting.
    pub fn step(&mut self) -> StepResult {
        self.machine.step()
    }

    pub fn process_input(&mut self, event: ButtonEvent) {
        self.machine.process_input(event);
    }
}

/// Application pacing rate, independent of the PPU's video-frame cadence.
pub const TIME_SLICES_PER_SECOND: usize = 60;

#[derive(Default)]
struct TimeSliceBudget {
    // Positive: unfinished work. Negative: instruction overshoot credited to
    // the next slice. Counts whole T-cycles, not host time.
    remaining_cycles: i64,
    // Fractional numerator carried when dividing CPU_FREQUENCY by the slice rate.
    cycle_remainder: usize,
}

impl TimeSliceBudget {
    fn begin_if_needed(&mut self) {
        // An output error can interrupt a slice. A retry finishes its remaining
        // budget instead of granting another slice's worth of emulated time.
        if self.remaining_cycles > 0 {
            return;
        }
        let numerator = self.cycle_remainder + CPU_FREQUENCY;
        self.remaining_cycles += (numerator / TIME_SLICES_PER_SECOND) as i64;
        self.cycle_remainder = numerator % TIME_SLICES_PER_SECOND;
    }

    fn record_step(&mut self, cycles: usize) {
        self.remaining_cycles -= cycles as i64;
    }
}

fn time_slice_sleep_duration(elapsed: Duration) -> Duration {
    (Duration::from_secs(1) / TIME_SLICES_PER_SECOND as u32).saturating_sub(elapsed)
}

/// Advances one nominal 1/60-second slice, or resumes an interrupted slice.
/// Returns cycles actually executed in this call; instruction overshoot and
/// fractional budgets carry between slices. This function never sleeps.
pub fn advance_time_slice(
    session: &mut Session,
    outputs: &mut (impl FrameSink + AudioSink),
    report_diagnostic: &mut impl FnMut(CpuDiagnostic),
) -> Result<usize, RunError> {
    session.time_slice.begin_if_needed();
    let mut cycles = 0;
    while session.time_slice.remaining_cycles > 0 {
        let result = session.step();
        cycles += result.cycles;
        // Account for execution before diagnostics/output can interrupt delivery.
        session.time_slice.record_step(result.cycles);
        if let Some(diagnostic) = result.diagnostic {
            report_diagnostic(diagnostic);
        }
        if let Some(frame) = &result.frame {
            outputs
                .present_frame(frame)
                .map_err(|error| RunError::Frame(Box::new(error)))?;
        }
        for buffer in &result.audio_buffers {
            outputs
                .queue_audio(buffer)
                .map_err(|error| RunError::Audio(Box::new(error)))?;
        }
    }
    Ok(cycles)
}

pub fn run(
    platform: &mut impl Platform,
    session: &mut Session,
    mut report_diagnostic: impl FnMut(CpuDiagnostic),
) -> Result<(), RunError> {
    while platform
        .poll_input()
        .map_err(|error| RunError::Input(Box::new(error)))?
        == RunState::Running
    {
        let slice_started = Instant::now();
        for event in platform.drain_input() {
            session.process_input(event);
        }
        advance_time_slice(session, platform, &mut report_diagnostic)?;
        sleep(time_slice_sleep_duration(slice_started.elapsed()));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rustboy_core::mbc::Cartridge;
    use std::convert::Infallible;
    use std::io;

    #[derive(Default)]
    struct FailingPlatform {
        frames: usize,
        audio_buffers: usize,
    }

    impl InputSource for FailingPlatform {
        type Error = io::Error;

        fn poll_input(&mut self) -> Result<RunState, Self::Error> {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "input disconnected",
            ))
        }

        fn drain_input(&mut self) -> Vec<ButtonEvent> {
            panic!("input must not be drained after polling fails");
        }
    }

    impl FrameSink for FailingPlatform {
        type Error = io::Error;

        fn present_frame(&mut self, _frame: &Frame) -> Result<(), Self::Error> {
            self.frames += 1;
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "display unavailable",
            ))
        }
    }

    impl AudioSink for FailingPlatform {
        type Error = io::Error;

        fn queue_audio(&mut self, _samples: &AudioBuffer) -> Result<(), Self::Error> {
            self.audio_buffers += 1;
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "audio disconnected",
            ))
        }
    }

    fn session_with_lcd(enabled: bool) -> Session {
        let mut rom = vec![0; 0x148];
        // LD A,LCDC; LDH (FF40),A; JR -2. Enabling the LCD produces a frame
        // before the first audio buffer; disabling it isolates audio delivery.
        rom[0x100..0x106].copy_from_slice(&[
            0x3E,
            if enabled { 0x80 } else { 0 },
            0xE0,
            0x40,
            0x18,
            0xFE,
        ]);
        Session::new(Machine::new(
            Cartridge::from_bytes(rom).expect("valid ROM header"),
        ))
    }

    fn assert_io_cause(error: &RunError, kind: io::ErrorKind, message: &str) {
        let cause = error
            .source()
            .expect("adapter cause")
            .downcast_ref::<io::Error>()
            .expect("original I/O error type");
        assert_eq!(cause.kind(), kind);
        assert_eq!(cause.to_string(), message);
    }

    #[test]
    fn input_failure_retains_its_cause_and_stops_before_advancing_the_machine() {
        let mut session = session_with_lcd(false);
        let initial_registers = session.machine().registers();
        let mut platform = FailingPlatform::default();

        let error = run(&mut platform, &mut session, |_| {}).expect_err("input fails");

        assert!(matches!(&error, RunError::Input(_)));
        assert_io_cause(&error, io::ErrorKind::Interrupted, "input disconnected");
        assert_eq!(
            error.to_string(),
            "could not poll input: input disconnected"
        );
        assert_eq!(session.machine().registers(), initial_registers);
        assert_eq!((platform.frames, platform.audio_buffers), (0, 0));
    }

    #[test]
    fn frame_failure_retains_its_cause_and_stops_output_delivery() {
        let mut session = session_with_lcd(true);
        let mut platform = FailingPlatform::default();

        let error = advance_time_slice(&mut session, &mut platform, &mut |_| {})
            .expect_err("display fails");

        assert!(matches!(&error, RunError::Frame(_)));
        assert_io_cause(
            &error,
            io::ErrorKind::PermissionDenied,
            "display unavailable",
        );
        assert_eq!(
            error.to_string(),
            "could not present frame: display unavailable"
        );
        assert_eq!((platform.frames, platform.audio_buffers), (1, 0));
    }

    #[test]
    fn audio_failure_retains_its_cause_and_stops_output_delivery() {
        let mut session = session_with_lcd(false);
        let mut platform = FailingPlatform::default();

        let error =
            advance_time_slice(&mut session, &mut platform, &mut |_| {}).expect_err("audio fails");

        assert!(matches!(&error, RunError::Audio(_)));
        assert_io_cause(&error, io::ErrorKind::BrokenPipe, "audio disconnected");
        assert_eq!(
            error.to_string(),
            "could not queue audio: audio disconnected"
        );
        assert_eq!((platform.frames, platform.audio_buffers), (0, 1));
    }

    #[derive(Default)]
    struct TestOutputs {
        frames: usize,
        audio_buffers: usize,
    }

    impl FrameSink for TestOutputs {
        type Error = Infallible;

        fn present_frame(&mut self, _frame: &Frame) -> Result<(), Self::Error> {
            self.frames += 1;
            Ok(())
        }
    }

    impl AudioSink for TestOutputs {
        type Error = Infallible;

        fn queue_audio(&mut self, _samples: &AudioBuffer) -> Result<(), Self::Error> {
            self.audio_buffers += 1;
            Ok(())
        }
    }

    fn looping_session(program: &[u8]) -> Session {
        let mut rom = vec![0; 0x8000];
        rom[0x100..0x100 + program.len()].copy_from_slice(program);
        Session::new(Machine::new(Cartridge::from_bytes(rom).expect("valid ROM")))
    }

    #[test]
    fn cumulative_slice_cycles_stay_within_one_instruction_of_the_exact_target() {
        // A 12-cycle JR loop and a mixed 4/12-cycle NOP/JR loop. Test two
        // nominal seconds without run(), host clocks, sleeps or real devices.
        for program in [&[0x18, 0xFE][..], &[0x00, 0x18, 0xFD][..]] {
            let mut session = looping_session(program);
            let mut outputs = TestOutputs::default();
            let mut total_cycles = 0;
            for slice in 1..=120 {
                total_cycles += advance_time_slice(&mut session, &mut outputs, &mut |_| {})
                    .expect("test sinks accept output");
                let target = slice * CPU_FREQUENCY / 60;
                assert!(
                    total_cycles >= target && total_cycles < target + 12,
                    "slice {slice}: executed {total_cycles}, target {target}"
                );
            }
            // Two exact CPU-frequency budgets include the fractional cycles
            // lost by repeatedly truncating CPU_FREQUENCY / 60.
            assert!(total_cycles >= 2 * CPU_FREQUENCY);
            assert_eq!(outputs.audio_buffers, 120);
            assert!(outputs.frames > 0 && outputs.frames < 120);
        }
    }

    #[test]
    fn retry_after_output_failure_finishes_the_interrupted_slice() {
        let mut reference = session_with_lcd(true);
        let complete_cycles =
            advance_time_slice(&mut reference, &mut TestOutputs::default(), &mut |_| {})
                .expect("reference slice");
        let mut session = session_with_lcd(true);
        let error = advance_time_slice(&mut session, &mut FailingPlatform::default(), &mut |_| {})
            .expect_err("frame sink fails before the slice ends");
        assert!(matches!(error, RunError::Frame(_)));

        let mut outputs = TestOutputs::default();
        let remaining_cycles = advance_time_slice(&mut session, &mut outputs, &mut |_| {})
            .expect("retry with working sinks");
        assert!(remaining_cycles < complete_cycles);
        assert_eq!(
            session.machine().registers(),
            reference.machine().registers()
        );
        for address in [0xFF04, 0xFF41, 0xFF44] {
            assert_eq!(
                session.machine().read_byte(address),
                reference.machine().read_byte(address),
                "register {address:04X}"
            );
        }
        // Failed output is not replayed; audio generated while completing the
        // remaining cycles is still delivered normally.
        assert_eq!((outputs.frames, outputs.audio_buffers), (0, 1));
    }

    #[test]
    fn advance_time_slice_runs_its_initial_budget_and_delivers_audio() {
        let cartridge = Cartridge::from_bytes(vec![0; 0x148]).expect("valid ROM header");
        let mut session = Session::new(Machine::new(cartridge));
        let mut outputs = TestOutputs::default();

        let cycles = advance_time_slice(&mut session, &mut outputs, &mut |_| {})
            .expect("test outputs accept all effects");

        let initial_target = CPU_FREQUENCY / TIME_SLICES_PER_SECOND;
        assert!(cycles >= initial_target);
        assert!(cycles < initial_target + 24);
        assert_eq!(outputs.audio_buffers, 1);
    }

    #[test]
    fn time_slice_sleep_only_waits_for_the_remaining_host_budget() {
        let slice = Duration::from_secs(1) / 60;

        assert_eq!(time_slice_sleep_duration(Duration::ZERO), slice);
        assert_eq!(
            time_slice_sleep_duration(Duration::from_millis(5)),
            slice - Duration::from_millis(5)
        );
        assert_eq!(
            time_slice_sleep_duration(slice + Duration::from_millis(1)),
            Duration::ZERO
        );
    }

    fn session_with_illegal_opcode() -> Session {
        let mut rom = vec![0; 0x148];
        rom[0x100] = 0xD3;
        Session::new(Machine::new(
            Cartridge::from_bytes(rom).expect("valid ROM header"),
        ))
    }

    #[test]
    fn diagnostic_is_delivered_once_while_idle_devices_keep_producing_output() {
        let mut session = session_with_illegal_opcode();
        let mut outputs = TestOutputs::default();
        let mut diagnostics = Vec::new();
        let mut total_cycles = 0;
        for slice in 1..=2 {
            let cycles = advance_time_slice(&mut session, &mut outputs, &mut |diagnostic| {
                diagnostics.push(diagnostic);
            })
            .expect("test outputs accept all effects");
            total_cycles += cycles;
            let target = slice * CPU_FREQUENCY / TIME_SLICES_PER_SECOND;
            assert!(total_cycles >= target && total_cycles < target + 4);
        }
        assert_eq!(
            diagnostics,
            [CpuDiagnostic::IllegalOpcode {
                address: 0x100,
                opcode: 0xD3
            }]
        );
        assert_eq!(
            session.machine().cpu_state(),
            rustboy_core::CpuState::IllegalOpcode
        );
        assert_eq!((outputs.frames, outputs.audio_buffers), (2, 2));
    }

    #[test]
    fn diagnostic_is_delivered_before_a_later_output_failure() {
        let mut session = session_with_illegal_opcode();
        let mut outputs = FailingPlatform::default();
        let mut diagnostics = Vec::new();
        let error = advance_time_slice(&mut session, &mut outputs, &mut |diagnostic| {
            diagnostics.push(diagnostic);
        })
        .expect_err("frame output fails");
        assert!(matches!(error, RunError::Frame(_)));
        assert_eq!(
            diagnostics,
            [CpuDiagnostic::IllegalOpcode {
                address: 0x100,
                opcode: 0xD3
            }]
        );
    }
}
