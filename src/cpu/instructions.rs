use super::{Cpu, StatusFlags, addressing::AddressingMode};

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Instruction {
    // Load/Store Operations
    LDA, // Load Accumulator
    LDX, // Load X Register
    LDY, // Load Y Register
    STA, // Store Accumulator
    STX, // Store X Register
    STY, // Store Y Register

    // Register Transfers
    TAX, // Transfer Accumulator to X
    TAY, // Transfer Accumulator to Y
    TXA, // Transfer X to Accumulator
    TYA, // Transfer Y to Accumulator

    // Stack Operations
    TSX, // Transfer Stack Pointer to X
    TXS, // Transfer X to Stack Pointer
    PHA, // Push Accumulator on Stack
    PHP, // Push Processor Status on Stack
    PLA, // Pull Accumulator from Stack
    PLP, // Pull Processor Status from Stack

    // Logical
    AND, // Logical AND
    EOR, // Exclusive OR
    ORA, // Logical Inclusive OR
    BIT, // Bit Test

    // Arithmetic
    ADC, // Add with Carry
    SBC, // Subtract with Carry
    CMP, // Compare Accumulator
    CPX, // Compare X Register
    CPY, // Compare Y Register

    // Increments & Decrements
    INC, // Increment a Memory Location
    INX, // Increment the X Register
    INY, // Increment the Y Register
    DEC, // Decrement a Memory Location
    DEX, // Decrement the X Register
    DEY, // Decrement the Y Register

    // Shifts
    ASL, // Arithmetic Shift Left
    LSR, // Logical Shift Right
    ROL, // Rotate Left
    ROR, // Rotate Right

    // Jumps & Calls
    JMP, // Jump to another location
    JSR, // Jump to a subroutine
    RTS, // Return from subroutine

    // Branches
    BCC, // Branch if Carry clear
    BCS, // Branch if Carry set
    BEQ, // Branch if Equal
    BMI, // Branch if Minus
    BNE, // Branch if Not Equal
    BPL, // Branch if Positive
    BVC, // Branch if Overflow clear
    BVS, // Branch if Overflow set

    // Status Flag Changes
    CLC, // Clear Carry flag
    CLD, // Clear Decimal Mode flag
    CLI, // Clear Interrupt Disable flag
    CLV, // Clear Overflow flag
    SEC, // Set Carry flag
    SED, // Set Decimal Mode flag
    SEI, // Set Interrupt Disable flag

    // System Functions
    BRK, // Force Interrupt
    NOP, // No Operation
    RTI, // Return from Interrupt

    XXX, // Unknown instruction
}

pub struct Opcode {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
}

impl Cpu {
    pub fn execute_instruction(&mut self, op: Opcode, operand_pc: u16) {
        match op.instruction {
            // Load/Store Operations
            Instruction::LDA => self.lda(op.mode, operand_pc),
            Instruction::LDX => self.ldx(op.mode, operand_pc),
            Instruction::LDY => self.ldy(op.mode, operand_pc),
            Instruction::STA => self.sta(op.mode, operand_pc),
            Instruction::STX => self.stx(op.mode, operand_pc),
            Instruction::STY => self.sty(op.mode, operand_pc),

            // Register Transfers
            Instruction::TAX => self.tax(),
            Instruction::TAY => self.tay(),
            Instruction::TXA => self.txa(),
            Instruction::TYA => self.tya(),

            // Stack Operations
            Instruction::TSX => self.tsx(),
            Instruction::TXS => self.txs(),
            Instruction::PHA => self.pha(),
            Instruction::PHP => self.php(),
            Instruction::PLA => self.pla(),
            Instruction::PLP => self.plp(),

            // Logical
            Instruction::AND => self.and(op.mode, operand_pc),
            Instruction::EOR => self.eor(op.mode, operand_pc),
            Instruction::ORA => self.ora(op.mode, operand_pc),
            Instruction::BIT => self.bit(op.mode, operand_pc),

            // Arithmetic
            Instruction::ADC => self.adc(op.mode, operand_pc),
            Instruction::SBC => self.sbc(op.mode, operand_pc),
            Instruction::CMP => self.cmp(op.mode, operand_pc),
            Instruction::CPX => self.cpx(op.mode, operand_pc),
            Instruction::CPY => self.cpy(op.mode, operand_pc),

            // Increments & Decrements
            Instruction::INC => self.inc(op.mode, operand_pc),
            Instruction::INX => self.inx(),
            Instruction::INY => self.iny(),
            Instruction::DEC => self.dec(op.mode, operand_pc),
            Instruction::DEX => self.dex(),
            Instruction::DEY => self.dey(),

            // Shifts
            Instruction::ASL if op.mode == AddressingMode::Accumulator => self.asl_accumulator(),
            Instruction::ASL => self.asl(op.mode, operand_pc),
            Instruction::LSR if op.mode == AddressingMode::Accumulator => self.lsr_accumulator(),
            Instruction::LSR => self.lsr(op.mode, operand_pc),
            Instruction::ROL if op.mode == AddressingMode::Accumulator => self.rol_accumulator(),
            Instruction::ROL => self.rol(op.mode, operand_pc),
            Instruction::ROR if op.mode == AddressingMode::Accumulator => self.ror_accumulator(),
            Instruction::ROR => self.ror(op.mode, operand_pc),

            // Jumps & Calls
            Instruction::JMP => self.jmp(op.mode, operand_pc),
            Instruction::JSR => self.jsr(op.mode, operand_pc),
            Instruction::RTS => self.rts(),

            // Branches
            Instruction::BCC => self.bcc(operand_pc),
            Instruction::BCS => self.bcs(operand_pc),
            Instruction::BEQ => self.beq(operand_pc),
            Instruction::BMI => self.bmi(operand_pc),
            Instruction::BNE => self.bne(operand_pc),
            Instruction::BPL => self.bpl(operand_pc),
            Instruction::BVC => self.bvc(operand_pc),
            Instruction::BVS => self.bvs(operand_pc),

            // Status Flag Changes
            Instruction::CLC => self.clc(),
            Instruction::CLD => self.cld(),
            Instruction::CLI => self.cli(),
            Instruction::CLV => self.clv(),
            Instruction::SEC => self.sec(),
            Instruction::SED => self.sed(),
            Instruction::SEI => self.sei(),

            // System Functions
            Instruction::BRK => self.brk(),
            Instruction::NOP => {}
            Instruction::RTI => self.rti(),

            Instruction::XXX => panic!("Unknown instruction: {:#04X}", op.instruction as u16),
        }
    }

    pub fn decode_opcode(&self, opcode: u8) -> Opcode {
        match opcode {
            // ===== LDA (Load Accumulator) =====
            0xA9 => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xA5 => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0xB5 => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0xAD => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0xBD => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0xB9 => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0xA1 => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0xB1 => Opcode {
                instruction: Instruction::LDA,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== LDX (Load X Register) =====
            0xA2 => Opcode {
                instruction: Instruction::LDX,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xA6 => Opcode {
                instruction: Instruction::LDX,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 2,
            },
            0xB6 => Opcode {
                instruction: Instruction::LDX,
                mode: AddressingMode::ZeroPageY,
                bytes: 2,
                cycles: 3,
            },
            0xAE => Opcode {
                instruction: Instruction::LDX,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0xBE => Opcode {
                instruction: Instruction::LDX,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed

            // ===== LDY (Load Y Register) =====
            0xA0 => Opcode {
                instruction: Instruction::LDY,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xA4 => Opcode {
                instruction: Instruction::LDY,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0xB4 => Opcode {
                instruction: Instruction::LDY,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0xAC => Opcode {
                instruction: Instruction::LDY,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0xBC => Opcode {
                instruction: Instruction::LDY,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed

            // ===== STA (Store Accumulator) =====
            0x85 => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x95 => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0x8D => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0x9D => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 5,
            },
            0x99 => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 5,
            },
            0x81 => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0x91 => Opcode {
                instruction: Instruction::STA,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 6,
            },

            // ===== STX (Store X Register) =====
            0x86 => Opcode {
                instruction: Instruction::STX,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x96 => Opcode {
                instruction: Instruction::STX,
                mode: AddressingMode::ZeroPageY,
                bytes: 2,
                cycles: 4,
            },
            0x8E => Opcode {
                instruction: Instruction::STX,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },

            // ===== STY (Store Y Register) =====
            0x84 => Opcode {
                instruction: Instruction::STY,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x94 => Opcode {
                instruction: Instruction::STY,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0x8C => Opcode {
                instruction: Instruction::STY,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },

            // ===== TAX (Transfer Accumulator to X) =====
            0xAA => Opcode {
                instruction: Instruction::TAX,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== TAY (Transfer Accumulator to Y) =====
            0xA8 => Opcode {
                instruction: Instruction::TAY,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== TXA (Transfer X to Accumulator) =====
            0x8A => Opcode {
                instruction: Instruction::TXA,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== TYA (Transfer Y to Accumulator) =====
            0x98 => Opcode {
                instruction: Instruction::TYA,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== TSX (Transfer Stack Pointer to X) =====
            0xBA => Opcode {
                instruction: Instruction::TSX,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== TXS (Transfer X to Stack Pointer) =====
            0x9A => Opcode {
                instruction: Instruction::TXS,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== PHA (Push Accumulator on Stack) =====
            0x48 => Opcode {
                instruction: Instruction::PHA,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 3,
            },

            // ===== PHP (Push Processor Status on Stack) =====
            0x08 => Opcode {
                instruction: Instruction::PHP,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 3,
            },

            // ===== PLA (Pull Accumulator from Stack) =====
            0x68 => Opcode {
                instruction: Instruction::PLA,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 4,
            },

            // ===== PLP (Pull Processor Status from Stack) =====
            0x28 => Opcode {
                instruction: Instruction::PLP,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 4,
            },

            // ===== AND (Logical AND) =====
            0x29 => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0x25 => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x35 => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0x2D => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0x3D => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x39 => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x21 => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0x31 => Opcode {
                instruction: Instruction::AND,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== EOR (Exclusive OR) =====
            0x49 => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0x45 => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x55 => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0x4D => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0x5D => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x59 => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x41 => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0x51 => Opcode {
                instruction: Instruction::EOR,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== ORA (Logical OR) =====
            0x09 => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0x05 => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x15 => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0x0D => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0x1D => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x19 => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x01 => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0x11 => Opcode {
                instruction: Instruction::ORA,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== BIT (Bit Test) =====
            0x24 => Opcode {
                instruction: Instruction::BIT,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x2C => Opcode {
                instruction: Instruction::BIT,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },

            // ===== ADC (Add with Carry) =====
            0x69 => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0x65 => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0x75 => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0x6D => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0x7D => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x79 => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0x61 => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0x71 => Opcode {
                instruction: Instruction::ADC,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== SBC (Subtract with Carry) =====
            0xE9 => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xE5 => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0xF5 => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0xED => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0xFD => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0xF9 => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0xE1 => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0xF1 => Opcode {
                instruction: Instruction::SBC,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== CMP (Compare Accumulator) =====
            0xC9 => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xC5 => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0xD5 => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            0xCD => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },
            0xDD => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0xD9 => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::AbsoluteY,
                bytes: 3,
                cycles: 4,
            }, // +1 if page boundary crossed
            0xC1 => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::IndirectX,
                bytes: 2,
                cycles: 6,
            },
            0xD1 => Opcode {
                instruction: Instruction::CMP,
                mode: AddressingMode::IndirectY,
                bytes: 2,
                cycles: 5,
            }, // +1 if page boundary crossed

            // ===== CPX (Compare X Register) =====
            0xE0 => Opcode {
                instruction: Instruction::CPX,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xE4 => Opcode {
                instruction: Instruction::CPX,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0xEC => Opcode {
                instruction: Instruction::CPX,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },

            // ===== CPY (Compare Y Register) =====
            0xC0 => Opcode {
                instruction: Instruction::CPY,
                mode: AddressingMode::Immediate,
                bytes: 2,
                cycles: 2,
            },
            0xC4 => Opcode {
                instruction: Instruction::CPY,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            0xCC => Opcode {
                instruction: Instruction::CPY,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 4,
            },

            // ===== INC (Increment a Memory Location) =====
            0xE6 => Opcode {
                instruction: Instruction::INC,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            0xF6 => Opcode {
                instruction: Instruction::INC,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            0xEE => Opcode {
                instruction: Instruction::INC,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },
            0xFE => Opcode {
                instruction: Instruction::INC,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // ===== INX (Increment the X Register) =====
            0xE8 => Opcode {
                instruction: Instruction::INX,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== INY (Increment the Y Register) =====
            0xC8 => Opcode {
                instruction: Instruction::INY,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== DEC (Decrement a Memory Location) =====
            0xC6 => Opcode {
                instruction: Instruction::DEC,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            0xD6 => Opcode {
                instruction: Instruction::DEC,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            0xCE => Opcode {
                instruction: Instruction::DEC,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },
            0xDE => Opcode {
                instruction: Instruction::DEC,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // ===== DEX (Decrement the X Register) =====
            0xCA => Opcode {
                instruction: Instruction::DEX,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== DEY (Decrement the Y Register) =====
            0x88 => Opcode {
                instruction: Instruction::DEY,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== ASL (Arithmetic Shift Left) =====
            0x0A => Opcode {
                instruction: Instruction::ASL,
                mode: AddressingMode::Accumulator,
                bytes: 1,
                cycles: 2,
            },
            0x06 => Opcode {
                instruction: Instruction::ASL,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            0x16 => Opcode {
                instruction: Instruction::ASL,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            0x0E => Opcode {
                instruction: Instruction::ASL,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },
            0x1E => Opcode {
                instruction: Instruction::ASL,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // ===== LSR (Logical Shift Right) =====
            0x4A => Opcode {
                instruction: Instruction::LSR,
                mode: AddressingMode::Accumulator,
                bytes: 1,
                cycles: 2,
            },
            0x46 => Opcode {
                instruction: Instruction::LSR,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            0x56 => Opcode {
                instruction: Instruction::LSR,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            0x4E => Opcode {
                instruction: Instruction::LSR,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },
            0x5E => Opcode {
                instruction: Instruction::LSR,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // ===== ROL (Rotate Left) =====
            0x2A => Opcode {
                instruction: Instruction::ROL,
                mode: AddressingMode::Accumulator,
                bytes: 1,
                cycles: 2,
            },
            0x26 => Opcode {
                instruction: Instruction::ROL,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            0x36 => Opcode {
                instruction: Instruction::ROL,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            0x2E => Opcode {
                instruction: Instruction::ROL,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },
            0x3E => Opcode {
                instruction: Instruction::ROL,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // ===== ROR (Rotate Right) =====
            0x6A => Opcode {
                instruction: Instruction::ROR,
                mode: AddressingMode::Accumulator,
                bytes: 1,
                cycles: 2,
            },
            0x66 => Opcode {
                instruction: Instruction::ROR,
                mode: AddressingMode::ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            0x76 => Opcode {
                instruction: Instruction::ROR,
                mode: AddressingMode::ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            0x6E => Opcode {
                instruction: Instruction::ROR,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },
            0x7E => Opcode {
                instruction: Instruction::ROR,
                mode: AddressingMode::AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // ===== JMP (Jump to another location) =====
            0x4C => Opcode {
                instruction: Instruction::JMP,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 3,
            },
            0x6C => Opcode {
                instruction: Instruction::JMP,
                mode: AddressingMode::Indirect,
                bytes: 3,
                cycles: 5,
            },

            // ===== JSR (Jump to another location) =====
            0x20 => Opcode {
                instruction: Instruction::JSR,
                mode: AddressingMode::Absolute,
                bytes: 3,
                cycles: 6,
            },

            // ===== RTS (Return from subroutine) =====
            0x60 => Opcode {
                instruction: Instruction::RTS,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 6,
            },

            // ===== BCC (Branch if Carry clear) =====
            0x90 => Opcode {
                instruction: Instruction::BCC,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BCS (Branch if Carry set) =====
            0xB0 => Opcode {
                instruction: Instruction::BCS,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BEQ (Branch if Equal) =====
            0xF0 => Opcode {
                instruction: Instruction::BEQ,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BMI (Branch if Minus) =====
            0x30 => Opcode {
                instruction: Instruction::BMI,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BNE (Branch if Not Equal) =====
            0xD0 => Opcode {
                instruction: Instruction::BNE,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BPL (Branch if Positive) =====
            0x10 => Opcode {
                instruction: Instruction::BPL,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BVC (Branch if Overflow clear) =====
            0x50 => Opcode {
                instruction: Instruction::BVC,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== BVS (Branch if Overflow set) =====
            0x70 => Opcode {
                instruction: Instruction::BVS,
                mode: AddressingMode::Relative,
                bytes: 2,
                cycles: 2,
            }, // +1 if branch succeeds, +2 if succeeds and page boundary crossed

            // ===== CLC (Clear Carry Flag) =====
            0x18 => Opcode {
                instruction: Instruction::CLC,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== CLD (Clear Decimal Mode) =====
            0xD8 => Opcode {
                instruction: Instruction::CLD,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== CLI (Clear Interrupt Disable) =====
            0x58 => Opcode {
                instruction: Instruction::CLI,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== CLV (Clear Overflow Flag) =====
            0xB8 => Opcode {
                instruction: Instruction::CLV,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== SEC (Set Carry Flag) =====
            0x38 => Opcode {
                instruction: Instruction::SEC,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== SED (Set Decimal Mode) =====
            0xF8 => Opcode {
                instruction: Instruction::SED,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== SEI (Set Interrupt Disable) =====
            0x78 => Opcode {
                instruction: Instruction::SEI,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== BRK (Force Interrupt) =====
            0x00 => Opcode {
                instruction: Instruction::BRK,
                mode: AddressingMode::Implied,
                bytes: 2,
                cycles: 7,
            },

            // ===== NOP (No Operation) =====
            0xEA => Opcode {
                instruction: Instruction::NOP,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 2,
            },

            // ===== RTI (Return from Interrupt) =====
            0x40 => Opcode {
                instruction: Instruction::RTI,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 6,
            },

            // ===== XXX (Unknown Opcode) =====
            _ => Opcode {
                instruction: Instruction::XXX,
                mode: AddressingMode::Implied,
                bytes: 1,
                cycles: 1,
            },
        }
    }

    pub fn is_control_flow_instruction(&self, instruction: Instruction) -> bool {
        matches!(
            instruction,
            Instruction::JMP
                | Instruction::JSR
                | Instruction::RTS
                | Instruction::BCC
                | Instruction::BCS
                | Instruction::BEQ
                | Instruction::BMI
                | Instruction::BNE
                | Instruction::BPL
                | Instruction::BVC
                | Instruction::BVS
                | Instruction::BRK
                | Instruction::RTI
        )
    }

    fn lda(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.set_register_a(value);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn ldx(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.set_register_x(value);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn ldy(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.set_register_y(value);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn sta(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        self.write(address, self.a);
    }

    fn stx(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        self.write(address, self.x);
    }

    fn sty(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        self.write(address, self.y);
    }

    fn tax(&mut self) {
        self.set_register_x(self.a);
    }

    fn tay(&mut self) {
        self.set_register_y(self.a);
    }

    fn txa(&mut self) {
        self.set_register_a(self.x);
    }

    fn tya(&mut self) {
        self.set_register_a(self.y);
    }

    fn tsx(&mut self) {
        self.x = self.sp;
        self.set_zero_and_negative_flags(self.x);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn pha(&mut self) {
        self.stack_push(self.a);
    }

    fn php(&mut self) {
        let mut status = self.status;
        status.insert(StatusFlags::BREAK_COMMAND);

        self.stack_push(status.bits());
    }

    fn pla(&mut self) {
        let value = self.stack_pop();
        self.set_register_a(value);
    }

    fn plp(&mut self) {
        self.status = StatusFlags::from_bits_truncate(self.stack_pop());
        self.status.remove(StatusFlags::BREAK_COMMAND);
    }

    fn and(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.set_register_a(value & self.a);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn eor(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.set_register_a(self.a ^ value);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn ora(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.set_register_a(self.a | value);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn bit(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, _) = self.read_operand(mode, operand_pc);
        let result = self.a & value;

        self.status.set(StatusFlags::ZERO, result == 0);
        self.status.set(StatusFlags::NEGATIVE, value & 0x80 != 0);
        self.status.set(StatusFlags::OVERFLOW, value & 0x40 != 0);
    }

    fn adc(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.add_value_to_register_a(value);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn sbc(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);
        self.add_value_to_register_a((value as i8).wrapping_neg().wrapping_sub(1) as u8);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn cmp(&mut self, mode: AddressingMode, operand_pc: u16) {
        self.compare(mode, operand_pc, self.a);
    }

    fn cpx(&mut self, mode: AddressingMode, operand_pc: u16) {
        self.compare(mode, operand_pc, self.x);
    }

    fn cpy(&mut self, mode: AddressingMode, operand_pc: u16) {
        self.compare(mode, operand_pc, self.y);
    }

    fn inc(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let value = self.read(address);
        let result = value.wrapping_add(1);

        self.write(address, result);
        self.set_zero_and_negative_flags(result);
    }

    fn inx(&mut self) {
        self.set_register_x(self.x.wrapping_add(1));
    }

    fn iny(&mut self) {
        self.set_register_y(self.y.wrapping_add(1));
    }

    fn dec(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let value = self.read(address);
        let result = value.wrapping_sub(1);

        self.write(address, result);
        self.set_zero_and_negative_flags(result);
    }

    fn dex(&mut self) {
        self.set_register_x(self.x.wrapping_sub(1));
    }

    fn dey(&mut self) {
        self.set_register_y(self.y.wrapping_sub(1));
    }

    fn asl_accumulator(&mut self) {
        let mut value = self.a;
        self.status.set(StatusFlags::CARRY, value >> 7 == 1);

        value <<= 1;
        self.set_register_a(value);
    }

    fn asl(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let mut result = self.read(address);

        self.status.set(StatusFlags::CARRY, result >> 7 == 1);

        result <<= 1;
        self.write(address, result);
        self.set_zero_and_negative_flags(result);
    }

    fn lsr_accumulator(&mut self) {
        let mut value = self.a;
        self.status.set(StatusFlags::CARRY, value & 1 == 1);

        value >>= 1;
        self.set_register_a(value);
    }

    fn lsr(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let mut result = self.read(address);

        self.status.set(StatusFlags::CARRY, result & 1 == 1);

        result >>= 1;
        self.write(address, result);
        self.set_zero_and_negative_flags(result);
    }

    fn rol_accumulator(&mut self) {
        let mut value = self.a;
        let carry = self.status.contains(StatusFlags::CARRY);

        self.status.set(StatusFlags::CARRY, value >> 7 == 1);

        value <<= 1;
        if carry {
            value |= 1
        }
        self.set_register_a(value);
    }

    fn rol(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let mut result = self.read(address);
        let carry = self.status.contains(StatusFlags::CARRY);

        self.status.set(StatusFlags::CARRY, result >> 7 == 1);

        result <<= 1;
        if carry {
            result |= 1;
        }
        self.write(address, result);
        self.set_zero_and_negative_flags(result);
    }

    fn ror_accumulator(&mut self) {
        let mut value = self.a;
        let carry = self.status.contains(StatusFlags::CARRY);

        self.status.set(StatusFlags::CARRY, value & 1 == 1);

        value >>= 1;
        if carry {
            value |= 0x80
        }
        self.set_register_a(value);
    }

    fn ror(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let mut result = self.read(address);
        let carry = self.status.contains(StatusFlags::CARRY);

        self.status.set(StatusFlags::CARRY, result & 1 == 1);

        result >>= 1;
        if carry {
            result |= 0x80;
        }
        self.write(address, result);
        self.set_zero_and_negative_flags(result);
    }

    fn jmp(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (target_address, _) = self.get_operand_address(mode, operand_pc);
        self.pc = target_address;
    }

    fn jsr(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (target_address, _) = self.get_operand_address(mode, operand_pc);

        let return_address = self.pc + 1; // Return address is the address of the LAST byte of the JSR instruction
        self.stack_push_u16(return_address);

        self.pc = target_address;
    }

    fn rts(&mut self) {
        let return_address = self.stack_pop_u16();
        self.pc = return_address + 1; // Return address is the address of the LAST byte of the JSR instruction, so we need to add 1 to get the next instruction address
    }

    fn bcc(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, !self.status.contains(StatusFlags::CARRY));
    }

    fn bcs(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, self.status.contains(StatusFlags::CARRY));
    }

    fn beq(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, self.status.contains(StatusFlags::ZERO));
    }

    fn bmi(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, self.status.contains(StatusFlags::NEGATIVE));
    }

    fn bne(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, !self.status.contains(StatusFlags::ZERO));
    }

    fn bpl(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, !self.status.contains(StatusFlags::NEGATIVE));
    }

    fn bvc(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, !self.status.contains(StatusFlags::OVERFLOW));
    }

    fn bvs(&mut self, operand_pc: u16) {
        self.branch_if(operand_pc, self.status.contains(StatusFlags::OVERFLOW));
    }

    fn clc(&mut self) {
        self.status.remove(StatusFlags::CARRY);
    }

    fn cld(&mut self) {
        self.status.remove(StatusFlags::DECIMAL_MODE);
    }

    fn cli(&mut self) {
        self.status.remove(StatusFlags::INTERRUPT_DISABLE);
    }

    fn clv(&mut self) {
        self.status.remove(StatusFlags::OVERFLOW);
    }

    fn sec(&mut self) {
        self.status.insert(StatusFlags::CARRY);
    }

    fn sed(&mut self) {
        self.status.insert(StatusFlags::DECIMAL_MODE);
    }

    fn sei(&mut self) {
        self.status.insert(StatusFlags::INTERRUPT_DISABLE);
    }

    fn brk(&mut self) {
        self.pc += 1; // BRK is a 2-byte instruction (opcode + padding byte), so we need to skip past the padding byte
        self.stack_push_u16(self.pc);

        let mut status = self.status;
        status.insert(StatusFlags::BREAK_COMMAND);
        status.insert(StatusFlags::UNUSED);
        self.stack_push(status.bits());

        self.status.insert(StatusFlags::INTERRUPT_DISABLE);

        // Jump to interrupt vector at $FFFE-$FFFF
        self.pc = self.read_u16(0xFFFE);
    }

    fn rti(&mut self) {
        let status = self.stack_pop();
        self.status = StatusFlags::from_bits_truncate(status & 0xEF) | StatusFlags::UNUSED;

        self.pc = self.stack_pop_u16();
    }

    fn add_value_to_register_a(&mut self, value: u8) {
        let result = self.a as u16
            + value as u16
            + (if self.status.contains(StatusFlags::CARRY) {
                1
            } else {
                0
            });

        self.status.set(StatusFlags::CARRY, result > 0xFF);

        let result = result as u8;
        self.status.set(
            StatusFlags::OVERFLOW,
            (value ^ result) & (result ^ self.a) & 0x80 != 0,
        );

        self.set_register_a(result);
    }

    fn compare(&mut self, mode: AddressingMode, operand_pc: u16, compare_with: u8) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);

        self.status.set(StatusFlags::CARRY, value <= compare_with);
        self.set_zero_and_negative_flags(compare_with.wrapping_sub(value));

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn branch_if(&mut self, operand_pc: u16, condition: bool) {
        if condition {
            let offset = self.read(operand_pc) as i8;
            let branch_base_address = operand_pc + 1;
            let target_address = branch_base_address.wrapping_add(offset as u16);

            let page_crossed = branch_base_address & 0xFF00 != target_address & 0xFF00;
            self.pc = target_address;

            self.cycles_remaining += 1; // +1 for branch taken
            if page_crossed {
                self.cycles_remaining += 1;
            }
        } else {
            self.pc = operand_pc + 1; // Move to the next instruction
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::cpu::{Cpu, StatusFlags};

    #[test]
    fn test_lda_immediate() {
        let mut bus = Bus::new();
        bus.load_program(&[0xA9, 0x42], 0x8000); // LDA #$42

        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x8000;

        cpu.clock(); // Fetch opcode
        cpu.clock(); // Complete instruction

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 0x8002);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_lda_immediate_zero_flag() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xA9); // LDA immediate
        bus.write(0x8001, 0x00); // Load 0

        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x8000;

        cpu.clock();
        cpu.clock();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_lda_immediate_negative_flag() {
        let mut bus = Bus::new();
        bus.write(0x8000, 0xA9); // LDA immediate
        bus.write(0x8001, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x8000;

        cpu.clock();
        cpu.clock();

        assert_eq!(cpu.a, 0x80);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_sta_zero_page() {
        let mut bus = Bus::new();
        bus.load_program(&[0xA9, 0x42, 0x85, 0x85], 0x8000); // LDA #$42, STA $85

        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x8000;

        cpu.clock(); // Fetch opcode
        cpu.clock(); // Complete instruction
        cpu.clock(); // Fetch opcode
        cpu.clock(); // Fetch operand
        cpu.clock(); // Complete instruction

        assert_eq!(cpu.bus.read(0x0085), 0x42);
    }
}
