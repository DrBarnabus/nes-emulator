use super::{MappedRead, MappedWrite, Mapper, Mirroring};
use crate::cartridge::{CHR_ROM_BANK_SIZE, PRG_ROM_BANK_SIZE};

pub struct Mapper000 {
    prg_banks: u8,
    chr_banks: u8,
    mirroring: Mirroring,
}

impl Mapper000 {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize) -> Self {
        Self {
            prg_banks: (prg_rom_size / PRG_ROM_BANK_SIZE) as u8,
            chr_banks: (chr_rom_size / CHR_ROM_BANK_SIZE) as u8,
            mirroring: Mirroring::Horizontal,
        }
    }
}

impl Mapper for Mapper000 {
    fn cpu_read(&mut self, address: u16) -> MappedRead {
        match address {
            0x6000..=0x7FFF => MappedRead::Address(address - 0x6000),
            0x8000..=0xFFFF => MappedRead::Address(if self.prg_banks > 1 { address - 0x8000 } else { (address - 0x8000) & 0x3FFF }),
            _ => MappedRead::None,
        }
    }

    fn cpu_write(&mut self, address: u16, _value: u8) -> MappedWrite {
        match address {
            0x6000..=0x7FFF => MappedWrite::Address(address - 0x6000),
            _ => MappedWrite::None, // ROM is read-only
        }
    }

    fn ppu_read(&mut self, address: u16) -> MappedRead {
        MappedRead::Address(address)
    }

    fn ppu_write(&mut self, address: u16, _value: u8) -> MappedWrite {
        if self.chr_banks == 0 {
            MappedWrite::Address(address)
        } else {
            MappedWrite::None
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
