mod mapper;

pub use mapper::Mapper;

use anyhow::{Context, Result, bail, ensure};
use mapper::{MappedRead, MappedWrite, Mapper000};

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // NES^Z
const PRG_ROM_BANK_SIZE: usize = 16384; // 16KB
const CHR_ROM_BANK_SIZE: usize = 8192; // 8KB
const PRG_RAM_SIZE: usize = 8192;
const CHR_RAM_SIZE: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenLower,
    SingleScreenUpper,
    FourScreen,
}

#[repr(C, packed)]
pub struct INesHeader {
    tag: [u8; 4],     // "NES" followed by MS-DOS EOF
    prg_rom_size: u8, // Size of PRG-ROM in 16KB units
    chr_rom_size: u8, // Size of CHR-ROM in 8KB units
    flags_6: u8,      // Lower nybble of mapper, mirroring, battery and trainer
    flags_7: u8,      // Upper nybble of mapper, version
    flags_8: u8,      // PRG-RAM size
    _unused: [u8; 7],
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_ram: Vec<u8>,
    pub mapper: Box<dyn Mapper>,
    pub mirroring: Mirroring,
}

impl Cartridge {
    pub fn load(path: &str) -> Result<Cartridge> {
        let data = std::fs::read(path).with_context(|| format!("Failed to read ROM file: {}", path))?;
        ensure!(data.len() >= 16, "ROM data is too small for expected iNES header");

        let header = unsafe { &*(data.as_ptr() as *const INesHeader) };
        ensure!(header.tag == NES_TAG, "Invalid iNES header");

        let mapper_number = (header.flags_7 & 0xF0) | (header.flags_6 >> 4);

        let mirroring = if header.flags_6 & 0x08 != 0 {
            Mirroring::FourScreen
        } else if header.flags_6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let prg_rom_size = (header.prg_rom_size as usize) * PRG_ROM_BANK_SIZE;
        let chr_rom_size = (header.chr_rom_size as usize) * CHR_ROM_BANK_SIZE;
        let has_trainer = header.flags_6 & 0x4 != 0;

        let mut offset = 16;
        if has_trainer {
            offset += 512;
        }

        ensure!(data.len() >= offset + prg_rom_size, "ROM data is too small for expected PRG-ROM");
        let prg_rom = data[offset..offset + prg_rom_size].to_vec();
        offset += prg_rom_size;

        let (chr_rom, chr_ram) = if chr_rom_size > 0 {
            ensure!(data.len() >= offset + chr_rom_size, "ROM data is too small for expected CHR-ROM");
            (data[offset..offset + chr_rom_size].to_vec(), vec![])
        } else {
            (vec![], vec![0; CHR_RAM_SIZE]) // CHR-RAM (8KB)
        };

        Ok(Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; PRG_RAM_SIZE], // PRG-RAM (8KB)
            chr_ram,
            mapper: Self::create_mapper(mapper_number, prg_rom_size, chr_rom_size)?,
            mirroring,
        })
    }

    /// CPU reads from $4020-$FFFF
    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match self.mapper.cpu_read(address) {
            MappedRead::Data(value) => value,
            MappedRead::PrgRom(address) if address < self.prg_rom.len() as u16 => self.prg_rom[address as usize],
            MappedRead::PrgRam(address) if address < self.prg_ram.len() as u16 => self.prg_ram[address as usize],
            _ => 0, // Open bus
        }
    }

    /// CPU writes to $4020-$FFFF
    pub fn cpu_write(&mut self, address: u16, value: u8) {
        if let MappedWrite::PrgRam(address) = self.mapper.cpu_write(address, value)
            && address < self.prg_ram.len() as u16
        {
            self.prg_ram[address as usize] = value;
        }
    }

    /// PPU reads from $0000-$1FFF
    pub fn ppu_read(&mut self, address: u16) -> u8 {
        let mapped_address = self.mapper.ppu_read(address);

        if !self.chr_rom.is_empty() && mapped_address < self.chr_rom.len() {
            self.chr_rom[mapped_address]
        } else if !self.chr_ram.is_empty() && mapped_address < self.chr_ram.len() {
            self.chr_ram[mapped_address]
        } else {
            0
        }
    }

    /// PPU writes to $0000-$1FFF (CHR-RAM only)
    pub fn ppu_write(&mut self, address: u16, value: u8) {
        if let Some(mapped_address) = self.mapper.ppu_write(address, value)
            && !self.chr_ram.is_empty()
            && mapped_address < self.chr_ram.len()
        {
            self.chr_ram[mapped_address] = value;
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring()
    }

    fn create_mapper(mapper_number: u8, prg_rom_size: usize, chr_rom_size: usize) -> Result<Box<dyn Mapper>> {
        match mapper_number {
            0 => Ok(Box::new(Mapper000::new(prg_rom_size, chr_rom_size))),
            _ => bail!("Unsupported mapper: {}", mapper_number),
        }
    }
}
