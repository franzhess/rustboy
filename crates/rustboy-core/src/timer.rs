pub struct Timer {
    pub irq_timer: bool,

    divider: u16,

    timer_counter: u8,
    timer_modulo: u8,
    timer_enabled: bool,
    timer_bit: u8,
    reload_delay: Option<u8>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            irq_timer: false,

            divider: 0,

            timer_counter: 0,
            timer_modulo: 0,
            timer_enabled: false,
            timer_bit: 7,
            reload_delay: None,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.divider >> 8) as u8,
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => {
                0xF8 | (if self.timer_enabled { 0x04 } else { 0x0 })
                    | (match self.timer_bit {
                        9 => 0x00,
                        3 => 0x01,
                        5 => 0x02,
                        _ => 0x03,
                    })
            }
            _ => {
                println!("Read at unmapped timer address: {:#06X}", address);
                0x00
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                // Writing DIV resets the shared divider, which can turn the timer input from
                // high to low and therefore produces a regular TIMA edge.
                let previous_signal = self.timer_signal();
                self.divider = 0;
                self.increment_on_falling_edge(previous_signal);
            }
            0xFF05 => {
                // Instructions currently perform their write before advancing their aggregate
                // T-cycles. In that ordering, a TIMA write while a reload is pending occurs
                // before the transfer and cancels it. Reload-cycle bus priority needs CPU
                // M-cycle scheduling, which is deliberately kept outside this timer commit.
                self.reload_delay = None;
                self.timer_counter = value;
            }
            0xFF06 => self.timer_modulo = value,
            0xFF07 => {
                // TAC changes the timer's input multiplexer immediately. A transition from the
                // old selected signal to the new one is indistinguishable from a divider edge.
                let previous_signal = self.timer_signal();
                self.timer_enabled = value & 0x04 != 0;
                self.timer_bit = match value & 0x03 {
                    0 => 9,
                    1 => 3,
                    2 => 5,
                    _ => 7,
                };
                self.increment_on_falling_edge(previous_signal);
            }
            _ => {
                println!("Write to unmapped timer address: {:#06X}", address);
            }
        }
    }

    pub fn do_ticks(&mut self, ticks: usize) {
        // The CPU advances devices in instruction-sized batches. Stepping the divider one
        // T-cycle at a time preserves every selected-bit edge within those batches.
        for _ in 0..ticks {
            self.advance_reload_delay();

            let previous_signal = self.timer_signal();
            self.divider = self.divider.wrapping_add(1);
            self.increment_on_falling_edge(previous_signal);
        }
    }

    fn timer_signal(&self) -> bool {
        self.timer_enabled && self.divider & (1 << self.timer_bit) != 0
    }

    fn increment_on_falling_edge(&mut self, previous_signal: bool) {
        if previous_signal && !self.timer_signal() {
            // A TIMA overflow first exposes 0x00. The TMA reload and interrupt request happen
            // four T-cycles later, so code can observe and alter the intermediate state.
            if self.timer_counter == 0xFF {
                self.timer_counter = 0;
                self.reload_delay = Some(4);
            } else {
                self.timer_counter += 1;
            }
        }
    }

    fn advance_reload_delay(&mut self) {
        let Some(delay) = self.reload_delay else {
            return;
        };

        if delay == 1 {
            // TMA is sampled when the delayed reload occurs, so writes to TMA during the
            // overflow window alter the value that becomes visible in TIMA.
            self.timer_counter = self.timer_modulo;
            self.irq_timer = true;
            self.reload_delay = None;
        } else {
            self.reload_delay = Some(delay - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;

    fn timer_about_to_overflow() -> Timer {
        let mut timer = Timer::new();
        timer.write_byte(0xFF05, 0xFF);
        timer.write_byte(0xFF07, 0x05);
        timer
    }

    #[test]
    fn increments_tima_on_each_selected_divider_falling_edge() {
        for (frequency, period) in [(0, 1024), (1, 16), (2, 64), (3, 256)] {
            let mut timer = Timer::new();
            timer.write_byte(0xFF07, 0x04 | frequency);

            timer.do_ticks(period);

            assert_eq!(timer.read_byte(0xFF05), 1, "TAC frequency {frequency}");
        }
    }

    #[test]
    fn div_register_exposes_the_upper_byte_of_the_shared_divider() {
        let mut timer = Timer::new();

        timer.do_ticks(0x1200);

        assert_eq!(timer.read_byte(0xFF04), 0x12);
    }

    #[test]
    fn resetting_divider_increments_tima_when_the_selected_bit_is_high() {
        let mut timer = Timer::new();
        timer.write_byte(0xFF07, 0x05);
        timer.do_ticks(8);

        timer.write_byte(0xFF04, 0);

        assert_eq!(timer.read_byte(0xFF05), 1);
    }

    #[test]
    fn changing_tac_can_increment_tima_when_it_drops_the_selected_signal() {
        let mut timer = Timer::new();
        timer.write_byte(0xFF07, 0x05);
        timer.do_ticks(8);

        timer.write_byte(0xFF07, 0x06);

        assert_eq!(timer.read_byte(0xFF05), 1);
    }

    #[test]
    fn disabling_tac_increments_tima_when_the_selected_bit_is_high() {
        let mut timer = Timer::new();
        timer.write_byte(0xFF07, 0x05);
        timer.do_ticks(8);

        timer.write_byte(0xFF07, 0x01);

        assert_eq!(timer.read_byte(0xFF05), 1);
    }

    #[test]
    fn reloads_tima_and_requests_an_interrupt_four_cycles_after_overflow() {
        let mut timer = timer_about_to_overflow();
        timer.write_byte(0xFF06, 0xAB);

        timer.do_ticks(16);
        assert_eq!(timer.read_byte(0xFF05), 0);
        assert!(!timer.irq_timer);

        timer.do_ticks(3);
        assert_eq!(timer.read_byte(0xFF05), 0);
        assert!(!timer.irq_timer);

        timer.do_ticks(1);
        assert_eq!(timer.read_byte(0xFF05), 0xAB);
        assert!(timer.irq_timer);
    }

    #[test]
    fn writing_tima_during_the_overflow_window_cancels_the_reload() {
        let mut timer = timer_about_to_overflow();

        timer.do_ticks(16);
        timer.write_byte(0xFF05, 0x42);
        timer.do_ticks(4);

        assert_eq!(timer.read_byte(0xFF05), 0x42);
        assert!(!timer.irq_timer);
    }

    #[test]
    fn reload_uses_tma_written_during_the_overflow_window() {
        let mut timer = timer_about_to_overflow();
        timer.write_byte(0xFF06, 0xAB);

        timer.do_ticks(16);
        timer.write_byte(0xFF06, 0xCD);
        timer.do_ticks(4);

        assert_eq!(timer.read_byte(0xFF05), 0xCD);
        assert!(timer.irq_timer);
    }

    #[test]
    fn reloads_and_continues_counting_within_one_tick_batch() {
        let mut timer = timer_about_to_overflow();
        timer.write_byte(0xFF06, 0xAB);

        // Overflow at cycle 16, reload at 20, and another falling edge at 32.
        timer.do_ticks(32);

        assert_eq!(timer.read_byte(0xFF05), 0xAC);
        assert!(timer.irq_timer);
    }

    #[test]
    fn register_induced_overflows_use_the_same_reload_delay() {
        for (address, value) in [(0xFF04, 0), (0xFF07, 0x06), (0xFF07, 0x01)] {
            let mut timer = timer_about_to_overflow();
            timer.write_byte(0xFF06, 0xAB);
            timer.do_ticks(8);

            // Reset DIV, select a low divider bit, or disable the timer while high.
            timer.write_byte(address, value);
            assert_eq!(timer.read_byte(0xFF05), 0);
            assert!(!timer.irq_timer);

            timer.do_ticks(3);
            assert_eq!(timer.read_byte(0xFF05), 0);
            assert!(!timer.irq_timer);

            timer.do_ticks(1);
            assert_eq!(timer.read_byte(0xFF05), 0xAB);
            assert!(timer.irq_timer);
        }
    }

    #[test]
    fn disabling_timer_does_not_cancel_a_pending_reload() {
        let mut timer = timer_about_to_overflow();
        timer.write_byte(0xFF06, 0xAB);
        timer.do_ticks(16);

        timer.write_byte(0xFF07, 0);
        timer.do_ticks(3);
        assert_eq!(timer.read_byte(0xFF05), 0);
        assert!(!timer.irq_timer);

        timer.do_ticks(1);
        assert_eq!(timer.read_byte(0xFF05), 0xAB);
        assert!(timer.irq_timer);
    }
}
