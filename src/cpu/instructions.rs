use super::{Cpu, StatusFlags, addressing::AddressingMode, mem::Mem, opcode::Opcode};

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

    // Undocumented
    AAC, // AND with Carry
    AAX, // Store Accumulator and X Register
    ARR, // AND then Rotate Right
    ASR, // AND then Logical Shift Right
    ATX, // Load X Register and Accumulator
    AXA, // Store Accumulator AND X Register AND High Byte
    AXS, // Subtract from X Register
    DCP, // Decrement and Compare
    DOP, // Double NOP
    ISC, // Increment and Subtract with Carry
    KIL, // Kill/Jam Processor
    LAR, // Load Accumulator, X Register and Stack Pointer
    LAX, // Load Accumulator and X Register
    RLA, // Rotate Left and AND
    RRA, // Rotate Right and Add
    SLO, // Shift Left and OR
    SRE, // Shift Right and EOR
    SXA, // Store X Register and High Byte
    SYA, // Store Y Register and High Byte
    TOP, // Triple NOP
    XAA, // Transfer X Register to Accumulator
    XAS, // Transfer Accumulator AND X Register to Stack Pointer
}

impl Instruction {
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
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

            // Undocumented
            Instruction::AAC => self.aac(operand_pc),
            Instruction::AAX => self.aax(op.mode, operand_pc),
            Instruction::ARR => self.arr(operand_pc),
            Instruction::ASR => self.asr(operand_pc),
            Instruction::ATX => self.atx(operand_pc),
            Instruction::AXA => self.axa(op.mode, operand_pc),
            Instruction::AXS => self.axs(operand_pc),
            Instruction::DCP => self.dcp(op.mode, operand_pc),
            Instruction::DOP => {
                self.read_operand(op.mode, operand_pc);
            }
            Instruction::ISC => self.isc(op.mode, operand_pc),
            Instruction::KIL => self.kil(),
            Instruction::LAR => self.lar(op.mode, operand_pc),
            Instruction::LAX => self.lax(op.mode, operand_pc),
            Instruction::RLA => self.rla(op.mode, operand_pc),
            Instruction::RRA => self.rra(op.mode, operand_pc),
            Instruction::SLO => self.slo(op.mode, operand_pc),
            Instruction::SRE => self.sre(op.mode, operand_pc),
            Instruction::SXA => self.sxa(op.mode, operand_pc),
            Instruction::SYA => self.sya(op.mode, operand_pc),
            Instruction::TOP => self.top(op.mode, operand_pc),
            Instruction::XAA => self.xaa(op.mode, operand_pc),
            Instruction::XAS => self.xas(op.mode, operand_pc),
        }
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
        self.status.insert(StatusFlags::UNUSED);
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

    fn aac(&mut self, operand_pc: u16) {
        let value = self.read(operand_pc);
        self.set_register_a(value & self.a);
        self.status.set(StatusFlags::CARRY, self.status.contains(StatusFlags::NEGATIVE));
    }

    fn aax(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        self.write(address, self.a & self.x);
    }

    fn arr(&mut self, operand_pc: u16) {
        let value = self.read(operand_pc);
        self.set_register_a(value & self.a);
        self.ror_accumulator();

        let result = self.a;
        let bit_5 = (result >> 5) & 1;
        let bit_6 = (result >> 6) & 1;

        self.status.set(StatusFlags::CARRY, bit_6 == 1);
        self.status.set(StatusFlags::OVERFLOW, bit_5 ^ bit_6 == 1);
        self.set_zero_and_negative_flags(result);
    }

    fn asr(&mut self, operand_pc: u16) {
        let value = self.read(operand_pc);
        self.set_register_a(value & self.a);
        self.lsr_accumulator();
    }

    fn atx(&mut self, operand_pc: u16) {
        self.lda(AddressingMode::Immediate, operand_pc);
        self.tax();
    }

    fn axa(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);

        let result = self.a & self.x & (address >> 8) as u8;
        self.write(address, result);
    }

    fn axs(&mut self, operand_pc: u16) {
        let value = self.read(operand_pc);

        let a_and_x = self.a & self.x;
        let result = a_and_x.wrapping_sub(value);

        self.status.set(StatusFlags::CARRY, value <= a_and_x);
        self.set_register_x(result);
    }

    fn dcp(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let value = self.read(address);

        let result = value.wrapping_sub(1);
        self.write(address, result);

        self.status.set(StatusFlags::CARRY, result <= self.a);
        self.set_zero_and_negative_flags(self.a.wrapping_sub(result));
    }

    fn isc(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let value = self.read(address);
        let result = value.wrapping_add(1);

        self.write(address, result);
        self.set_zero_and_negative_flags(result);
        self.add_value_to_register_a((result as i8).wrapping_neg().wrapping_sub(1) as u8);
    }

    fn kil(&mut self) {
        self.halted = true;
    }

    fn lar(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);

        let result = value & self.sp;
        self.a = result;
        self.x = result;
        self.sp = result;
        self.set_zero_and_negative_flags(result);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn lax(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (value, page_crossed) = self.read_operand(mode, operand_pc);

        self.set_register_a(value);
        self.x = self.a;

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn rla(&mut self, mode: AddressingMode, operand_pc: u16) {
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

        self.set_register_a(result & self.a);
    }

    fn rra(&mut self, mode: AddressingMode, operand_pc: u16) {
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

        self.add_value_to_register_a(result);
    }

    fn slo(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let mut result = self.read(address);

        self.status.set(StatusFlags::CARRY, result >> 7 == 1);

        result <<= 1;
        self.write(address, result);
        self.set_zero_and_negative_flags(result);

        self.set_register_a(result | self.a);
    }

    fn sre(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);
        let mut result = self.read(address);

        self.status.set(StatusFlags::CARRY, result & 1 == 1);

        result >>= 1;
        self.write(address, result);
        self.set_zero_and_negative_flags(result);

        self.set_register_a(result ^ self.a);
    }

    fn sxa(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);

        let result = self.x & ((address >> 8) as u8 + 1);
        self.write(address, result);
    }

    fn sya(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);

        let result = self.y & ((address >> 8) as u8 + 1);
        self.write(address, result);
    }

    fn top(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (_, page_crossed) = self.read_operand(mode, operand_pc);

        if page_crossed {
            self.cycles_remaining += 1;
        }
    }

    fn xaa(&mut self, mode: AddressingMode, operand_pc: u16) {
        self.a = self.x;
        self.set_zero_and_negative_flags(self.a);

        let (value, _) = self.read_operand(mode, operand_pc);
        self.set_register_a(value & self.a);
    }

    fn xas(&mut self, mode: AddressingMode, operand_pc: u16) {
        let (address, _) = self.get_operand_address(mode, operand_pc);

        let value = self.a & self.x;
        self.sp = value;

        let value = ((address >> 8) as u8 + 1) & self.sp;
        self.write(address, value);
    }

    fn add_value_to_register_a(&mut self, value: u8) {
        let result = self.a as u16 + value as u16 + (if self.status.contains(StatusFlags::CARRY) { 1 } else { 0 });

        self.status.set(StatusFlags::CARRY, result > 0xFF);

        let result = result as u8;
        self.status.set(StatusFlags::OVERFLOW, (value ^ result) & (result ^ self.a) & 0x80 != 0);

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

// #[cfg(test)]
// mod tests {
//     use crate::bus::Bus;
//     use crate::cpu::mem::Mem;
//     use crate::cpu::{Cpu, StatusFlags};
//
//     #[test]
//     fn test_lda_immediate() {
//         let mut bus = Bus::new();
//         bus.load_program(&[0xA9, 0x42], 0x0600); // LDA #$42
//
//         let mut cpu = Cpu::new(bus);
//         cpu.pc = 0x0600;
//
//         cpu.clock(); // Fetch opcode
//         cpu.clock(); // Complete instruction
//
//         assert_eq!(cpu.a, 0x42);
//         assert_eq!(cpu.pc, 0x0602);
//         assert!(!cpu.status.contains(StatusFlags::ZERO));
//         assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
//     }
//
//     #[test]
//     fn test_lda_immediate_zero_flag() {
//         let mut bus = Bus::new();
//         bus.load_program(&[0xA9, 0x00], 0x0600); // LDA #0
//
//         let mut cpu = Cpu::new(bus);
//         cpu.pc = 0x0600;
//
//         cpu.clock();
//         cpu.clock();
//
//         assert_eq!(cpu.a, 0x00);
//         assert!(cpu.status.contains(StatusFlags::ZERO));
//         assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
//     }
//
//     #[test]
//     fn test_lda_immediate_negative_flag() {
//         let mut bus = Bus::new();
//         bus.load_program(&[0xA9, 0x80], 0x0600); // LDA #80
//
//         let mut cpu = Cpu::new(bus);
//         cpu.pc = 0x0600;
//
//         cpu.clock();
//         cpu.clock();
//
//         assert_eq!(cpu.a, 0x80);
//         assert!(!cpu.status.contains(StatusFlags::ZERO));
//         assert!(cpu.status.contains(StatusFlags::NEGATIVE));
//     }
//
//     #[test]
//     fn test_sta_zero_page() {
//         let mut bus = Bus::new();
//         bus.load_program(&[0xA9, 0x42, 0x85, 0x85], 0x0600); // LDA #$42, STA $85
//
//         let mut cpu = Cpu::new(bus);
//         cpu.pc = 0x0600;
//
//         cpu.clock(); // Fetch opcode
//         cpu.clock(); // Complete instruction
//         cpu.clock(); // Fetch opcode
//         cpu.clock(); // Fetch operand
//         cpu.clock(); // Complete instruction
//
//         assert_eq!(cpu.bus.read(0x0085), 0x42);
//     }
// }
