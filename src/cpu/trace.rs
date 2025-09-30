use crate::cpu::Cpu;
use crate::cpu::addressing::AddressingMode;
use crate::cpu::mem::Mem;
use crate::cpu::opcode::OPCODES_MAP;

pub fn trace(cpu: &mut Cpu) -> String {
    let opcode_pc = cpu.pc;
    let opcode = cpu.read(opcode_pc);
    let opcode = **OPCODES_MAP.get(&opcode).unwrap_or_else(|| panic!("Unknown opcode: {:#2X}", opcode));

    let mut hex_dump = vec![];
    hex_dump.push(opcode.opcode);

    let (memory_address, stored_value) = match opcode.mode {
        AddressingMode::Implied | AddressingMode::Accumulator => (0, 0),
        _ => {
            let address = cpu.get_operand_address(opcode.mode, opcode_pc + 1).0;
            let value = cpu.read_operand(opcode.mode, opcode_pc + 1).0;

            (address, value)
        }
    };

    let tmp = match opcode.size_bytes {
        1 if opcode.mode == AddressingMode::Accumulator => String::from("A "),
        2 if opcode.opcode == 0x00 => String::from(""),
        2 => {
            let address = cpu.read(opcode_pc + 1);
            hex_dump.push(address);

            match opcode.mode {
                AddressingMode::Immediate => format!("#${:02x}", address),
                AddressingMode::ZeroPage => format!("${:02x} = {:02x}", memory_address, stored_value),
                AddressingMode::ZeroPageX => format!("${:02x},X @ {:02x} = {:02x}", address, memory_address, stored_value),
                AddressingMode::ZeroPageY => format!("${:02x},Y @ {:02x} = {:02x}", address, memory_address, stored_value),
                AddressingMode::Relative => format!("${:04x}", (opcode_pc as usize + 2).wrapping_add((address as i8) as usize)),
                AddressingMode::IndirectX => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    address.wrapping_add(cpu.x),
                    memory_address,
                    stored_value
                ),
                AddressingMode::IndirectY => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    memory_address.wrapping_sub(cpu.y as u16),
                    memory_address,
                    stored_value
                ),
                _ => panic!(
                    "Unexpected addressing mode {:?} found for opcode {:02x} with 2 byte length",
                    opcode.mode, opcode.opcode
                ),
            }
        }
        3 => {
            let address_low = cpu.read(opcode_pc + 1);
            let address_high = cpu.read(opcode_pc + 2);
            hex_dump.push(address_low);
            hex_dump.push(address_high);

            let address = cpu.read_u16(opcode_pc + 1);

            match opcode.mode {
                AddressingMode::Indirect => {
                    let jmp_address = if address & 0x00FF == 0x00FF {
                        let low_byte = cpu.read(address);
                        let high_byte = cpu.read(address & 0xFF00);
                        (high_byte as u16) << 8 | (low_byte as u16)
                    } else {
                        cpu.read_u16(address)
                    };

                    format!("(${:04x}) = {:04x}", address, jmp_address)
                }
                AddressingMode::Absolute => {
                    if opcode.opcode == 0x4C || opcode.opcode == 0x20 {
                        format!("${:04x}", memory_address)
                    } else {
                        format!("${:04x} = {:02x}", memory_address, stored_value)
                    }
                }
                AddressingMode::AbsoluteX => format!("${:04x},X @ {:04x} = {:02x}", address, memory_address, stored_value),
                AddressingMode::AbsoluteY => format!("${:04x},Y @ {:04x} = {:02x}", address, memory_address, stored_value),
                _ => panic!(
                    "Unexpected addressing mode {:?} found for opcode {:02x} with 3 byte length",
                    opcode.mode, opcode.opcode
                ),
            }
        }
        _ => String::from(""),
    };

    let hex_str = hex_dump.iter().map(|z| format!("{:02x}", z)).collect::<Vec<String>>().join(" ");

    // replace mnemonic with that expected by nestest (might remove later)
    let mnemonic = match opcode.mnemonic {
        "SBC" if opcode.opcode == 0xEB => "*SBC",
        "NOP" if matches!(opcode.opcode, 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA) => "*NOP",
        "DOP" | "TOP" => "*NOP",
        "LAX" => "*LAX",
        "AAX" => "*SAX",
        "DCP" => "*DCP",
        "ISC" => "*ISB",
        "SLO" => "*SLO",
        "RLA" => "*RLA",
        "RRA" => "*RRA",
        "SRE" => "*SRE",
        mnemonic => mnemonic,
    };

    let asm_str = format!("{:04x}  {:8} {: >4} {}", opcode_pc, hex_str, mnemonic, tmp).trim().to_string();

    let bus = cpu.bus.borrow();
    let ppu_scanline = bus.ppu.scanline;
    let ppu_cycle = bus.ppu.cycle;

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} PPU:{:3},{:3} CYC:{}",
        asm_str, cpu.a, cpu.x, cpu.y, cpu.status, cpu.sp, ppu_scanline, ppu_cycle, cpu.cycles
    )
    .to_ascii_uppercase()
}
