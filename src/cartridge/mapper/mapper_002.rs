use super::{MappedRead, MappedWrite, Mapper, Mirroring};
use crate::cartridge::{CHR_ROM_BANK_SIZE, PRG_ROM_BANK_SIZE};

pub struct Mapper002 {
    prg_banks: u8,
    chr_banks: u8,
    mirroring: Mirroring,
    prg_bank_select: u8, // Which 16KB PRG ROM bank is in lower slot
    bank_mask: u8,       // Mask for valid bank selection
}

impl Mapper002 {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize, mirroring: Mirroring) -> Self {
        let prg_banks = (prg_rom_size / PRG_ROM_BANK_SIZE) as u8;
        let chr_banks = (chr_rom_size / CHR_ROM_BANK_SIZE) as u8;
        let bank_mask = prg_banks.next_power_of_two() - 1;

        Self {
            prg_banks,
            chr_banks,
            mirroring,
            prg_bank_select: 0,
            bank_mask,
        }
    }
}

impl Mapper for Mapper002 {
    fn cpu_read(&mut self, address: u16) -> MappedRead {
        match address {
            0x8000..=0xBFFF => {
                let rom_addr = (self.prg_bank_select as usize * 0x4000) + (address - 0x8000) as usize;
                MappedRead::PrgRom(rom_addr)
            },
            0xC000..=0xFFFF => {
                let rom_addr = ((self.prg_banks - 1) as usize * 0x4000) + (address - 0xC000) as usize;
                MappedRead::PrgRom(rom_addr)
            },
            _ => MappedRead::None,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) -> MappedWrite {
        match address {
            0x8000..=0xFFFF => {
                self.prg_bank_select = value & self.bank_mask;
                MappedWrite::None
            },
            _ => MappedWrite::None,
        }
    }

    fn ppu_read(&mut self, address: u16) -> usize {
        address as usize
    }

    fn ppu_write(&mut self, address: u16, _value: u8) -> Option<usize> {
        if self.chr_banks == 0 { Some(address as usize) } else { None }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
