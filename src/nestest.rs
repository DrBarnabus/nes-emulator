pub mod bus;
pub mod cpu;
pub mod rom;

use bus::Bus;
use cpu::{Cpu, mem::Mem, trace::trace};
use rom::Rom;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let rom = Rom::load("nestest.nes").unwrap();
    let bus = Rc::new(RefCell::new(Bus::new(rom)));
    let mut cpu = Cpu::new(bus);
    cpu.reset();

    cpu.pc = 0xC000;
    cpu.run(|cpu| {
        println!("{}", trace(cpu));
    });

    let test_result = cpu.read(0x0002);
    let error_detail = cpu.read(0x0003);

    match test_result {
        0x00 => println!("Tests passed: ok"),
        test_result if error_detail != 0x00 => println!("Tests failed at test number: {:#02X}, error detail: {:#02X}", test_result, error_detail),
        test_result => println!("Tests failed at test number: {:#02X}", test_result),
    }
}
