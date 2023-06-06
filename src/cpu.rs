use crate::bus::BUS;

#[derive(Debug)]
pub struct CPU {
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    ps: u8,
    bus: BUS,
}

#[derive(Debug, PartialEq, Eq)]
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
    Error,
}

const STACK_START: u16 = 0x0100;
const STACK_END: u16 = 0x01FF;

fn get_nth_bit_u8(byte: u8, n: u8) -> u8 {
    return (byte >> n) & 1;
}

fn assemble_2_bytes_le_u16(ms_byte: u8, ls_byte: u8) -> u16 {
    return ((ms_byte as u16) << 8) | (ls_byte as u16);
}

fn get_mode_from_opcode(opcode: u8) -> AddressingModes {
    match opcode {
        0x00 | 0x08 | 0x18 | 0x28 | 0x38 | 0x40 | 0x48 | 0x58 | 0x60 | 0x68 | 0x78 | 0x88
        | 0x8A | 0x98 | 0x9A | 0xA8 | 0xAA | 0xBA | 0xB8 | 0xC8 | 0xCA | 0xD8 | 0xE8 | 0xEA
        | 0xF8 => {
            return AddressingModes::Implicit;
        }
        0x01 | 0x21 | 0x41 | 0x61 | 0x81 | 0xA1 | 0xC1 | 0xE1 => {
            return AddressingModes::IndirectX;
        }
        0x05 | 0x06 | 0x65 | 0x24 | 0x25 | 0x26 | 0x45 | 0x46 | 0x66 | 0x85 | 0x86 | 0xA4
        | 0xA5 | 0xA6 | 0xC4 | 0xC5 | 0xC6 | 0xE4 | 0xE5 | 0xE6 => {
            return AddressingModes::ZeroPage;
        }
        0x09 | 0x29 | 0x49 | 0x69 | 0xA0 | 0xA2 | 0xA9 | 0xC0 | 0xC9 | 0xE0 | 0xE9 => {
            return AddressingModes::Immediate;
        }
        0x0A | 0x2A | 0x4A | 0x6A => {
            return AddressingModes::Accumulator;
        }
        0x0D | 0x0E | 0x6D | 0x20 | 0x2C | 0x2D | 0x2E | 0x4C | 0x4D | 0x4E | 0x6E | 0x8C
        | 0x8D | 0x8E | 0xAC | 0xAD | 0xAE | 0xCC | 0xCD | 0xCE | 0xEC | 0xED | 0xEE => {
            return AddressingModes::Absolute;
        }
        0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => {
            return AddressingModes::Relative;
        }
        0x11 | 0x31 | 0x51 | 0x71 | 0x91 | 0xB1 | 0xD1 | 0xF1 => {
            return AddressingModes::IndirectY;
        }
        0x15 | 0x16 | 0x35 | 0x36 | 0x55 | 0x56 | 0x75 | 0x76 | 0x94 | 0x95 | 0xB4 | 0xB5
        | 0xD5 | 0xD6 | 0xF5 | 0xF6 => {
            return AddressingModes::ZeroPageX;
        }
        0x1D | 0x1E | 0x7D | 0x3D | 0x3E | 0x5E | 0x5D | 0x7E | 0x9D | 0xBC | 0xBD | 0xDD
        | 0xDE | 0xFD | 0xFE => {
            return AddressingModes::AbsoluteX;
        }
        0x6C => {
            return AddressingModes::Indirect;
        }
        0x79 | 0x39 | 0x59 | 0x99 | 0xB9 | 0xD9 | 0xF9 => {
            return AddressingModes::AbsoluteY;
        }
        0xB6 | 0x96 => {
            return AddressingModes::ZeroPageY;
        }
        0x19 | 0xBE => {
            return AddressingModes::AbsoluteY;
        }
        _ => {
            println!("no mode for {}", opcode);
            return AddressingModes::Error;
        }
    }
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

    fn read_byte_from_memory(&mut self, addr: u16) -> u8 {
        self.update_program_counter_n(1);
        return self.bus.read_memory_byte(addr);
    }

    fn write_byte_to_memory(&mut self, addr: u16, val: u8) {
        self.update_program_counter_n(1);
        self.bus.write_memory_byte(addr, val);
    }

    fn read_2_bytes_from_memory(&mut self, addr: u16) -> u16 {
        let ls_byte = self.read_byte_from_memory(addr);
        let ms_byte = self.read_byte_from_memory(addr + 1);

        return assemble_2_bytes_le_u16(ms_byte, ls_byte);
    }

    fn write_2_bytes_to_memory(&mut self, addr: u16, val: u16) {
        let ls_byte = (val >> 8) as u8;
        let ms_byte = (val & 0xFF) as u8;

        self.write_byte_to_memory(addr, ms_byte);
        self.write_byte_to_memory(addr + 1, ls_byte);
    }

    fn pop_byte_from_stack(&mut self) -> u8 {
        let sp = self.get_sp();
        let val = self.read_byte_from_memory(STACK_START + (sp as u16));
        self.set_sp(sp + 1);
        return val;
    }

    fn push_byte_to_stack(&mut self, val: u8) {
        let sp = self.get_sp();
        self.write_byte_to_memory(STACK_START + (sp as u16), val);
        self.set_sp(sp - 1);
    }

    fn pop_2_bytes_from_stack(&mut self) -> u16 {
        let sp = self.get_sp();
        let val = self.read_2_bytes_from_memory(STACK_START + (sp as u16));
        self.set_sp(sp + 2);
        return val;
    }

    fn push_2_bytes_to_stack(&mut self, val: u16) {
        let sp = self.get_sp();
        self.write_2_bytes_to_memory(STACK_START + (sp as u16), val);
        self.set_sp(sp - 2);
    }

    // prob needs some work
    fn handle_addressing_mode(&mut self, mode: &AddressingModes) -> u16 {
        match mode {
            AddressingModes::Implicit => return 0,
            AddressingModes::Accumulator => return self.get_a() as u16,
            AddressingModes::Immediate => return self.get_pc(),
            AddressingModes::ZeroPage => {
                let pc = self.get_pc();
                return self.read_byte_from_memory(pc) as u16;
            }
            AddressingModes::ZeroPageX => {
                let x = self.get_x() as u16;
                let pc = self.get_pc();
                return self.read_byte_from_memory(pc.wrapping_add(x)) as u16;
            }
            AddressingModes::ZeroPageY => {
                let y = self.get_y() as u16;
                let pc = self.get_pc();
                return self.read_byte_from_memory(pc.wrapping_add(y)) as u16;
            }
            AddressingModes::Relative => {
                let pc = self.get_pc();
                let pc_2 = self.get_pc();
                return pc.wrapping_add_signed(self.read_byte_from_memory(pc_2) as i16);
            }
            AddressingModes::Absolute => {
                let pc = self.get_pc();
                return self.read_2_bytes_from_memory(pc);
            }
            AddressingModes::AbsoluteX => {
                let pc = self.get_pc();
                let x = self.get_x() as u16;
                return self.read_2_bytes_from_memory(pc.wrapping_add(x));
            }
            AddressingModes::AbsoluteY => {
                let pc = self.get_pc();
                let y = self.get_y() as u16;
                return self.read_2_bytes_from_memory(pc.wrapping_add(y));
            }
            AddressingModes::Indirect => {
                let pc = self.get_pc();
                let ls_byte_location = self.read_byte_from_memory(pc) as u16;
                return self.read_2_bytes_from_memory(ls_byte_location);
            }
            AddressingModes::IndirectX => {
                let pc = self.get_pc();
                let x = self.get_x() as u16;
                let ls_byte_location = self.read_byte_from_memory(pc.wrapping_add(x)) as u16;
                return self.read_2_bytes_from_memory(ls_byte_location);
            }
            AddressingModes::IndirectY => {
                let pc = self.get_pc();
                let ls_byte_location = self.read_byte_from_memory(pc) as u16;
                let y = self.get_y();
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

    fn run_instruction_function_from_opcode(&mut self, opcode: u8) {
        let mode = get_mode_from_opcode(opcode);
        match opcode {
            0x00 => {
                self.brk();
            }
            0x01 | 0x05 | 0x09 | 0x0D | 0x11 | 0x15 | 0x19 | 0x1D => {
                self.ora(mode);
            }
            0x06 | 0x0A | 0x0E | 0x16 | 0x1E => {
                self.asl(mode);
            }
            0x08 => {
                self.php();
            }
            0x10 => {
                self.bpl(mode);
            }
            0x18 => {
                self.clc();
            }
            0x20 => {
                self.jsr(mode);
            }
            0x21 | 0x25 | 0x29 | 0x2D | 0x31 | 0x35 | 0x39 | 0x3D => {
                self.and(mode);
            }
            0x24 | 0x2C => {
                self.bit(mode);
            }
            0x26 | 0x2A | 0x2E | 0x36 | 0x3E => {
                self.rol(mode);
            }
            0x28 => {
                self.plp();
            }
            0x30 => {
                self.bmi(mode);
            }
            0x38 => {
                self.sec();
            }
            0x40 => {
                self.rti();
            }
            0x41 | 0x45 | 0x49 | 0x4D | 0x51 | 0x55 | 0x59 | 0x5D => {
                self.eor(mode);
            }
            0x48 => {
                self.pha();
            }
            0x4A | 0x46 | 0x4E | 0x56 | 0x5E => {
                self.lsr(mode);
            }
            0x4C | 0x6C => {
                self.jmp(mode);
            }
            0x50 => {
                self.bvc(mode);
            }
            0x58 => {
                self.cli();
            }
            0x60 => {
                self.rts();
            }
            0x61 | 0x65 | 0x69 | 0x6D | 0x71 | 0x75 | 0x79 | 0x7D => {
                self.adc(mode);
            }
            0x66 | 0x6A | 0x6E | 0x76 | 0x7E => {
                self.ror(mode);
            }
            0x68 => {
                self.pla();
            }
            0x70 => {
                self.bvs(mode);
            }
            0x78 => {
                self.sei();
            }
            0x81 | 0x85 | 0x8D | 0x91 | 0x95 | 0x99 | 0x9D => {
                self.sta(mode);
            }
            0x84 | 0x8C | 0x94 => {
                self.sty(mode);
            }
            0x86 | 0x96 | 0x8E => {
                self.stx(mode);
            }
            0x88 => {
                self.dey();
            }
            0x8A => {
                self.txa();
            }
            0x90 => {
                self.bcc(mode);
            }
            0x98 => {
                self.tya();
            }
            0x9A => {
                self.txs();
            }
            0xA0 | 0xA4 | 0xAC | 0xB4 | 0xBC => {
                self.ldy(mode);
            }
            0xA1 | 0xA5 | 0xA9 | 0xAD | 0xB1 | 0xB5 | 0xB9 | 0xBD => {
                self.lda(mode);
            }
            0xA2 | 0xA6 | 0xAE | 0xB6 | 0xBE => {
                self.ldx(mode);
            }
            0xA8 => {
                self.tay();
            }
            0xAA => {
                self.tax();
            }
            0xB0 => {
                self.bcs(mode);
            }
            0xB8 => {
                self.clv();
            }
            0xBA => {
                self.tsx();
            }
            0xC0 | 0xC4 | 0xCC => {
                self.cpy(mode);
            }
            0xC1 | 0xC5 | 0xC9 | 0xCD | 0xD1 | 0xD5 | 0xD9 | 0xDD => {
                self.cmp(mode);
            }
            0xC6 | 0xD6 | 0xCE | 0xDE => {
                self.dec(mode);
            }
            0xCA => {
                self.dex();
            }
            0xC8 => {
                self.iny();
            }
            0xD0 => {
                self.bne(mode);
            }
            0xD8 => {
                self.cld();
            }
            0xE0 | 0xE4 | 0xEC => {
                self.cpx(mode);
            }
            0xE1 | 0xE5 | 0xE9 | 0xED | 0xF1 | 0xF5 | 0xF9 | 0xFD => {
                self.sbc(mode);
            }
            0xE6 | 0xEE | 0xF6 | 0xFE => {
                self.inc(mode);
            }
            0xE8 => {
                self.inx();
            }
            0xEA => {
                self.nop();
            }
            0xF0 => {
                self.beq(mode);
            }
            0xF8 => {
                self.sed();
            }
            _ => {
                println!("instr not found for {}", opcode);
            }
        }
    }

    fn get_pc(&mut self) -> u16 {
        return self.pc;
    }

    fn get_ps(&mut self) -> u8 {
        return self.ps;
    }

    fn get_sp(&mut self) -> u8 {
        return self.sp;
    }

    fn get_a(&mut self) -> u8 {
        return self.a;
    }

    fn get_x(&mut self) -> u8 {
        return self.x;
    }

    fn get_y(&mut self) -> u8 {
        return self.y;
    }

    fn get_carry_flag(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b00000001, 0);
    }

    fn get_zero_flag(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b00000010, 1);
    }

    fn get_interrupt_disable(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b00000100, 2);
    }

    fn get_decimal_mode(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b00001000, 3);
    }

    fn get_break_command(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b00110000, 5);
    }

    fn get_overflow_flag(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b01000000, 6);
    }

    fn get_negative_flag(&mut self) -> u8 {
        return get_nth_bit_u8(self.get_ps() & 0b10000000, 7);
    }

    fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }

    fn set_ps(&mut self, val: u8) {
        self.ps = val;
    }

    fn set_sp(&mut self, val: u8) {
        self.sp = val;
    }

    fn set_a(&mut self, val: u8) {
        self.a = val;
    }

    fn set_x(&mut self, val: u8) {
        self.x = val;
    }

    fn set_y(&mut self, val: u8) {
        self.y = val;
    }

    fn set_carry_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b00000001);
    }

    fn set_zero_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b00000010);
    }

    fn set_interrupt_disable(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b00000100);
    }

    fn set_decimal_mode(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b00001000);
    }

    fn set_break_command(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b00110000);
    }

    fn set_overflow_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b01000000);
    }

    fn set_negative_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps | 0b10000000);
    }

    fn unset_carry_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b11111110);
    }

    fn unset_zero_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b11111101);
    }

    fn unset_interrupt_disable(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b11111011);
    }

    fn unset_decimal_mode(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b11110111);
    }

    fn unset_break_command(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b11001111);
    }

    fn unset_overflow_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b10111111);
    }

    fn unset_negative_flag(&mut self) {
        let ps = self.get_ps();
        self.set_ps(ps & 0b01111111);
    }

    fn update_program_counter_n(&mut self, val: i8) {
        //TODO: prob needs work
        let pc = self.get_pc();
        if val >= 0 {
            self.set_pc(pc.wrapping_add(val as u16));
        } else {
            self.set_pc(pc.wrapping_sub(val as u16));
        }
    }

    fn update_zero_and_negative_flags_u8(&mut self, val: u8) {
        if val == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
        if get_nth_bit_u8(val, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn adc(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();
        let carry_flag = self.get_carry_flag();
        let result = a + val + carry_flag;

        self.set_a(result);
        //TODO: fix logic for overflow
        // let overflow =
        // if overflow == 1 {
        //     unset_carry_flag();
        //     set_overflow_flag();
        // } else {
        //     set_carry_flag();
        //     unset_overflow_flag();
        // }
        self.update_zero_and_negative_flags_u8(result);
    }

    fn and(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();
        let result = a & val;

        self.set_a(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn asl(&mut self, mode: AddressingModes) {
        let val;
        match mode {
            AddressingModes::Accumulator => val = self.get_a(),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                val = self.read_byte_from_memory(addr);
            }
        }
        let bit_7 = get_nth_bit_u8(val, 7);
        let result = val << 1;

        match mode {
            AddressingModes::Accumulator => self.set_a(result),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                self.write_byte_to_memory(addr, result);
            }
        }

        let a = self.get_a();
        if bit_7 == 1 {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }
        // TODO: check if always checking a? or check result
        if a == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
        if get_nth_bit_u8(result, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn bcc(&mut self, mode: AddressingModes) {
        let carry_flag = self.get_carry_flag();
        if carry_flag == 0 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn bcs(&mut self, mode: AddressingModes) {
        let carry_flag = self.get_carry_flag();
        if carry_flag == 1 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn beq(&mut self, mode: AddressingModes) {
        let zero_flag = self.get_zero_flag();
        if zero_flag == 1 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn bit(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();

        let result = a & val;
        let bit_7 = get_nth_bit_u8(val, 7);
        let bit_6 = get_nth_bit_u8(val, 6);

        if result == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
        if bit_7 == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
        if bit_6 == 1 {
            self.set_overflow_flag();
        } else {
            self.unset_overflow_flag();
        }
    }

    fn bmi(&mut self, mode: AddressingModes) {
        let negative_flag = self.get_negative_flag();
        if negative_flag == 1 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn bne(&mut self, mode: AddressingModes) {
        let zero_flag = self.get_zero_flag();
        if zero_flag == 0 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn bpl(&mut self, mode: AddressingModes) {
        let negative_flag = self.get_negative_flag();
        if negative_flag == 0 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn brk(&mut self) {
        //TODO: handle brk
        let pc = self.get_pc();
        let ps = self.get_ps();
        // let irq = self.read_2_bytes_from_memory(0xFFFE);

        // self.push_2_bytes_to_stack(pc);
        // self.push_byte_to_stack(ps);
        //self.set_pc(irq);
        self.set_break_command();
    }

    fn bvc(&mut self, mode: AddressingModes) {
        let overflow_flag = self.get_overflow_flag();
        if overflow_flag == 0 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn bvs(&mut self, mode: AddressingModes) {
        let overflow_flag = self.get_overflow_flag();
        if overflow_flag == 1 {
            let addr = self.handle_addressing_mode(&mode);
            let val = self.read_byte_from_memory(addr) as i8;
            let pc = self.get_pc();
            //TODO: check logic
            self.set_pc(pc.wrapping_add(val as u16));
        }
    }

    fn clc(&mut self) {
        self.unset_carry_flag();
    }

    fn cld(&mut self) {
        self.unset_decimal_mode();
    }

    fn cli(&mut self) {
        self.unset_interrupt_disable();
    }

    fn clv(&mut self) {
        self.unset_overflow_flag();
    }

    fn cmp(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();
        let result = a - val;

        if a >= val {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }

        if a == val {
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

    fn cpx(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let x = self.get_x();
        let result = x.wrapping_sub(val);

        if x >= val {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }

        if x == val {
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

    fn cpy(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let y = self.get_y();
        let result = y.wrapping_sub(val);

        if y >= val {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }

        if y == val {
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

    fn dec(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let result = val.wrapping_sub(val);
        self.write_byte_to_memory(addr, result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn dex(&mut self) {
        let x = self.get_x();
        let result = x.wrapping_sub(1);
        self.set_x(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn dey(&mut self) {
        let y = self.get_y();
        let result = y.wrapping_sub(1);
        self.set_y(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn eor(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();
        let result = a ^ val;
        self.set_a(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn inc(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let result = val.wrapping_add(1);
        self.write_byte_to_memory(addr, result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn inx(&mut self) {
        let x = self.get_x();
        let result = x.wrapping_add(1);
        self.set_x(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn iny(&mut self) {
        let y = self.get_y();
        let result = y.wrapping_add(1);
        self.set_y(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn jmp(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        self.set_pc(addr);
    }

    fn jsr(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        //TODO: check this
        self.push_2_bytes_to_stack(addr.wrapping_sub(1));
        self.set_pc(addr);
    }
    fn lda(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        self.set_a(val);
        self.update_zero_and_negative_flags_u8(val);
    }

    fn ldx(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        self.set_x(val);
        self.update_zero_and_negative_flags_u8(val);
    }

    fn ldy(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        self.y = val;
        self.update_zero_and_negative_flags_u8(self.y);
    }

    fn lsr(&mut self, mode: AddressingModes) {
        let val;
        match mode {
            AddressingModes::Accumulator => val = self.get_a(),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                val = self.read_byte_from_memory(addr);
            }
        }
        let bit_0 = val & 1;
        let newval = val >> 1;

        if mode == AddressingModes::Accumulator {
            self.set_a(newval);
        } else {
            let addr = self.handle_addressing_mode(&mode);
            self.write_byte_to_memory(addr, newval);
        }

        if bit_0 == 1 {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }

        self.update_zero_and_negative_flags_u8(newval);
    }

    fn nop(&mut self) {
        self.update_program_counter_n(1);
    }

    fn ora(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();
        let result = a | val;
        self.set_a(result);
        self.update_zero_and_negative_flags_u8(result);
    }

    fn pha(&mut self) {
        let a = self.get_a();
        self.push_byte_to_stack(a);
    }

    fn php(&mut self) {
        let ps = self.get_ps();
        self.push_byte_to_stack(ps);
    }

    fn pla(&mut self) {
        let val_stack = self.pop_byte_from_stack();
        self.set_a(val_stack);
        self.update_zero_and_negative_flags_u8(val_stack);
    }

    fn plp(&mut self) {
        let val_stack = self.pop_byte_from_stack();
        self.set_ps(val_stack);
    }

    fn rol(&mut self, mode: AddressingModes) {
        let val;
        match mode {
            AddressingModes::Accumulator => val = self.get_a(),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                val = self.read_byte_from_memory(addr);
            }
        }
        let bit_7 = get_nth_bit_u8(val, 7);
        let carry_flag = self.get_carry_flag();
        let result = (val << 1).wrapping_add(carry_flag);

        match mode {
            AddressingModes::Accumulator => self.set_a(result),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                self.write_byte_to_memory(addr, result);
            }
        }

        let a = self.get_a();
        if bit_7 == 1 {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }
        // TODO: check if always checking a? or check result
        if a == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
        if get_nth_bit_u8(result, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn ror(&mut self, mode: AddressingModes) {
        let val;
        match mode {
            AddressingModes::Accumulator => val = self.get_a(),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                val = self.read_byte_from_memory(addr);
            }
        }
        let bit_0 = get_nth_bit_u8(val, 0);
        let carry_flag = self.get_carry_flag();
        let result = (val >> 1) | (carry_flag << 7);

        match mode {
            AddressingModes::Accumulator => self.set_a(result),
            _ => {
                let addr = self.handle_addressing_mode(&mode);
                self.write_byte_to_memory(addr, result);
            }
        }

        let a = self.get_a();
        if bit_0 == 1 {
            self.set_carry_flag();
        } else {
            self.unset_carry_flag();
        }
        // TODO: check if always checking a? or check result
        if a == 0 {
            self.set_zero_flag();
        } else {
            self.unset_zero_flag();
        }
        if get_nth_bit_u8(result, 7) == 1 {
            self.set_negative_flag();
        } else {
            self.unset_negative_flag();
        }
    }

    fn rti(&mut self) {
        let ps_stack = self.pop_byte_from_stack();
        let pc_stack = self.pop_2_bytes_from_stack();

        self.set_ps(ps_stack);
        self.set_pc(pc_stack);
    }

    fn rts(&mut self) {
        let pc_stack = self.pop_2_bytes_from_stack();

        self.set_pc(pc_stack.wrapping_sub(1));
    }

    fn sbc(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let val = self.read_byte_from_memory(addr);
        let a = self.get_a();
        let carry_flag = self.get_carry_flag();
        // TODO: fix logic
        let result = a - val - (1 - carry_flag);
        self.set_a(result);
        // let overflow =
        // if overflow == 1 {
        //     unset_carry_flag();
        //     set_overflow_flag();
        // } else {
        //     set_carry_flag();
        //     unset_overflow_flag();
        // }
        self.update_zero_and_negative_flags_u8(result);
    }

    fn sec(&mut self) {
        self.set_carry_flag();
    }

    fn sed(&mut self) {
        self.set_decimal_mode();
    }

    fn sei(&mut self) {
        self.set_interrupt_disable();
    }

    fn sta(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let a = self.get_a();
        self.write_byte_to_memory(addr, a);
    }

    fn stx(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let x = self.get_x();
        self.write_byte_to_memory(addr, x);
    }

    fn sty(&mut self, mode: AddressingModes) {
        let addr = self.handle_addressing_mode(&mode);
        let y = self.get_y();
        self.write_byte_to_memory(addr, y);
    }

    fn tax(&mut self) {
        let a = self.get_a();
        self.set_x(a);
        self.update_zero_and_negative_flags_u8(a);
    }

    fn tay(&mut self) {
        let a = self.get_a();
        self.set_y(a);
        self.update_zero_and_negative_flags_u8(a);
    }

    fn tsx(&mut self) {
        let sp = self.get_sp();
        self.set_x(sp);
        self.update_zero_and_negative_flags_u8(sp);
    }

    fn txa(&mut self) {
        let x = self.get_x();
        self.set_a(x);
        self.update_zero_and_negative_flags_u8(x);
    }

    fn txs(&mut self) {
        let x = self.get_x();
        self.set_sp(x);
    }

    fn tya(&mut self) {
        let y = self.get_y();
        self.set_a(y);
        self.update_zero_and_negative_flags_u8(y);
    }

    pub fn load_to_memory(&mut self, start_addr: u16, data_vec: Vec<u8>) {
        let data_vec_iterator = data_vec.iter();
        let mut cur_addr = start_addr;
        for data in data_vec_iterator {
            self.write_byte_to_memory(cur_addr, *data);
            cur_addr = cur_addr.wrapping_add(1);
            println!("adding data");
        }
    }

    pub fn start(&mut self, start_addr: u16) {
        self.pc = start_addr;
        loop {
            if self.get_break_command() == 1 {
                return;
            }
            let pc = self.pc;
            let opcode = self.read_byte_from_memory(pc);
            self.run_instruction_function_from_opcode(opcode);
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
        cpu.load_to_memory(0x8000, vec![0xa9, 0x00]);
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
        let val: u16 = 0xFF8A;
        cpu.write_2_bytes_to_memory(addr, val);
        assert_eq!(cpu.read_2_bytes_from_memory(addr), val)
    }

    #[test]
    fn test_lda_and_sta() {
        let mut cpu = CPU::init();
        let addr = 0x8000;
        cpu.load_to_memory(addr, vec![0xa9, 0xc0, 0x85, 0xe8]);
        cpu.start(addr);

        assert_eq!(cpu.read_byte_from_memory(0xe8), 0xc0);
    }

    #[test]
    fn test_ldx_and_stx() {
        let mut cpu = CPU::init();
        let addr = 0x8000;
        cpu.load_to_memory(addr, vec![0xa2, 0xc0, 0x86, 0xe8]);
        cpu.start(addr);

        assert_eq!(cpu.read_byte_from_memory(0xe8), 0xc0);
    }
}
