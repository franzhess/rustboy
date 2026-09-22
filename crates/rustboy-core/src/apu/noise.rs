use crate::apu::VolumeEnvelope;

pub struct Noise {
    enabled: bool,
    counter: usize,
    period: usize,
    lfsr: u16,   //gameboy has a 15bit lfsr - 16bit is good enough :)
    short: bool, //gameboy has 15bit/7bit - 16/8 for us
    length_data: u8,
    duration: u16,
    length_enabled: bool,
    volume_envelope: VolumeEnvelope,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            enabled: false,
            counter: 0,
            period: 1,
            lfsr: 0x7FFF,
            short: false,
            length_data: 0,
            duration: 0,
            length_enabled: false,
            volume_envelope: VolumeEnvelope::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF20 => self.length_data | 0xC0,
            0xFF21 => self.volume_envelope.read_byte(),
            _ => 0x3F,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF20 => {
                // NR41 has six length bits. Like the square channels, it stores the complement
                // of the effective 64-step length counter.
                self.length_data = value & 0b0011_1111;
                self.duration = 64 - self.length_data as u16;
            }
            0xFF21 => self.volume_envelope.write_byte(value),
            0xFF22 => {
                self.short = value & 0b0000_1000 == 0b0000_1000;
                let divider = match value & 0b0000_0111 {
                    0 => 8,
                    n => n as usize * 16,
                };
                self.period = divider << (value >> 4);
            }
            0xFF23 => {
                self.length_enabled = value & 0b0100_0000 == 0b0100_0000;

                if value & 0b1000_0000 == 0b1000_0000 {
                    // Triggering an expired noise channel reloads its maximum length. An active
                    // counter is not restarted merely because the channel is retriggered.
                    if self.duration == 0 {
                        self.duration = 64;
                    }
                    self.enabled = true;
                    self.volume_envelope.reset();
                }
            }
            _ => (),
        }
    }

    pub fn do_ticks(&mut self, ticks: usize) {
        self.counter += ticks;

        while self.counter >= self.period {
            self.counter -= self.period;

            let bit = self.lfsr & 1 ^ (self.lfsr >> 1) & 1;
            self.lfsr = self.lfsr >> 1 | bit << 15;
            if self.short {
                self.lfsr |= bit << 6;
            }
        }
    }

    pub fn get_sample(&self) -> i16 {
        if self.enabled {
            if self.lfsr & 1 == 1 {
                -self.volume_envelope.get_volume()
            } else {
                self.volume_envelope.get_volume()
            }
        } else {
            0
        }
    }

    pub fn timer_step(&mut self) {
        if self.enabled && self.length_enabled {
            self.duration -= 1;
            self.enabled = self.duration > 0;
        }
    }

    pub fn envelope_step(&mut self) {
        self.volume_envelope.step();
    }
}

#[cfg(test)]
mod tests {
    use super::Noise;

    #[test]
    fn noise_length_register_uses_all_six_bits() {
        let mut noise = Noise::new();
        noise.write_byte(0xFF20, 0);
        assert_eq!(noise.duration, 64);

        noise.write_byte(0xFF20, 0b0011_1111);
        assert_eq!(noise.duration, 1);
    }

    #[test]
    fn triggering_an_empty_noise_counter_reloads_its_maximum_length() {
        let mut noise = Noise::new();
        noise.write_byte(0xFF23, 0b1100_0000);

        noise.timer_step();

        assert!(noise.is_enabled());
        assert_eq!(noise.duration, 63);
    }
}
