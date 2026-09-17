use libtest_mimic::{Arguments, Failed, Trial};
use rustboy_adapter_rom::{load_rom, LoadError};
use rustboy_application::{AudioSink, FrameSink, Session};
use rustboy_core::{Machine, CPU_FREQUENCY};
use serde::Deserialize;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Deserialize)]
struct Suite {
    name: String,
    tests: Vec<RomTest>,
}

#[derive(Deserialize)]
struct RomTest {
    name: String,
    rom: String,
    #[serde(default)]
    models: Vec<String>,
    exit: ExitCondition,
    success: Option<SuccessCondition>,
}

#[derive(Deserialize)]
struct ExitCondition {
    opcode: Option<u8>,
    time: Option<f64>,
}

#[derive(Deserialize)]
struct SuccessCondition {
    memory: Option<MemoryExpectation>,
    registers: Option<RegisterExpectation>,
}

#[derive(Deserialize)]
struct MemoryExpectation {
    address: u16,
    value: u8,
}

#[derive(Deserialize)]
struct RegisterExpectation {
    b: Option<u8>,
    c: Option<u8>,
    d: Option<u8>,
    e: Option<u8>,
    f: Option<u8>,
    h: Option<u8>,
    l: Option<u8>,
}

enum Outcome {
    Passed,
    Failed(String),
    Skipped(String),
}

struct TestHardware;

impl FrameSink for TestHardware {
    fn present_frame(&mut self, _screen_buffer: Vec<u8>) -> Result<(), String> {
        Ok(())
    }
}

impl AudioSink for TestHardware {
    fn queue_audio(&mut self, _sound_buffer: Vec<i16>) -> Result<(), String> {
        Ok(())
    }
}

fn main() -> ExitCode {
    let args = Arguments::from_args();
    let trials = if args.ignored || args.include_ignored {
        collect_trials()
    } else {
        Vec::new()
    };
    libtest_mimic::run(&args, trials).exit_code()
}

fn collect_trials() -> Vec<Trial> {
    let root = std::env::var_os("RUSTBOY_TEST_ROMS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("roms/GameboyTestSuites"));
    if !root.is_dir() {
        let message = format!(
            "test ROM suite not found at {}. Clone it with `git clone https://github.com/adtennant/GameboyTestSuites.git roms/GameboyTestSuites`",
            root.display()
        );
        return vec![Trial::test("rom_suite_setup", move || {
            Err(Failed::from(message))
        })];
    }

    let mut metadata_files: Vec<_> = fs::read_dir(root.join("tests"))
        .unwrap_or_else(|error| panic!("could not read suite metadata: {error}"))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    metadata_files.sort();

    let mut trials = Vec::new();
    for metadata in metadata_files {
        if metadata
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("json")
        {
            continue;
        }
        let suite: Suite = match fs::read(&metadata)
            .ok()
            .and_then(|contents| serde_json::from_slice(&contents).ok())
        {
            Some(suite) => suite,
            None => continue,
        };
        for test in suite.tests {
            let name = format!("{}/{}", suite.name, test.name);
            if !test.models.iter().any(|model| model == "dmg") {
                continue;
            }
            let Some(success) = &test.success else {
                continue;
            };
            if success.memory.is_none() && success.registers.is_none() {
                continue;
            }
            let root = root.clone();
            let suite_name = suite.name.clone();
            trials.push(
                Trial::test(name.clone(), move || {
                    match catch_unwind(AssertUnwindSafe(|| run_test(&root, &suite_name, &test)))
                        .unwrap_or_else(|panic| Outcome::Failed(panic_message(panic)))
                    {
                        Outcome::Passed => Ok(()),
                        Outcome::Failed(reason) | Outcome::Skipped(reason) => Err(reason.into()),
                    }
                })
                .with_ignored_flag(true),
            );
        }
    }
    trials
}

fn panic_message(panic: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else {
        "emulator panicked without a message".to_owned()
    }
}

fn run_test(root: &Path, suite: &str, test: &RomTest) -> Outcome {
    let Some(success) = &test.success else {
        return Outcome::Skipped("no machine-readable success assertion".to_owned());
    };
    let rom_path = root.join("tests").join(suite).join(&test.rom);
    let rom = match load_rom(&rom_path) {
        Ok(rom) => rom,
        Err(LoadError::Cartridge(rustboy_core::mbc::RomLoadError::UnsupportedCartridge(
            cartridge_type,
        ))) => {
            return Outcome::Skipped(format!("unsupported cartridge type {cartridge_type:#04X}"));
        }
        Err(error) => return Outcome::Failed(format!("could not load ROM: {error}")),
    };
    let mut emulator = Session::new(Machine::new(rom));
    let mut hardware = TestHardware;
    let max_cycles = (test.exit.time.unwrap_or(5.0) * CPU_FREQUENCY as f64) as usize;
    if let Some(opcode) = test.exit.opcode {
        match run_until_opcode(&mut emulator, &mut hardware, opcode, max_cycles) {
            Ok(true) => (),
            Ok(false) => {
                return Outcome::Failed(format!("did not execute exit opcode {opcode:#04X}"));
            }
            Err(error) => return Outcome::Failed(error),
        }
    } else {
        if let Err(error) = run_for_cycles(&mut emulator, &mut hardware, max_cycles) {
            return Outcome::Failed(error);
        }
    }
    if let Some(memory) = &success.memory {
        let actual = emulator.machine().read_byte(memory.address);
        if actual != memory.value {
            return Outcome::Failed(format!(
                "memory at {:#06X}: expected {:#04X}, got {actual:#04X}",
                memory.address, memory.value
            ));
        }
    }
    if let Some(registers) = &success.registers {
        let actual = emulator.machine().registers();
        if let Some(value) = registers.b {
            if actual.b != value {
                return Outcome::Failed(format!(
                    "register B: expected {value:#04X}, got {:#04X}",
                    actual.b
                ));
            }
        }
        if let Some(value) = registers.c {
            if actual.c != value {
                return Outcome::Failed(format!(
                    "register C: expected {value:#04X}, got {:#04X}",
                    actual.c
                ));
            }
        }
        if let Some(value) = registers.d {
            if actual.d != value {
                return Outcome::Failed(format!(
                    "register D: expected {value:#04X}, got {:#04X}",
                    actual.d
                ));
            }
        }
        if let Some(value) = registers.e {
            if actual.e != value {
                return Outcome::Failed(format!(
                    "register E: expected {value:#04X}, got {:#04X}",
                    actual.e
                ));
            }
        }
        if let Some(value) = registers.f {
            if actual.f != value {
                return Outcome::Failed(format!(
                    "register F: expected {value:#04X}, got {:#04X}",
                    actual.f
                ));
            }
        }
        if let Some(value) = registers.h {
            if actual.h != value {
                return Outcome::Failed(format!(
                    "register H: expected {value:#04X}, got {:#04X}",
                    actual.h
                ));
            }
        }
        if let Some(value) = registers.l {
            if actual.l != value {
                return Outcome::Failed(format!(
                    "register L: expected {value:#04X}, got {:#04X}",
                    actual.l
                ));
            }
        }
    }
    Outcome::Passed
}

fn run_for_cycles(
    emulator: &mut Session,
    hardware: &mut TestHardware,
    cycles: usize,
) -> Result<(), String> {
    let mut elapsed = 0;
    while elapsed < cycles {
        elapsed += step(emulator, hardware)?;
    }
    Ok(())
}

fn run_until_opcode(
    emulator: &mut Session,
    hardware: &mut TestHardware,
    opcode: u8,
    max_cycles: usize,
) -> Result<bool, String> {
    let mut elapsed = 0;
    while elapsed < max_cycles {
        let executed_opcode = emulator.machine().next_opcode();
        let ticks = step(emulator, hardware)?;
        elapsed += ticks;
        if executed_opcode == Some(opcode) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn step(emulator: &mut Session, hardware: &mut TestHardware) -> Result<usize, String> {
    let result = emulator.step();
    if let Some(frame) = result.frame {
        hardware.present_frame(frame)?;
    }
    for buffer in result.audio_buffers {
        hardware.queue_audio(buffer)?;
    }
    Ok(result.cycles)
}
