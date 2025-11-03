pub mod registers;
pub mod render;

use crate::cartridge::{Cartridge, Mirroring};
use crate::ppu::registers::addr::PpuAddrRegister;
use crate::ppu::registers::ctrl::PpuCtrlRegister;
use crate::ppu::registers::mask::PpuMaskRegister;
use crate::ppu::registers::scroll::PpuScrollRegister;
use crate::ppu::registers::status::PpuStatusRegister;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Ppu {
    pub cartridge: Rc<RefCell<Cartridge>>,

    pub ctrl: PpuCtrlRegister,
    pub mask: PpuMaskRegister,
    pub status: PpuStatusRegister,
    pub scroll: PpuScrollRegister,
    pub addr: PpuAddrRegister,

    oam_addr: u8,
    oam_data: [u8; 256],
    pub palette_table: [u8; 32],
    vram: [u8; 2048], // 8KB VRAM
    internal_data_buffer: u8,

    pub cycle: u16,    // 0-340
    pub scanline: u16, // 0-261
    pub frame: u64,

    nmi_occurred: bool,
    nmi_output: bool,
    nmi_previous: bool,

    // Latched scroll values for rendering (captured at end of vblank)
    pub render_scroll_x: u8,
    pub render_scroll_y: u8,
    pub render_nametable_addr: u16,
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cartridge,

            ctrl: PpuCtrlRegister::new(),
            mask: PpuMaskRegister::new(),
            status: PpuStatusRegister::new(),
            scroll: PpuScrollRegister::new(),
            addr: PpuAddrRegister::new(),

            oam_addr: 0,
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            vram: [0; 2048],
            internal_data_buffer: 0,

            cycle: 0,
            scanline: 0,
            frame: 0,

            nmi_occurred: false,
            nmi_output: false,
            nmi_previous: false,

            render_scroll_x: 0,
            render_scroll_y: 0,
            render_nametable_addr: 0x2000,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.cycle += 1;

        let mut new_frame = false;

        if self.cycle > 340 {
            if self.is_sprite_0_hit(self.cycle as usize) {
                self.status.insert(PpuStatusRegister::SPRITE_0_HIT);
            }

            self.cycle = 0;
            self.scanline += 1;

            if self.scanline == 241 {
                self.render_scroll_x = self.scroll.scroll_x;
                self.render_scroll_y = self.scroll.scroll_y;
                self.render_nametable_addr = self.ctrl.nametable_addr();

                self.status.remove(PpuStatusRegister::SPRITE_0_HIT);
                self.set_vblank();
            } else if self.scanline > 261 {
                self.scanline = 0;
                self.frame += 1;

                new_frame = true;
            }
        }

        if self.scanline == 261 && self.cycle == 1 {
            self.status.remove(PpuStatusRegister::SPRITE_OVERFLOW);
            self.status.remove(PpuStatusRegister::SPRITE_0_HIT);
            self.clear_vblank();
        }

        self.update_nmi_output();

        new_frame
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => self.read_status(),
            0x2004 => self.read_oam_data(),
            0x2007 => self.read_data(),
            _ => 0, // Open bus
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => self.write_to_ppu_ctrl(value),
            0x2001 => self.write_to_ppu_mask(value),
            0x2002 => {} // Read-only
            0x2003 => self.write_to_oam_addr(value),
            0x2004 => self.write_to_oam_data(value),
            0x2005 => self.write_to_ppu_scroll(value),
            0x2006 => self.write_to_ppu_addr(value),
            0x2007 => self.write_data(value),
            _ => unreachable!(),
        }
    }

    pub fn poll_nmi(&mut self) -> bool {
        let pending = !self.nmi_previous && self.nmi_output;
        self.nmi_previous = self.nmi_output;
        pending
    }

    fn read_status(&mut self) -> u8 {
        let result = self.status.bits();

        self.clear_vblank();
        self.addr.reset_latch();
        self.scroll.reset_latch();

        result
    }

    fn read_oam_data(&mut self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    fn read_data(&mut self) -> u8 {
        let address = self.addr.get();
        self.increment_vram_addr();

        match address {
            0..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.cartridge.borrow_mut().ppu_read(address);

                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_address(address) as usize];

                result
            }
            0x3000..=0x3EFF => unimplemented!("Address space 0x3000..0x3EFF is not expected to be used, attempted to read {:#x}", address),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.palette_table[((address - 0x10) - 0x3F00) as usize],
            0x3F00..=0x3FFF => self.palette_table[(address - 0x3F00) as usize],
            _ => panic!("Unexpected access to mirrored address space, attempted to read {:#x}", address),
        }
    }

    fn write_to_ppu_ctrl(&mut self, value: u8) {
        self.ctrl.update(value);
    }

    fn write_to_ppu_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    fn write_to_ppu_scroll(&mut self, value: u8) {
        self.scroll.update(value);
    }

    fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    fn write_data(&mut self, value: u8) {
        let address = self.addr.get();

        match address {
            0..=0x1FFF => self.cartridge.borrow_mut().ppu_write(address, value),
            0x2000..=0x2FFF => self.vram[self.mirror_vram_address(address) as usize] = value,
            0x3000..=0x3EFF => unimplemented!("Address space 0x3000..0x3EFF is not expected to be used, attempted to write {:#x}", address),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.palette_table[((address - 0x10) - 0x3F00) as usize] = value,
            0x3F00..=0x3FFF => {
                self.palette_table[(address - 0x3F00) as usize] = value;
            }
            _ => panic!("Unexpected access to mirrored address space, attempted to write {:#x}", address),
        }

        self.increment_vram_addr();
    }

    pub fn write_to_oam_dma(&mut self, value: &[u8; 256]) {
        for v in value.iter() {
            self.write_to_oam_data(*v);
        }
    }

    fn mirror_vram_address(&self, address: u16) -> u16 {
        let mirrored_vram = address & 0x2FFF;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;

        match (self.cartridge.borrow_mut().mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) | (Mirroring::Horizontal, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 2) => vram_index - 0x400,
            _ => vram_index,
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_add_increment());
    }

    fn set_vblank(&mut self) {
        self.status.insert(PpuStatusRegister::VBLANK_STARTED);
        self.nmi_occurred = true;
    }

    fn clear_vblank(&mut self) {
        self.status.remove(PpuStatusRegister::VBLANK_STARTED);
        self.nmi_occurred = false;
    }

    fn update_nmi_output(&mut self) {
        let nmi_enabled = self.ctrl.contains(PpuCtrlRegister::GENERATE_NMI);
        self.nmi_output = self.nmi_occurred && nmi_enabled;
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        (y == self.scanline as usize) && x <= cycle && self.mask.contains(PpuMaskRegister::SHOW_SPRITES)
    }
}
