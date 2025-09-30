pub mod bus;
pub mod cpu;
pub mod ppu;
pub mod rom;

use bus::Bus;
use cpu::{Cpu, mem::Mem, trace::trace};
use rom::Rom;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let rom = Rom::load("test_roms/nestest.nes").unwrap();
    let bus = Rc::new(RefCell::new(Bus::new(rom)));
    let mut cpu = Cpu::new(Rc::clone(&bus));
    cpu.reset();

    cpu.pc = 0xC000;

    // Tick PPU for the reset cycles (7 CPU cycles = 21 PPU cycles)
    for _ in 0..21 {
        bus.borrow_mut().ppu.tick();
    }

    loop {
        if cpu.halted {
           break;
        }

        println!("{}", trace(&mut cpu));

        let cpu_cycles = cpu.step();
        for _ in 0..cpu_cycles * 3 {
            let mut bus = bus.borrow_mut();
            bus.ppu.tick();

            if bus.ppu.poll_nmi() {
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
