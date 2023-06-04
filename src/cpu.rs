use crate::bus::BUS;

#[derive(Debug)]
pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub ps: u8,
    pub bus: BUS,
}

enum AddressingModes {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

fn get_nth_bit_u8(byte: u8, n: u8) -> u8 {
    return (byte >> n) & 1;
}

fn get_nth_bit_u16(byte: u16, n: u8) -> u16 {
    return (byte >> n) & 1;
}

fn assemble_2_bytes_le_u16(ms_byte: u8, ls_byte: u8) -> u16 {
    return ((ms_byte as u16) << 8) | (ls_byte as u16);
}

impl CPU {
    pub fn init() -> Self {
        return CPU {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            ps: 0,
            bus: BUS::init(),
        };
    }

    pub fn read_byte_from_memory(&mut self, addr: u16) -> u8 {
        self.update_program_counter_n(1);
        return self.bus.read_memory_byte(addr);
    }

    pub fn write_byte_to_memory(&mut self, addr: u16, val: u8) {
        self.update_program_counter_n(1);
        self.bus.write_memory_byte(addr, val);
    }

    pub fn read_2_bytes_from_memory(&mut self, addr: u16) -> u16 {
        let ls_byte = self.read_byte_from_memory(addr);
        let ms_byte = self.read_byte_from_memory(addr + 1);

        return assemble_2_bytes_le_u16(ms_byte, ls_byte);
    }

    pub fn write_2_bytes_to_memory(&mut self, addr: u16, val: u16) {
        let ls_byte = (val >> 8) as u8;
        let ms_byte = (val & 0xFF) as u8;

        self.write_byte_to_memory(addr, ms_byte);
        self.write_byte_to_memory(addr + 1, ls_byte);
    }
    // prob needs some work
    fn handle_addressing_mode(&mut self, mode: AddressingModes) -> u16 {
        match mode {
            AddressingModes::Implicit => return 0,
            AddressingModes::Accumulator => return self.fetch_a() as u16,
            AddressingModes::Immediate => return self.fetch_pc(),
            AddressingModes::ZeroPage => {
                let pc = self.fetch_pc();
                return self.read_byte_from_memory(pc) as u16;
            }
            AddressingModes::ZeroPageX => {
                let x = self.fetch_x() as u16;
                let pc = self.fetch_pc();
                return self.read_byte_from_memory(pc.wrapping_add(x)) as u16;
            }
            AddressingModes::ZeroPageY => {
                let y = self.fetch_y() as u16;
                let pc = self.fetch_pc();
                return self.read_byte_from_memory(pc.wrapping_add(y)) as u16;
            }
            AddressingModes::Relative => {
                let pc = self.fetch_pc();
                let pc_2 = self.fetch_pc();
                return pc.wrapping_add_signed(self.read_byte_from_memory(pc_2) as i16);
            }
            AddressingModes::Absolute => {
                let pc = self.fetch_pc();
                return self.read_2_bytes_from_memory(pc);
            }
            AddressingModes::AbsoluteX => {
                let pc = self.fetch_pc();
                let x = self.fetch_x() as u16;
                return self.read_2_bytes_from_memory(pc.wrapping_add(x));
            }
            AddressingModes::AbsoluteY => {
                let pc = self.fetch_pc();
                let y = self.fetch_y() as u16;
                return self.read_2_bytes_from_memory(pc.wrapping_add(y));
            }
            AddressingModes::Indirect => {
                let pc = self.fetch_pc();
                let ls_byte_location = self.read_byte_from_memory(pc) as u16;
                return self.read_2_bytes_from_memory(ls_byte_location);
            }
            AddressingModes::IndirectX => {
                let pc = self.fetch_pc();
                let x = self.fetch_x() as u16;
                let ls_byte_location = self.read_byte_from_memory(pc.wrapping_add(x)) as u16;
                return self.read_2_bytes_from_memory(ls_byte_location);
            }
            AddressingModes::IndirectY => {
                let pc = self.fetch_pc();
                let ls_byte_location = self.read_byte_from_memory(pc) as u16;
                let y = self.fetch_y();
                return self
                    .read_2_bytes_from_memory(ls_byte_location)
                    .wrapping_add(y as u16);
            }
            _ => {
                println!("Unknown mode");
                return 0;
            }
        }
    }

    fn get_carry_flag(&self) -> u8 {
        return self.ps & 0b00000001;
    }

    fn get_zero_flag(&self) -> u8 {
        return self.ps & 0b00000010;
    }

    fn get_interrupt_disable(&self) -> u8 {
        return self.ps & 0b00000100;
    }

    fn get_decimal_mode(&self) -> u8 {
        return self.ps & 0b00001000;
    }

    fn get_break_command(&self) -> u8 {
        return self.ps & 0b00010000;
    }

    fn get_overflow_flag(&self) -> u8 {
        return self.ps & 0b01000000;
    }

    fn get_negative_flag(&self) -> u8 {
        return self.ps & 0b10000000;
    }

    fn set_carry_flag(&mut self) {
        self.ps = self.ps | 0b00000001;
    }

    fn set_zero_flag(&mut self) {
        self.ps = self.ps | 0b00000010;
    }

    fn set_interrupt_disable(&mut self) {
        self.ps = self.ps | 0b00000100;
    }

    fn set_decimal_mode(&mut self) {
        self.ps = self.ps | 0b00001000;
    }

    fn set_break_command(&mut self) {
        self.ps = self.ps | 0b00010000;
    }

    fn set_overflow_flag(&mut self) {
        self.ps = self.ps | 0b01000000;
    }

    fn set_negative_flag(&mut self) {
        self.ps = self.ps | 0b10000000;
    }

    fn unset_carry_flag(&mut self) {
        self.ps = self.ps & 0b11111110;
    }

    fn unset_zero_flag(&mut self) {
        self.ps = self.ps & 0b11111101;
    }

    fn unset_interrupt_disable(&mut self) {
        self.ps = self.ps & 0b11111011;
    }

    fn unset_decimal_mode(&mut self) {
        self.ps = self.ps & 0b11110111;
    }

    fn unset_break_command(&mut self) {
        self.ps = self.ps & 0b11101111;
    }

    fn unset_overflow_flag(&mut self) {
        self.ps = self.ps & 0b10111111;
    }

    fn unset_negative_flag(&mut self) {
        self.ps = self.ps & 0b01111111;
    }

    fn update_program_counter_n(&mut self, val: i16) {
        if val >= 0 {
            self.pc = self.pc.wrapping_add(val as u16);
        } else {
            self.pc = self.pc.wrapping_sub(val as u16);
        }
    }

    fn fetch_pc(&mut self) -> u16 {
        return self.pc;
    }

    fn fetch_a(&mut self) -> u8 {
        return self.a;
    }

    fn fetch_x(&mut self) -> u8 {
        return self.x;
    }

    fn fetch_y(&mut self) -> u8 {
        return self.y;
    }

    fn update_zero_and_negative_flags_u8(&mut self, bit: u8) {
        if bit == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
        if get_nth_bit_u8(bit, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn cpy(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(mode);
        let val = self.read_byte_from_memory(addr);
        let result = self.y - val;

        if self.y >= val {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }

        if self.y == val {
            self.set_zero_flag()
        } else {
            self.unset_zero_flag()
        }

        if get_nth_bit_u8(result, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);

        if self.x == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }

        if get_nth_bit_u8(self.x, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn lda(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(mode);
        let val = self.read_byte_from_memory(addr);
        self.a = val;
        self.update_zero_and_negative_flags_u8(self.a);
    }

    fn ldx(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(mode);
        let val = self.read_byte_from_memory(addr);
        self.x = val;
        self.update_zero_and_negative_flags_u8(self.x);
    }

    fn ldy(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(mode);
        let val = self.read_byte_from_memory(addr);
        self.y = val;
        self.update_zero_and_negative_flags_u8(self.y);
    }

    //todo: accumulator handling
    fn lsr(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(mode);
        let val = self.read_byte_from_memory(addr);
        let bit_0 = val & 1;
        let newval = val >> 1;
        self.write_byte_to_memory(addr, newval);
        self.update_zero_and_negative_flags_u8(newval)
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.update_zero_and_negative_flags_u8(self.x);
    }

    pub fn load_to_memory(&mut self, startAddr: u16, dataVec: Vec<u8>) {
        let dataVecIterator = dataVec.iter();
        let mut curAddr = startAddr;
        for data in dataVecIterator {
            self.write_byte_to_memory(curAddr, *data);
            curAddr += 1;
        }
    }

    pub fn start(&mut self, startAddr: u16) {
        self.pc = startAddr;
        loop {
            let pc = self.pc;
            let opcode = self.read_byte_from_memory(pc);
            match opcode {
                0x00 => {
                    return;
                }
                0xA0 => {
                    self.ldy(AddressingModes::Immediate);
                }
                //LDA
                0xA1 => {
                    self.lda(AddressingModes::IndirectX);
                }
                0xA5 => {
                    self.lda(AddressingModes::ZeroPage);
                }
                0xA9 => {
                    self.lda(AddressingModes::Immediate);
                }
                0xAD => {
                    self.lda(AddressingModes::Absolute);
                }
                0xB1 => {
                    self.lda(AddressingModes::IndirectY);
                }
                0xB5 => {
                    self.lda(AddressingModes::ZeroPageX);
                }
                0xB9 => {
                    self.lda(AddressingModes::AbsoluteY);
                }
                0xBD => {
                    self.lda(AddressingModes::AbsoluteX);
                }
                // LDX
                0xA2 => {
                    self.ldx(AddressingModes::Immediate);
                }
                0xA6 => {
                    self.ldx(AddressingModes::ZeroPage);
                }
                0xAE => {
                    self.ldx(AddressingModes::Absolute);
                }
                0xB6 => {
                    self.ldx(AddressingModes::ZeroPageY);
                }
                0xBE => {
                    self.ldx(AddressingModes::AbsoluteY);
                }
                // LDY
                0xA0 => {
                    self.ldy(AddressingModes::Immediate);
                }
                0xA4 => {
                    self.ldy(AddressingModes::ZeroPage);
                }
                0xAC => {
                    self.ldy(AddressingModes::Absolute);
                }
                0xB4 => {
                    self.ldy(AddressingModes::ZeroPageX);
                }
                0xBC => {
                    self.ldy(AddressingModes::AbsoluteX);
                }
                //TAX: implicit
                0xAA => {
                    self.tax();
                }
                //CPY: immediate
                0xC0 => {
                    self.cpy(AddressingModes::Immediate);
                }
                //INX: immediate
                0xe8 => {
                    self.inx();
                }
                _ => return,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::init();
        cpu.load_to_memory(0x8000, vec![0xa9, 0x05, 0x00]);
        cpu.start(0x8000);
        assert_eq!(cpu.a, 0x05);
        assert!(cpu.ps & 0b0000_0010 == 0b00);
        assert!(cpu.ps & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::init();
        cpu.load_to_memory(0x8000, vec![0xa9, 0x00, 0x00]);
        cpu.start(0x8000);
        assert!(cpu.ps & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::init();
        cpu.a = 10;
        cpu.load_to_memory(0x8000, vec![0xaa, 0x00]);
        cpu.start(0x8000);

        assert_eq!(cpu.x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::init();
        cpu.load_to_memory(0x8000, vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        cpu.start(0x8000);

        assert_eq!(cpu.x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::init();
        cpu.x = 0xff;
        cpu.load_to_memory(0x8000, vec![0xe8, 0xe8, 0x00]);
        cpu.start(0x8000);

        assert_eq!(cpu.x, 1)
    }

    #[test]
    fn test_memory_read() {
        let mut cpu = CPU::init();
        cpu.write_byte_to_memory(0xA1, 65);
        assert_eq!(cpu.read_byte_from_memory(0xA1), 65)
    }

    #[test]
    fn test_memory_write_u16() {
        let mut cpu = CPU::init();
        let addr = 0x8000;
        let val: u16 = 0xFFFF;
        cpu.write_2_bytes_to_memory(addr, val);
        assert_eq!(cpu.read_2_bytes_from_memory(addr), 0xFFFF)
    }
}
