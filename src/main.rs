mod cpu;
mod hardware;

use hardware::init_hardware;
use std::time::*;

fn main() {
    let (mut input, mut display) = init_hardware(160, 144);

    let mut cpu = cpu::Cpu::new();

    while let Ok(input_state) = input.process_input() {
        let tick_result = cpu.tick();

        if tick_result.screen_changed {
            display.draw_screen(cpu.get_screen_buffer_ref());
        }
    }
}
