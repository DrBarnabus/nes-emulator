use crate::cpu::mem::Mem;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::rom::Rom;

const RAM: u16 = 0x0000;
const RAM_MASK: u16 = 0x07FF;
const RAM_END: u16 = 0x1FFF;

const PPU_CTRL: u16 = 0x2000;
const PPU_MASK: u16 = 0x2001;
const PPU_STATUS: u16 = 0x2002;
const PPU_OAM_ADDR: u16 = 0x2003;
const PPU_OAM_DATA: u16 = 0x2004;
const PPU_SCROLL: u16 = 0x2005;
const PPU_ADDR: u16 = 0x2006;
const PPU_DATA: u16 = 0x2007;
const PPU_MIRROR: u16 = 0x2008;
const PPU_MIRROR_END: u16 = 0x3FFF;
const PPU_OAM_DMA: u16 = 0x4014;

const JOYPAD_1: u16 = 0x4016;
const JOYPAD_2: u16 = 0x4017;

const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus {
    ram: [u8; 0x800], // 2KB internal RAM
    prg_rom: Vec<u8>,

    pub ppu: Ppu,
    pub joypad_1: Joypad,

    // Interrupt lines
    pub nmi_pending: bool,
    pub irq_pending: bool,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.chr_rom, rom.screen_mirroring);

        Self {
            ram: [0; 0x800],
            prg_rom: rom.prg_rom,

            ppu,
            joypad_1: Joypad::new(),

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
        if self.prg_rom.len() == 0x4000 && address >= 0x4000 {
            address %= 0x4000;
        }

        self.prg_rom[address as usize]
    }
}

impl Mem for Bus {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            RAM..=RAM_END => self.ram[(address & RAM_MASK) as usize],
            PPU_CTRL | PPU_MASK | PPU_OAM_ADDR | PPU_SCROLL | PPU_ADDR | PPU_OAM_DMA => {
                // println!("Cannot read from write-only PPU address, attempted to read {:02x}", address);
                0
            }
            PPU_STATUS => self.ppu.read_status(),
            PPU_OAM_DATA => self.ppu.read_oam_data(),
            PPU_DATA => self.ppu.read_data(),
            PPU_MIRROR..=PPU_MIRROR_END => self.read(address & 0x2007),
            JOYPAD_1 => self.joypad_1.read(),
            JOYPAD_2 => {
                // println!("Ignoring attempted memory access for joypad 2, attempted to read {:02x}", address);
                0
            }
            PRG_ROM..=PRG_ROM_END => self.read_prg_rom(address),
            _ => {
                // println!("Ignoring attempted memory access, attempted to read {:02x}", address);
                0
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            RAM..=RAM_END => self.ram[(address & RAM_MASK) as usize] = value,
            PPU_CTRL => self.ppu.write_to_ppu_ctrl(value),
            PPU_MASK => self.ppu.write_to_ppu_mask(value),
            PPU_STATUS => panic!("Cannot write to PPU status, attempted to write {:02x}", address),
            PPU_OAM_ADDR => self.ppu.write_to_oam_addr(value),
            PPU_OAM_DATA => self.ppu.write_to_oam_data(value),
            PPU_SCROLL => self.ppu.write_to_ppu_scroll(value),
            PPU_ADDR => self.ppu.write_to_ppu_addr(value),
            PPU_DATA => self.ppu.write_data(value),
            PPU_MIRROR..=PPU_MIRROR_END => self.write(address & 0x2007, value),
            PPU_OAM_DMA => {
                let mut buffer = [0u8; 256];
                let high_byte = (value as u16) << 8;
                for i in 0..256 {
                    buffer[i as usize] = self.read(high_byte + i);
                }

                self.ppu.write_to_oam_dma(&buffer);
            }
            JOYPAD_1 => self.joypad_1.write(value),
            JOYPAD_2 => {
                // println!("Ignoring attempted memory access for joypad 2, attempted to write {:02x}", address);
            }
            PRG_ROM..=PRG_ROM_END => panic!("Cannot write to PRG ROM, attempted to write {:02x}", address),
            _ => {
                // println!("Ignoring attempted memory access, attempted to write {:02x}", address);
            }
        }
    }
}
