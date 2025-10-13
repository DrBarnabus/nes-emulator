use nes_emulator::cpu::Cpu;
use nes_emulator::{cartridge::Cartridge, emulator::Emulator};

pub fn load_test_rom(path: &str) -> Emulator {
    let cartridge = Cartridge::load(path).unwrap();
    Emulator::new(cartridge)
}

pub fn run_test(emulator: &mut Emulator) -> (bool, String) {
    emulator.reset();

    const MAX_FRAMES: u64 = 600; // Allow up to 600 frames (10 seconds) to run the test
    for _ in 0..MAX_FRAMES {
        emulator.run_frame();

        let byte1 = emulator.cpu.read(0x6001);
        let byte2 = emulator.cpu.read(0x6002);
        let byte3 = emulator.cpu.read(0x6003);

        if byte1 != 0xDE || byte2 != 0xB0 || byte3 != 0x61 {
            continue; // Allow another frame to run, magic bytes are not yet present
        }

        let status = emulator.cpu.read(0x6000);

        if status == 0x80 {
            continue; // Test still running
        } else if status == 0 {
            return (true, read_test_output(&mut emulator.cpu));
        } else {
            let output = read_test_output(&mut emulator.cpu);
            return (false, format!("FAILED - Code {}\n{}", status, output));
        }
    }

    (false, format!("FAILED - Test timed out after {} frames", MAX_FRAMES))
}

fn read_test_output(cpu: &mut Cpu) -> String {
    let mut output = String::new();

    for address in 0x6004..=0x6FFF {
        let byte = cpu.read(address);
        if byte == 0 {
            break;
        }

        output.push(byte as char);
    }

    output
}

#[test]
fn apu_len_ctr() {
    let mut emulator = load_test_rom("test_roms/apu_test/1-len_ctr.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "Length counter test failed", output);
}

#[test]
fn apu_len_table() {
    let mut emulator = load_test_rom("test_roms/apu_test/2-len_table.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "Length table test failed", output);
}

#[test]
fn apu_irq_flag() {
    let mut emulator = load_test_rom("test_roms/apu_test/3-irq_flag.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "IRQ flag test failed", output);
}

#[test]
#[ignore = "The APU does not yet handle this properly"]
fn apu_jitter() {
    let mut emulator = load_test_rom("test_roms/apu_test/4-jitter.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "Jitter test failed", output);
}

#[test]
fn apu_len_timing() {
    let mut emulator = load_test_rom("test_roms/apu_test/5-len_timing.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "Length timing test failed", output);
}

#[test]
#[ignore = "The APU does not yet handle this properly"]
fn app_irq_flag_timing() {
    let mut emulator = load_test_rom("test_roms/apu_test/6-irq_flag_timing.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "IRQ flag timing test failed", output);
}

#[test]
#[ignore = "The APU does not yet handle this properly"]
fn apu_dmc_basics() {
    let mut emulator = load_test_rom("test_roms/apu_test/7-dmc_basics.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "DMC Basics test failed", output);
}

#[test]
#[ignore = "The APU does not yet handle this properly"]
fn apu_dmc_rates() {
    let mut emulator = load_test_rom("test_roms/apu_test/8-dmc_rates.nes");
    let (success, output) = run_test(&mut emulator);

    assert!(success, "{}\n{}", "DMC rates test failed", output);
}
