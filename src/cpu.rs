#[derive(Debug)]
pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub ps: u8,
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
    IndirectX,
    IndirectY,
    IndexedIndirect,
    IndirectIndexed,
}

fn get_nth_bit_u8(byte: u8, n: u8) -> u8 {
    return (byte >> n) & 1;
}

fn get_nth_bit_u16(byte: u16, n: u8) -> u16 {
    return (byte >> n) & 1;
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
        };
    }

    //TODO
    fn handle_addressing_mode(&self, mode: &AddressingModes) {
        // match {
        //     _ => println!("Unknown");
        // }
    }

    fn get_carry_flag(&mut self) -> u8 {
        return self.ps & 0b00000001;
    }

    fn get_zero_flag(&mut self) -> u8 {
        return self.ps & 0b00000010;
    }

    fn get_interrupt_disable(&mut self) -> u8 {
        return self.ps & 0b00000100;
    }

    fn get_decimal_mode(&mut self) -> u8 {
        return self.ps & 0b00001000;
    }

    fn get_break_command(&mut self) -> u8 {
        return self.ps & 0b00010000;
    }

    fn get_overflow_flag(&mut self) -> u8 {
        return self.ps & 0b01000000;
    }

    fn get_negative_flag(&mut self) -> u8 {
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

    fn increment_program_counter(&mut self) {
        self.pc += 1;
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

    fn cpy(&mut self, val: u8) {
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

    fn lda(&mut self, val: u8) {
        self.a = val;
        self.update_zero_and_negative_flags_u8(self.a);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.update_zero_and_negative_flags_u8(self.x);
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.pc = 0;
        loop {
            println!("{:?}", self);
            let opcode = program[self.pc as usize];
            self.increment_program_counter();

            match opcode {
                0x00 => {
                    return;
                }
                //LDA: immediate
                0xA9 => {
                    let param = program[self.pc as usize];
                    self.increment_program_counter();

                    self.lda(param);
                }
                //TAX: implicit
                0xAA => {
                    self.tax();
                }
                //CPY: immediate
                0xC0 => {
                    let param = program[self.pc as usize];
                    self.increment_program_counter();

                    self.cpy(param);
                }
                //INX: immediate
                0xe8 => {
                    self.inx();
                }
                _ => println!("Unknown"),
            }
        }
    }
}
