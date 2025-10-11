pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod controller;
pub mod cpu;
pub mod ppu;

use crate::apu::Apu;
use crate::ppu::Ppu;
use bus::Bus;
use cartridge::Cartridge;
use cpu::{Cpu, trace::trace};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let cartridge = Rc::new(RefCell::new(Cartridge::load("test_roms/nestest.nes").unwrap()));
    let ppu = Rc::new(RefCell::new(Ppu::new(Rc::clone(&cartridge))));
    let apu = Rc::new(RefCell::new(Apu::new()));
    let bus = Rc::new(RefCell::new(Bus::new(Rc::clone(&ppu), Rc::clone(&apu), Rc::clone(&cartridge))));
    let mut cpu = Cpu::new(Rc::clone(&bus));
    cpu.reset();

    cpu.pc = 0xC000;

    // Tick PPU for the reset cycles (7 CPU cycles = 21 PPU cycles)
    for _ in 0..21 {
        ppu.borrow_mut().tick();
    }

    loop {
        if cpu.halted {
            break;
        }

        println!("{}", trace(&mut cpu));

        let cpu_cycles = cpu.step();
        for _ in 0..cpu_cycles * 3 {
            let mut bus = bus.borrow_mut();
            ppu.borrow_mut().tick();

            if ppu.borrow_mut().poll_nmi() {
                bus.trigger_nmi();
            }
        }
    }

    let test_result = cpu.read(0x0002);
    let error_detail = cpu.read(0x0003);

    match test_result {
        0x00 => println!("Tests passed: ok"),
        test_result if error_detail != 0x00 => println!("Tests failed at test number: {:#02X}, error detail: {:#02X}", test_result, error_detail),
        test_result => println!("Tests failed at test number: {:#02X}", test_result),
    }
}
