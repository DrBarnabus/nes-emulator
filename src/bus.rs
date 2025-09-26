use crate::cpu::mem::Mem;
use crate::rom::Rom;

const RAM: u16 = 0x0000;
const RAM_MASK: u16 = 0x07FF;
const RAM_END: u16 = 0x1FFF;

const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MASK: u16 = 0x0007;
const PPU_REGISTERS_END: u16 = 0x3FFF;

const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus {
    ram: [u8; 0x800], // 2KB internal RAM
    rom: Rom,

    // Interrupt lines
    pub nmi_pending: bool,
    pub irq_pending: bool,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Self {
            ram: [0; 0x800],
            rom,

            nmi_pending: false,
            irq_pending: false,
        }
    }

    pub fn trigger_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn trigger_irq(&mut self) {
        self.irq_pending = true;
    }

    pub fn poll_nmi(&mut self) -> bool {
        let pending = self.nmi_pending;
        self.nmi_pending = false;
        pending
    }

    pub fn poll_irq(&mut self) -> bool {
        let pending = self.irq_pending;
        self.irq_pending = false;
        pending
    }

    fn read_prg_rom(&self, address: u16) -> u8 {
        let mut address = address - 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && address >= 0x4000 {
            address %= 0x4000;
        }

        self.rom.prg_rom[address as usize]
    }
}

impl Mem for Bus {
    fn read(&self, address: u16) -> u8 {
        match address {
            RAM..=RAM_END => self.ram[(address & RAM_MASK) as usize],
            PPU_REGISTERS..=PPU_REGISTERS_END => todo!("PPU is not supported yet, attempted to read {:#x}", (address & PPU_REGISTERS_MASK) as usize),
            PRG_ROM..=PRG_ROM_END => self.read_prg_rom(address),
            _ => {
                println!("Ignoring attempted memory access, attempted to read {:#x}", address);
                0
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            RAM..=RAM_END => self.ram[(address & RAM_MASK) as usize] = value,
            PPU_REGISTERS..=PPU_REGISTERS_END => todo!("PPU is not supported yet, attempted to write {:#x}", (address & PPU_REGISTERS_MASK) as usize),
            PRG_ROM..=PRG_ROM_END => panic!("Cannot write to PRG ROM, attempted to write {:#x}", address),
            _ => {
                println!("Ignoring attempted memory access, attempted to write {:#x}", address);
            }
        }
    }
}
