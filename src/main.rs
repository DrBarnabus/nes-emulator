pub mod cpu;
pub mod bus;

use cpu::Cpu;
use bus::Bus;

fn main() {
    let program = vec![
        0xA9, 0x42,       // LDA #$42 (load immediate value 0x42 into A)
        0x85, 0x00,       // STA $00 (store A into zero page address $00)
        0xA5, 0x00,       // LDA $00 (load from zero page address $00 into A)
        0xEA,             // NOP
        0x4C, 0x00, 0x80, // JMP $8000 (infinite loop)
    ];

    let mut bus = Bus::new();
    bus.load_program(&program, 0x8000);

    let mut cpu = Cpu::new(bus);
    cpu.pc = 0x8000;

    for i in 0..20 {
        println!("Cycle {}: PC=${:04X}, A=${:02X}, Status={:08b}", i, cpu.pc, cpu.a, cpu.status.bits());
        cpu.clock();
    }
}
