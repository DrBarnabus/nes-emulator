use super::addressing::AddressingMode;
use super::instructions::Instruction;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct Opcode {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub size_bytes: u8,
    pub cycles: u8,
    pub additional_cycle_on_page_cross: bool,
    pub additional_cycle_on_branch_taken: bool,
}

impl Opcode {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        opcode: u8,
        mnemonic: &'static str,
        instruction: Instruction,
        mode: AddressingMode,
        size_bytes: u8,
        cycles: u8,
        additional_cycle_on_page_cross: bool,
        additional_cycle_on_branch_taken: bool,
    ) -> Opcode {
        Opcode {
            opcode,
            mnemonic,
            instruction,
            mode,
            size_bytes,
            cycles,
            additional_cycle_on_page_cross,
            additional_cycle_on_branch_taken,
        }
    }
}

lazy_static! {
    pub static ref OPCODES: Vec<Opcode> = vec![
        // LDA (Load Accumulator)
        Opcode::new(0xA9, "LDA", Instruction::LDA, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xA5, "LDA", Instruction::LDA, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xB5, "LDA", Instruction::LDA, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0xAD, "LDA", Instruction::LDA, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0xBD, "LDA", Instruction::LDA, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0xB9, "LDA", Instruction::LDA, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0xA1, "LDA", Instruction::LDA, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0xB1, "LDA", Instruction::LDA, AddressingMode::IndirectY, 2, 5, true, false),

        // LDX (Load X Register)
        Opcode::new(0xA2, "LDX", Instruction::LDX, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xA6, "LDX", Instruction::LDX, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xB6, "LDX", Instruction::LDX, AddressingMode::ZeroPageY, 2, 4, false, false),
        Opcode::new(0xAE, "LDX", Instruction::LDX, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0xBE, "LDX", Instruction::LDX, AddressingMode::AbsoluteY, 3, 4, true, false),

        // LDY (Load Y Register)
        Opcode::new(0xA0, "LDY", Instruction::LDY, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xA4, "LDY", Instruction::LDY, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xB4, "LDY", Instruction::LDY, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0xAC, "LDY", Instruction::LDY, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0xBC, "LDY", Instruction::LDY, AddressingMode::AbsoluteX, 3, 4, true, false),

        // STA (Store Accumulator)
        Opcode::new(0x85, "STA", Instruction::STA, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x95, "STA", Instruction::STA, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0x8D, "STA", Instruction::STA, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0x9D, "STA", Instruction::STA, AddressingMode::AbsoluteX, 3, 5, false, false),
        Opcode::new(0x99, "STA", Instruction::STA, AddressingMode::AbsoluteY, 3, 5, false, false),
        Opcode::new(0x81, "STA", Instruction::STA, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0x91, "STA", Instruction::STA, AddressingMode::IndirectY, 2, 6, false, false),

        // STX (Store X Register)
        Opcode::new(0x86, "STX", Instruction::STX, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x96, "STX", Instruction::STX, AddressingMode::ZeroPageY, 2, 4, false, false),
        Opcode::new(0x8E, "STX", Instruction::STX, AddressingMode::Absolute, 3, 4, false, false),

        // STY (Store Y Register)
        Opcode::new(0x84, "STY", Instruction::STY, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x94, "STY", Instruction::STY, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0x8C, "STY", Instruction::STY, AddressingMode::Absolute, 3, 4, false, false),

        // Register Transfers
        Opcode::new(0xAA, "TAX", Instruction::TAX, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0xA8, "TAY", Instruction::TAY, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x8A, "TXA", Instruction::TXA, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x98, "TYA", Instruction::TYA, AddressingMode::Implied, 1, 2, false, false),

        // Stack Operations
        Opcode::new(0xBA, "TSX", Instruction::TSX, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x9A, "TXS", Instruction::TXS, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x48, "PHA", Instruction::PHA, AddressingMode::Implied, 1, 3, false, false),
        Opcode::new(0x08, "PHP", Instruction::PHP, AddressingMode::Implied, 1, 3, false, false),
        Opcode::new(0x68, "PLA", Instruction::PLA, AddressingMode::Implied, 1, 4, false, false),
        Opcode::new(0x28, "PLP", Instruction::PLP, AddressingMode::Implied, 1, 4, false, false),

        // Logical Operations
        Opcode::new(0x29, "AND", Instruction::AND, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0x25, "AND", Instruction::AND, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x35, "AND", Instruction::AND, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0x2D, "AND", Instruction::AND, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0x3D, "AND", Instruction::AND, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0x39, "AND", Instruction::AND, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0x21, "AND", Instruction::AND, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0x31, "AND", Instruction::AND, AddressingMode::IndirectY, 2, 5, true, false),

        Opcode::new(0x49, "EOR", Instruction::EOR, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0x45, "EOR", Instruction::EOR, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x55, "EOR", Instruction::EOR, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0x4D, "EOR", Instruction::EOR, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0x5D, "EOR", Instruction::EOR, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0x59, "EOR", Instruction::EOR, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0x41, "EOR", Instruction::EOR, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0x51, "EOR", Instruction::EOR, AddressingMode::IndirectY, 2, 5, true, false),

        Opcode::new(0x09, "ORA", Instruction::ORA, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0x05, "ORA", Instruction::ORA, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x15, "ORA", Instruction::ORA, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0x0D, "ORA", Instruction::ORA, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0x1D, "ORA", Instruction::ORA, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0x19, "ORA", Instruction::ORA, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0x01, "ORA", Instruction::ORA, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0x11, "ORA", Instruction::ORA, AddressingMode::IndirectY, 2, 5, true, false),

        Opcode::new(0x24, "BIT", Instruction::BIT, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x2C, "BIT", Instruction::BIT, AddressingMode::Absolute, 3, 4, false, false),

        // Arithmetic Operations
        Opcode::new(0x69, "ADC", Instruction::ADC, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0x65, "ADC", Instruction::ADC, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0x75, "ADC", Instruction::ADC, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0x6D, "ADC", Instruction::ADC, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0x7D, "ADC", Instruction::ADC, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0x79, "ADC", Instruction::ADC, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0x61, "ADC", Instruction::ADC, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0x71, "ADC", Instruction::ADC, AddressingMode::IndirectY, 2, 5, true, false),

        Opcode::new(0xE9, "SBC", Instruction::SBC, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xE5, "SBC", Instruction::SBC, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xF5, "SBC", Instruction::SBC, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0xED, "SBC", Instruction::SBC, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0xFD, "SBC", Instruction::SBC, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0xF9, "SBC", Instruction::SBC, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0xE1, "SBC", Instruction::SBC, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0xF1, "SBC", Instruction::SBC, AddressingMode::IndirectY, 2, 5, true, false),

        Opcode::new(0xC9, "CMP", Instruction::CMP, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xC5, "CMP", Instruction::CMP, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xD5, "CMP", Instruction::CMP, AddressingMode::ZeroPageX, 2, 4, false, false),
        Opcode::new(0xCD, "CMP", Instruction::CMP, AddressingMode::Absolute, 3, 4, false, false),
        Opcode::new(0xDD, "CMP", Instruction::CMP, AddressingMode::AbsoluteX, 3, 4, true, false),
        Opcode::new(0xD9, "CMP", Instruction::CMP, AddressingMode::AbsoluteY, 3, 4, true, false),
        Opcode::new(0xC1, "CMP", Instruction::CMP, AddressingMode::IndirectX, 2, 6, false, false),
        Opcode::new(0xD1, "CMP", Instruction::CMP, AddressingMode::IndirectY, 2, 5, true, false),

        Opcode::new(0xE0, "CPX", Instruction::CPX, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xE4, "CPX", Instruction::CPX, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xEC, "CPX", Instruction::CPX, AddressingMode::Absolute, 3, 4, false, false),

        Opcode::new(0xC0, "CPY", Instruction::CPY, AddressingMode::Immediate, 2, 2, false, false),
        Opcode::new(0xC4, "CPY", Instruction::CPY, AddressingMode::ZeroPage, 2, 3, false, false),
        Opcode::new(0xCC, "CPY", Instruction::CPY, AddressingMode::Absolute, 3, 4, false, false),

        // Increments & Decrements
        Opcode::new(0xE6, "INC", Instruction::INC, AddressingMode::ZeroPage, 2, 5, false, false),
        Opcode::new(0xF6, "INC", Instruction::INC, AddressingMode::ZeroPageX, 2, 6, false, false),
        Opcode::new(0xEE, "INC", Instruction::INC, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0xFE, "INC", Instruction::INC, AddressingMode::AbsoluteX, 3, 7, false, false),

        Opcode::new(0xE8, "INX", Instruction::INX, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0xC8, "INY", Instruction::INY, AddressingMode::Implied, 1, 2, false, false),

        Opcode::new(0xC6, "DEC", Instruction::DEC, AddressingMode::ZeroPage, 2, 5, false, false),
        Opcode::new(0xD6, "DEC", Instruction::DEC, AddressingMode::ZeroPageX, 2, 6, false, false),
        Opcode::new(0xCE, "DEC", Instruction::DEC, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0xDE, "DEC", Instruction::DEC, AddressingMode::AbsoluteX, 3, 7, false, false),

        Opcode::new(0xCA, "DEX", Instruction::DEX, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x88, "DEY", Instruction::DEY, AddressingMode::Implied, 1, 2, false, false),

        // Shifts
        Opcode::new(0x0A, "ASL", Instruction::ASL, AddressingMode::Accumulator, 1, 2, false, false),
        Opcode::new(0x06, "ASL", Instruction::ASL, AddressingMode::ZeroPage, 2, 5, false, false),
        Opcode::new(0x16, "ASL", Instruction::ASL, AddressingMode::ZeroPageX, 2, 6, false, false),
        Opcode::new(0x0E, "ASL", Instruction::ASL, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0x1E, "ASL", Instruction::ASL, AddressingMode::AbsoluteX, 3, 7, false, false),

        Opcode::new(0x4A, "LSR", Instruction::LSR, AddressingMode::Accumulator, 1, 2, false, false),
        Opcode::new(0x46, "LSR", Instruction::LSR, AddressingMode::ZeroPage, 2, 5, false, false),
        Opcode::new(0x56, "LSR", Instruction::LSR, AddressingMode::ZeroPageX, 2, 6, false, false),
        Opcode::new(0x4E, "LSR", Instruction::LSR, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0x5E, "LSR", Instruction::LSR, AddressingMode::AbsoluteX, 3, 7, false, false),

        Opcode::new(0x2A, "ROL", Instruction::ROL, AddressingMode::Accumulator, 1, 2, false, false),
        Opcode::new(0x26, "ROL", Instruction::ROL, AddressingMode::ZeroPage, 2, 5, false, false),
        Opcode::new(0x36, "ROL", Instruction::ROL, AddressingMode::ZeroPageX, 2, 6, false, false),
        Opcode::new(0x2E, "ROL", Instruction::ROL, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0x3E, "ROL", Instruction::ROL, AddressingMode::AbsoluteX, 3, 7, false, false),

        Opcode::new(0x6A, "ROR", Instruction::ROR, AddressingMode::Accumulator, 1, 2, false, false),
        Opcode::new(0x66, "ROR", Instruction::ROR, AddressingMode::ZeroPage, 2, 5, false, false),
        Opcode::new(0x76, "ROR", Instruction::ROR, AddressingMode::ZeroPageX, 2, 6, false, false),
        Opcode::new(0x6E, "ROR", Instruction::ROR, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0x7E, "ROR", Instruction::ROR, AddressingMode::AbsoluteX, 3, 7, false, false),

        // Jumps & Calls
        Opcode::new(0x4C, "JMP", Instruction::JMP, AddressingMode::Absolute, 3, 3, false, false),
        Opcode::new(0x6C, "JMP", Instruction::JMP, AddressingMode::Indirect, 3, 5, false, false),
        Opcode::new(0x20, "JSR", Instruction::JSR, AddressingMode::Absolute, 3, 6, false, false),
        Opcode::new(0x60, "RTS", Instruction::RTS, AddressingMode::Implied, 1, 6, false, false),

        // Branches
        Opcode::new(0x90, "BCC", Instruction::BCC, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0xB0, "BCS", Instruction::BCS, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0xF0, "BEQ", Instruction::BEQ, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0x30, "BMI", Instruction::BMI, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0xD0, "BNE", Instruction::BNE, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0x10, "BPL", Instruction::BPL, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0x50, "BVC", Instruction::BVC, AddressingMode::Relative, 2, 2, true, true),
        Opcode::new(0x70, "BVS", Instruction::BVS, AddressingMode::Relative, 2, 2, true, true),

        // Status Flag Changes
        Opcode::new(0x18, "CLC", Instruction::CLC, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0xD8, "CLD", Instruction::CLD, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x58, "CLI", Instruction::CLI, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0xB8, "CLV", Instruction::CLV, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x38, "SEC", Instruction::SEC, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0xF8, "SED", Instruction::SED, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x78, "SEI", Instruction::SEI, AddressingMode::Implied, 1, 2, false, false),

        // System Functions
        Opcode::new(0x00, "BRK", Instruction::BRK, AddressingMode::Implied, 2, 7, false, false),
        Opcode::new(0xEA, "NOP", Instruction::NOP, AddressingMode::Implied, 1, 2, false, false),
        Opcode::new(0x40, "RTI", Instruction::RTI, AddressingMode::Implied, 1, 6, false, false),
    ];

    pub static ref OPCODES_MAP: HashMap<u8, &'static Opcode> = {
        let mut map = HashMap::new();
        for opcode in OPCODES.iter() {
            map.insert(opcode.opcode, opcode);
        }

        map
    };
}
