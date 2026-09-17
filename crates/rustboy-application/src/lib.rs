use rustboy_core::mbc::Cartridge;
use rustboy_core::{ButtonEvent, Machine, StepResult, CPU_FREQUENCY};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub trait FrameSink {
    fn present_frame(&mut self, frame: Vec<u8>) -> Result<(), String>;
}

pub trait AudioSink {
    fn queue_audio(&mut self, samples: Vec<i16>) -> Result<(), String>;
}

pub trait InputSource {
    fn poll_input(&mut self) -> Result<RunState, String>;
    fn drain_input(&mut self) -> Vec<ButtonEvent>;
}

pub trait RomSource {
    fn load_cartridge(&self) -> Result<Cartridge, String>;
}

pub trait Platform: InputSource + FrameSink + AudioSink {}

impl<T: InputSource + FrameSink + AudioSink> Platform for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Running,
    Quit,
}

pub struct Session {
    machine: Machine,
}

impl Session {
    pub fn new(machine: Machine) -> Self {
        Self { machine }
    }

    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn machine_mut(&mut self) -> &mut Machine {
        &mut self.machine
    }

    pub fn step(&mut self) -> StepResult {
        self.machine.step()
    }

    pub fn process_input(&mut self, event: ButtonEvent) {
        self.machine.process_input(event);
    }
}

pub const FRAME_CYCLE_BUDGET: usize = CPU_FREQUENCY / 60;

fn frame_sleep_duration(elapsed: Duration) -> Duration {
    (Duration::from_secs(1) / 60).saturating_sub(elapsed)
}

pub fn advance_frame(
    session: &mut Session,
    outputs: &mut (impl FrameSink + AudioSink),
) -> Result<usize, String> {
    let mut cycles = 0;
    while cycles < FRAME_CYCLE_BUDGET {
        let result = session.step();
        cycles += result.cycles;
        if let Some(frame) = result.frame {
            outputs.present_frame(frame)?;
        }
        for buffer in result.audio_buffers {
            outputs.queue_audio(buffer)?;
        }
    }
    Ok(cycles)
}

pub fn run(platform: &mut impl Platform, session: &mut Session) -> Result<(), String> {
    while platform.poll_input()? == RunState::Running {
        let slice_started = Instant::now();
        for event in platform.drain_input() {
            session.process_input(event);
        }
        advance_frame(session, platform)?;
        sleep(frame_sleep_duration(slice_started.elapsed()));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rustboy_core::mbc::Cartridge;

    #[derive(Default)]
    struct TestOutputs {
        frames: usize,
        audio_buffers: usize,
    }

    impl FrameSink for TestOutputs {
        fn present_frame(&mut self, _frame: Vec<u8>) -> Result<(), String> {
            self.frames += 1;
            Ok(())
        }
    }

    impl AudioSink for TestOutputs {
        fn queue_audio(&mut self, _samples: Vec<i16>) -> Result<(), String> {
            self.audio_buffers += 1;
            Ok(())
        }
    }

    #[test]
    fn advance_frame_runs_a_full_cycle_budget_and_delivers_audio() {
        let cartridge = Cartridge::from_bytes(vec![0; 0x148]).expect("valid ROM header");
        let mut session = Session::new(Machine::new(cartridge));
        let mut outputs = TestOutputs::default();

        let cycles =
            advance_frame(&mut session, &mut outputs).expect("test outputs accept all effects");

        assert!(cycles >= FRAME_CYCLE_BUDGET);
        assert!(cycles <= FRAME_CYCLE_BUDGET + 24);
        assert_eq!(outputs.audio_buffers, 1);
    }

    #[test]
    fn frame_sleep_only_waits_for_the_remaining_frame_budget() {
        let frame = Duration::from_secs(1) / 60;

        assert_eq!(frame_sleep_duration(Duration::ZERO), frame);
        assert_eq!(
            frame_sleep_duration(Duration::from_millis(5)),
            frame - Duration::from_millis(5)
        );
        assert_eq!(
            frame_sleep_duration(frame + Duration::from_millis(1)),
            Duration::ZERO
        );
    }
}
