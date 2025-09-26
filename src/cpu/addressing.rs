use super::Cpu;

#[derive(Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

impl Cpu {
    pub fn get_operand_address(&mut self, mode: AddressingMode, pc: u16) -> (u16, bool) {
        match mode {
            AddressingMode::Implied | AddressingMode::Accumulator => (0, false), // No address needed
            AddressingMode::Immediate => (pc, false), // The operand is the value (not an address)
            AddressingMode::ZeroPage => {
                // Zero page address (0x0000-0x00FF)
                let address = self.read(pc) as u16;
                (address, false)
            }
            AddressingMode::ZeroPageX => {
                // Zero page address + X register (wraps within 0x0000-0x00FF)
                let base = self.read(pc);
                let address = base.wrapping_add(self.x) as u16;
                (address, false)
            }
            AddressingMode::ZeroPageY => {
                // Zero page address + Y register (wraps within 0x0000-0x00FF)
                let base = self.read(pc);
                let address = base.wrapping_add(self.x) as u16;
                (address, false)
            }
            AddressingMode::Relative => {
                // 8-bit signed offset (wraps at $FFFF)
                let offset = self.read(pc) as i8;
                let address = self.pc.wrapping_add(offset as u16);
                (address, (self.pc & 0xFF00) != (address & 0xFF00))
            }
            AddressingMode::Absolute => {
                // Full 16-bit address
                let address = self.read_u16(pc);
                (address, false)
            }
            AddressingMode::AbsoluteX => {
                // Full 16-bit address + X register (wraps at $FFFF)
                let base = self.read_u16(pc);
                let address = base.wrapping_add(self.x as u16);
                (address, (base & 0xFF00) != (address & 0xFF00))
            }
            AddressingMode::AbsoluteY => {
                // Full 16-bit address + Y register (wraps at $FFFF)
                let base = self.read_u16(pc);
                let address = base.wrapping_add(self.y as u16);
                (address, (base & 0xFF00) != (address & 0xFF00))
            }
            AddressingMode::Indirect => {
                // Full 16-bit address read from pointer address (only used by JMP)
                let pointer = self.read_u16(pc);

                // 6502 bug: if the pointer is at the page boundary (e.g. $10FF), high byte is read for $1000 instead of $1100s
                let low_byte = self.read(pointer) as u16;
                let high_byte = if pointer & 0x00FF == 0x00FF {
                    self.read(pointer & 0xFF00) as u16
                } else {
                    self.read(pointer + 1) as u16
                };

                let address = (high_byte << 8) | low_byte;
                (address, false)
            }
            AddressingMode::IndirectX => {
                // Full 16-bit address read from pointer address + X register (wraps within 0x0000-0x00FF)
                let base = self.read(pc);
                let pointer = base.wrapping_add(self.x);

                let low_byte = self.read(pointer as u16) as u16;
                let high_byte = self.read(pointer.wrapping_add(1) as u16) as u16;

                let address = (high_byte << 8) | low_byte;
                (address, false)
            }
            AddressingMode::IndirectY => {
                // Full 16-bit address read from pointer address + Y register (wraps within 0x0000-0x00FF)
                let pointer = self.read(pc) as u16;

                let low_byte = self.read(pointer) as u16;
                let high_byte = self.read((pointer + 1) & 0xFF) as u16;

                let base = (high_byte << 8) | low_byte;
                let address = base.wrapping_add(self.y as u16);
                (address, (base & 0xFF00) != (address & 0xFF00))
            }
        }
    }
}
