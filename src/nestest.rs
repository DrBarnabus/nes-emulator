pub mod bus;
pub mod cpu;
pub mod rom;

use bus::Bus;
use cpu::Cpu;
use cpu::trace::trace;
use rom::Rom;

fn main() {
    let rom = Rom::load("nestest.nes").unwrap();
    let bus = Bus::new(rom);
    let mut cpu = Cpu::new(bus);
    cpu.reset();

    cpu.pc = 0xC000;

    cpu.run(|cpu| {
        println!("{}", trace(cpu));
    })
}
