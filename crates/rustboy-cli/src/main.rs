use rustboy_adapter_rom::FileRomSource;
use rustboy_adapter_sdl3::Sdl3Adapter;
use rustboy_application::{run, RomSource, Session};
use rustboy_core::{Machine, SCREEN_HEIGHT, SCREEN_WIDTH};

fn main() {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("Usage: rustboy <path-to-rom-or-zip>");
        std::process::exit(2);
    };
    let cartridge = FileRomSource::new(path)
        .load_cartridge()
        .unwrap_or_else(|error| {
            eprintln!("Could not load ROM: {error}");
            std::process::exit(1);
        });
    println!("Successfully loaded: {}", cartridge.name());

    let mut adapter = Sdl3Adapter::new(2 * SCREEN_WIDTH as u32, 2 * SCREEN_HEIGHT as u32)
        .unwrap_or_else(|error| {
            eprintln!("Could not initialize SDL3: {error}");
            std::process::exit(1);
        });
    let mut session = Session::new(Machine::new(cartridge));
    if let Err(error) = adapter.play() {
        eprintln!("Could not start audio: {error}");
    } else if let Err(error) = run(&mut adapter, &mut session, |diagnostic| {
        eprintln!("{diagnostic}")
    }) {
        eprintln!("Emulator stopped: {error}");
    }
    if let Err(error) = adapter.stop() {
        eprintln!("Could not stop audio: {error}");
    }
}
