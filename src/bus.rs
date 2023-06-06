use crate::ram::RAM;

#[derive(Debug)]
pub struct BUS {
    ram: RAM,
}

impl BUS {
    pub fn init() -> Self {
        return BUS { ram: RAM::init() };
    }

    pub fn read_memory_byte(&self, addr: u16) -> u8 {
        return self.ram.read_u8(addr);
    }

    pub fn write_memory_byte(&mut self, addr: u16, val: u8) {
        return self.ram.write_u8(addr, val);
    }
}
