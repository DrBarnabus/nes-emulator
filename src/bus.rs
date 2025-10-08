use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    pub ram: [u8; 2048], // 2KB internal RAM
    pub ppu: Rc<RefCell<Ppu>>,
    pub controller_1: Controller,
    pub joypad_2: Controller,

    pub cartridge: Rc<RefCell<Cartridge>>,

    pub nmi_pending: bool,
    pub irq_pending: bool,
}

impl Bus {
    pub fn new(ppu: Rc<RefCell<Ppu>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            ram: [0; 2048],
            ppu,
            controller_1: Controller::new(),
            joypad_2: Controller::new(),

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
            // APU Pulse 1
            0x4000..=0x4003 => 0, // Write-only, open bus

            // APU Pulse 2
            0x4004..=0x4007 => 0, // Write-only, open bus

            // APU Triangle
            0x4008..=0x400B => 0, // Write-only, open bus

            // APU Noise
            0x400C..=0x400F => 0, // Write-only, open bus

            // APU DMC
            0x4010..=0x4013 => 0, // Write-only, open bus

            // OAM DMA
            0x4014 => 0, // Write-only, open bus

            // TODO: APU Status
            0x4015 => 0,

            // Controller 1
            0x4016 => self.controller_1.read(),

            // Controller 2
            0x4017 => self.joypad_2.read(),

            _ => unreachable!(),
        }
    }

    fn write_io(&mut self, address: u16, value: u8) {
        match address {
            // TODO: APU Pulse 1
            0x4000..=0x4003 => {}

            // TODO: APU Pulse 2
            0x4004..=0x4007 => {}

            // TODO: APU Triangle
            0x4008..=0x400B => {}

            // TODO: APU Noise
            0x400C..=0x400F => {}

            // TODO: APU DMC
            0x4010..=0x4013 => {}

            // OAM DMA
            0x4014 => self.write_oam_dma(value),

            // TODO: APU Status
            0x4015 => {}

            0x4016 => {
                self.controller_1.write(value);
                self.joypad_2.write(value);
            }

            // TODO: APU Frame Counter
            0x4017 => {}

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
