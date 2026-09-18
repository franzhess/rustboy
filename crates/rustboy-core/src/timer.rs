pub struct Timer {
    pub irq_timer: bool,

    divider: u16,

    timer_counter: u8,
    timer_modulo: u8,
    timer_enabled: bool,
    timer_bit: u8,
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
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.divider >> 8) as u8,
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => {
                (if self.timer_enabled { 0x04 } else { 0x0 })
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
            0xFF05 => self.timer_counter = value,
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
            // The timer is clocked by a falling edge, not by a periodic accumulator. Overflow
            // reload timing is intentionally handled separately from edge detection.
            self.timer_counter = match self.timer_counter {
                0xFF => {
                    self.irq_timer = true;
                    self.timer_modulo
                }
                _ => self.timer_counter + 1,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;

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
}
