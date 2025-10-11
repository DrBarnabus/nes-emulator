use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    pub ram: [u8; 2048], // 2KB internal RAM
    pub ppu: Rc<RefCell<Ppu>>,
    pub apu: Rc<RefCell<Apu>>,
    pub controller_1: Controller,
    pub controller_2: Controller,

    pub cartridge: Rc<RefCell<Cartridge>>,

    pub nmi_pending: bool,
    pub irq_pending: bool,
}

impl Bus {
    pub fn new(ppu: Rc<RefCell<Ppu>>, apu: Rc<RefCell<Apu>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            ram: [0; 2048],
            ppu,
            apu,
            controller_1: Controller::new(),
            controller_2: Controller::new(),

            cartridge,

            nmi_pending: false,
            irq_pending: false,
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_read(address & 0x2007),
            0x4000..=0x4017 => self.read_io(address),
            0x4018..=0x401F => 0, // Open bus
            0x4020..=0xFFFF => self.cartridge.borrow_mut().cpu_read(address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize] = value,
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_write(address & 0x2007, value),
            0x4000..=0x4017 => self.write_io(address, value),
            0x4018..=0x401F => {}
            0x4020..=0xFFFF => self.cartridge.borrow_mut().cpu_write(address, value),
        }
    }

    pub fn trigger_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn poll_nmi(&mut self) -> bool {
        let pending = self.nmi_pending;
        self.nmi_pending = false;
        pending
    }

    pub fn trigger_irq(&mut self) {
        self.irq_pending = true;
    }

    pub fn poll_irq(&mut self) -> bool {
        let pending = self.irq_pending;
        self.irq_pending = false;
        pending
    }

    fn read_io(&mut self, address: u16) -> u8 {
        match address {
            // OAM DMA
            0x4014 => 0, // Write-only, open bus

            // Controller 1 & 2
            0x4016 => self.controller_1.read(),
            0x4017 => self.controller_2.read(),

            // APU
            0x4000..=0x4017 => self.apu.borrow_mut().cpu_read(address),

            _ => unreachable!(),
        }
    }

    fn write_io(&mut self, address: u16, value: u8) {
        match address {
            // OAM DMA
            0x4014 => self.write_oam_dma(value),

            // Controller 1 & 2 Strobe
            0x4016 => {
                self.controller_1.write(value);
                self.controller_2.write(value);
            }

            // APU
            0x4000..=0x4017 => self.apu.borrow_mut().cpu_write(address, value),

            _ => unreachable!(),
        }
    }

    fn write_oam_dma(&mut self, value: u8) {
        let mut buffer = [0u8; 256];
        let high_byte = (value as u16) << 8;
        for i in 0..256 {
            buffer[i as usize] = self.read(high_byte + i);
        }

        self.ppu.borrow_mut().write_to_oam_dma(&buffer);
    }
}
