use crate::cpu::Cpu;

pub trait Mem {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn read_u16(&self, address: u16) -> u16 {
        let low_byte = self.read(address) as u16;
        let high_byte = self.read(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    fn write_u16(&mut self, address: u16, value: u16) {
        let high_byte = (value >> 8) as u8;
        let low_byte = (value & 0xFF) as u8;
        self.write(address, low_byte);
        self.write(address + 1, high_byte);
    }
}

impl Mem for Cpu {
    fn read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }
}
