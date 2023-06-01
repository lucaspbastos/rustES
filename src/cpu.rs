#[derive(Debug)]
pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub ps: u8,
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

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.pc = 0;
        loop {
            println!("{:?}", self);
            let opcode = program[self.pc as usize];
            self.increment_program_counter();

            match opcode {
                0x00 => {
                    return;
                },
                _ => println!("Unknown"),
            }
        }
    }
}
