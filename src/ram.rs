const START_SYS_RAM: u16 = 0x0000;
const END_SYS_RAM: u16 = 0x07FF;
const START_PPU_REGISTERS: u16 = 0x2000;
const END_PPU_REGISTERS: u16 = 0x2007;
const START_AUDIO_CONTROLLERS_REGISTERS: u16 = 0x4000;
const END_AUDIO_CONTROLLERS_REGISTERS: u16 = 0x4016;
const START_EXPANSION_MODULES: u16 = 0x5000;
const END_EXPANSION_MODULES: u16 = 0x5FFF;
const START_CARTRIDGE_ROM: u16 = 0x8000;
const END_CARTRIDGE_ROM: u16 = 0xFFFF;

#[derive(Debug)]
pub struct RAM {
    memory: [u8; 0xFFFF],
}

impl RAM {
    pub fn init() -> Self {
        return RAM {
            memory: [0; 0xFFFF],
        };
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }
}
