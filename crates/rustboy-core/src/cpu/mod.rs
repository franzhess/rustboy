mod alu;
mod op_codes;
mod op_codes_cb;
mod registers;

use crate::cpu::registers::{FlagRegister, RegisterName16, RegisterName8, Registers};
use crate::mbc::Mbc;
use crate::mmu::Mmu;
use crate::ButtonEvent;

pub enum OpCodeResult {
    Executed(usize),
    UnknownOpCode,
}

pub struct CpuStepResult {
    pub cycles: usize,
    pub opcode: Option<u8>,
}

type UnaryOperation8 = fn(&mut dyn FlagRegister, u8) -> u8;
type BinaryOperation8 = fn(&mut dyn FlagRegister, u8, u8) -> u8;
type BinaryOperation16 = fn(&mut dyn FlagRegister, u16, u16) -> u16;

pub struct Cpu {
    registers: Registers,
    mmu: Mmu,
    halted: bool,
    ime: bool,           // interrupt master enable - set by DI and EI
    ei_requested: usize, //EI has one cycle delay
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterValues {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
}

impl Cpu {
    pub fn new(rom: Box<dyn Mbc>) -> Cpu {
        Cpu {
            registers: Registers::new(),
            mmu: Mmu::new(rom),
            halted: false,
            ime: false,      //interrupt master enable
            ei_requested: 0, //enable interrupt requested - in the original gameboy the enabling of the interrupts took two cycles (see tick)
        }
    }

    pub fn tick(&mut self) -> CpuStepResult {
        self.ei_requested = match self.ei_requested {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };

        let result = if self.handle_irq() {
            // Interrupt entry is a five-M-cycle hardware sequence: acknowledge IF, push the
            // current PC, and load the vector. Devices continue running for all 20 T-cycles.
            CpuStepResult {
                cycles: 20,
                opcode: None,
            }
        } else if !self.halted {
            self.do_cycle()
        } else {
            CpuStepResult {
                cycles: 4,
                opcode: None,
            }
        };

        self.mmu.do_ticks(result.cycles);

        result
    }

    pub fn next_opcode(&self) -> Option<u8> {
        (!self.halted).then(|| self.mmu.read_byte(self.registers.pc))
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mmu.read_byte(address)
    }

    pub fn registers(&self) -> RegisterValues {
        RegisterValues {
            a: self.registers.a,
            b: self.registers.b,
            c: self.registers.c,
            d: self.registers.d,
            e: self.registers.e,
            f: self.registers.get_af() as u8,
            h: self.registers.h,
            l: self.registers.l,
        }
    }

    pub fn take_frame(&mut self) -> Option<Vec<u8>> {
        self.mmu.take_frame()
    }

    pub fn take_audio_buffers(&mut self) -> Vec<Vec<i16>> {
        self.mmu.take_audio_buffers()
    }

    fn handle_irq(&mut self) -> bool {
        self.mmu.process_irq_requests(); //loads the irq requests into 0xFF0F

        let irq_requested = self.mmu.read_byte(0xFF0F);
        let irq = self.mmu.read_byte(0xFFFF) & irq_requested & 0x1F;
        if irq == 0 {
            return false;
        }

        // Any enabled request wakes HALT, even when IME is clear and the CPU must defer the
        // actual interrupt service. The HALT-bug fetch behavior is handled separately.
        self.halted = false;
        if !self.ime {
            return false;
        }

        self.ime = false;
        let irq_num = irq.trailing_zeros(); //0 vblank, 1 stat, 2 timer, 3 serial, 4 joypad

        self.push(self.registers.pc);
        self.registers.pc = (0x0040 + 8 * irq_num) as u16;
        self.mmu.write_byte(0xFF0F, irq_requested & !(1 << irq_num));
        true
    }

    pub fn process_input_event(&mut self, event: ButtonEvent) {
        self.mmu.joypad.receive_event(event);
    }

    fn do_cycle(&mut self) -> CpuStepResult {
        let current_address = self.registers.pc;
        let op_code = self.fetch_byte();

        //println!("do_cycle: {:#04X} @ {:#06X}", op_code, current_address);

        let cycles = match op_codes::execute(op_code, self) {
            OpCodeResult::Executed(ticks) => ticks,
            OpCodeResult::UnknownOpCode => {
                println!(
                    "Unknown command {:#04X} at {:#06X}",
                    op_code, current_address
                );
                self.halted = true;
                4
            } //NOOP on unknown opcodes
        };
        CpuStepResult {
            cycles,
            opcode: Some(op_code),
        }
    }

    fn fetch_byte(&mut self) -> u8 {
        let res = self.mmu.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        res
    }

    fn fetch_word(&mut self) -> u16 {
        let res = self.mmu.read_word(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);
        res
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2); //stack grows down from 0xFFFE and stores words
                                                               //println!("pushing {:06X} to   {:06X}", value, self.registers.sp);
        self.mmu.write_word(self.registers.sp, value);
    }

    fn pop(&mut self) -> u16 {
        let result = self.mmu.read_word(self.registers.sp);
        //println!("popping {:06X} from {:06X}", result, self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(2);
        result
    }

    fn call(&mut self, address: u16) {
        //println!("CALL {:#06X}@{:#04X}", address, self.registers.pc - 1);
        self.push(self.registers.pc); //it's not pc + 1 because after fetching the address the pc is already at the next instruction
        self.registers.pc = address;
    }

    fn retrn(&mut self) {
        self.registers.pc = self.pop();
    }

    fn jump_r(&mut self) {
        let offset = self.fetch_byte();
        self.registers.pc = self.registers.pc.wrapping_add(offset as i8 as i16 as u16);
    }

    fn execute(&mut self, op: UnaryOperation8, arg: RegisterName8) {
        let value = self.registers.get(arg);
        let result = op(&mut self.registers, value);
        self.registers.set(arg, result);
    }

    fn execute_hl(&mut self, op: UnaryOperation8) {
        let original_value = self.mmu.read_byte(self.registers.get_hl());
        let new_value = op(&mut self.registers, original_value);
        self.mmu.write_byte(self.registers.get_hl(), new_value);
    }

    fn execute_binary(&mut self, op: BinaryOperation8, arg: RegisterName8) {
        let value2 = self.registers.get(arg);
        self.execute_binary_with_value(op, value2);
    }

    fn execute_binary_with_value(&mut self, op: BinaryOperation8, value2: u8) {
        let value1 = self.registers.a;
        let result = op(&mut self.registers, value1, value2);
        self.registers.a = result;
    }

    fn execute16(&mut self, op: BinaryOperation16, arg: RegisterName16) {
        let value1 = self.registers.get_hl();
        let value2 = self.registers.get16(arg);
        let result = op(&mut self.registers, value1, value2);
        self.registers.set_hl(result);
    }
}

#[cfg(test)]
mod tests {
    use super::{op_codes, Cpu};
    use crate::cpu::registers::{CpuFlag, FlagRegister};
    use crate::mbc::Mbc;

    struct TestMbc;

    impl Mbc for TestMbc {
        fn read_rom(&self, _address: u16) -> u8 {
            0
        }

        fn read_ram(&self, _address: u16) -> u8 {
            0
        }

        fn write_rom(&mut self, _address: u16, _value: u8) {}

        fn write_ram(&mut self, _address: u16, _value: u8) {}
    }

    #[test]
    fn instruction_fetch_wraps_the_program_counter() {
        let mut cpu = Cpu::new(Box::new(TestMbc));
        cpu.registers.pc = 0xFFFF;

        cpu.fetch_byte();
        assert_eq!(cpu.registers.pc, 0);

        cpu.registers.pc = 0xFFFE;
        cpu.fetch_word();
        assert_eq!(cpu.registers.pc, 0);
    }

    #[test]
    fn not_taken_conditional_jump_wraps_while_skipping_its_operand() {
        let mut cpu = Cpu::new(Box::new(TestMbc));
        cpu.registers.pc = 0xFFFF;
        cpu.registers.set_flag(CpuFlag::Z, true);

        // PC already points to JR NZ's one-byte operand after its opcode was fetched.
        op_codes::execute(0x20, &mut cpu);

        assert_eq!(cpu.registers.pc, 0);
    }

    #[test]
    fn highest_priority_interrupt_consumes_twenty_cycles_and_ticks_devices() {
        let mut cpu = Cpu::new(Box::new(TestMbc));
        cpu.registers.pc = 0x1234;
        cpu.registers.sp = 0xFFFE;
        cpu.ime = true;
        cpu.mmu.write_byte(0xFFFF, 0b0000_0101);
        cpu.mmu.write_byte(0xFF0F, 0b0000_0101);
        cpu.mmu.write_byte(0xFF07, 0x05);

        let result = cpu.tick();
        assert_eq!(result.cycles, 20);
        assert_eq!(result.opcode, None);
        assert!(!cpu.ime);

        // VBlank (bit 0) wins over the simultaneously pending timer interrupt (bit 2).
        assert_eq!(cpu.registers.pc, 0x0040);
        assert_eq!(cpu.registers.sp, 0xFFFC);
        assert_eq!(cpu.mmu.read_word(0xFFFC), 0x1234);
        assert_eq!(cpu.mmu.read_byte(0xFF0F), 0b0000_0100);

        // Twenty T-cycles include the timer's first bit-3 falling edge at cycle 16.
        assert_eq!(cpu.mmu.read_byte(0xFF05), 1);

        // The handler's NOP runs on the next step, without servicing the pending timer IRQ.
        let result = cpu.tick();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, Some(0x00));
        assert_eq!(cpu.registers.pc, 0x0041);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn halt_idle_reports_no_opcode_but_waking_execution_does() {
        let mut cpu = Cpu::new(Box::new(TestMbc));
        cpu.registers.pc = 0xC000;
        cpu.mmu.write_byte(0xC000, 0x76); // HALT
        cpu.mmu.write_byte(0xC001, 0x04); // INC B

        assert_eq!(cpu.tick().opcode, Some(0x76));
        let result = cpu.tick();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, None);
        assert_eq!(cpu.registers.pc, 0xC001);

        cpu.mmu.write_byte(0xFFFF, 1);
        cpu.mmu.write_byte(0xFF0F, 1);
        let result = cpu.tick();
        assert_eq!(result.cycles, 4);
        assert_eq!(result.opcode, Some(0x04));
        assert_eq!(cpu.registers.pc, 0xC002);
    }
}
