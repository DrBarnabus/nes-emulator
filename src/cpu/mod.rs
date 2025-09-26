mod addressing;
mod instructions;

use super::bus::Bus;
use addressing::AddressingMode;
use instructions::Opcode;

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct StatusFlags: u8 {
        const CARRY = 0b0000_0001; // C
        const ZERO = 0b0000_0010; // Z
        const INTERRUPT_DISABLE = 0b0000_0100; // I
        const DECIMAL_MODE = 0b0000_1000; // D
        const BREAK_COMMAND = 0b0001_0000; // B
        const UNUSED = 0b0010_0000; // Always 1
        const OVERFLOW = 0b0100_0000; // V
        const NEGATIVE = 0b1000_0000; // N
    }
}

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xFD;

pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: StatusFlags,

    pub bus: Bus,

    cycles: u64,

    current_instruction: Option<(Opcode, u16)>,
    cycles_remaining: u8,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            pc: 0,
            sp: STACK_RESET,
            a: 0,
            x: 0,
            y: 0,
            status: StatusFlags::INTERRUPT_DISABLE | StatusFlags::UNUSED,
            bus,
            cycles: 0,
            current_instruction: None,
            cycles_remaining: 0,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.status = StatusFlags::INTERRUPT_DISABLE | StatusFlags::UNUSED;

        self.pc = self.read_u16(0xFFFC);
    }

    pub fn run<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Cpu),
    {
        loop {
            self.clock(); // First clock to fetch the next instruction
            while self.cycles_remaining > 0 {
                self.clock();
            }

            callback(self);
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_remaining == 0 {
            self.handle_interrupts();

            if self.cycles_remaining == 0 {
                let opcode = self.read(self.pc);
                self.pc += 1; // Move past the opcode byte

                let op = self.decode_opcode(opcode);
                let operand_pc = self.pc;

                // Advance PC past the operands (unless it's a control flow instruction)
                if !self.is_control_flow_instruction(op.instruction) {
                    self.pc += (op.bytes - 1) as u16;
                }

                self.cycles_remaining = op.cycles;
                self.current_instruction = Some((op, operand_pc));
            }
        }

        self.cycles_remaining -= 1;
        if self.cycles_remaining == 0 {
            if let Some((op, operand_pc)) = self.current_instruction.take() {
                self.execute_instruction(op, operand_pc);
            }
        }

        self.cycles += 1;
    }

    pub fn read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        let low_byte = self.read(address) as u16;
        let high_byte = self.read(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    pub fn read_operand(&mut self, mode: AddressingMode, pc: u16) -> (u8, bool) {
        match mode {
            AddressingMode::Accumulator => (self.a, false),
            AddressingMode::Immediate => (self.read(pc), false),
            _ => {
                let (address, page_crossed) = self.get_operand_address(mode, pc);
                (self.read(address), page_crossed)
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(STACK + self.sp as u16)
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let low_byte = self.stack_pop() as u16;
        let high_byte = self.stack_pop() as u16;
        (high_byte << 8) | low_byte
    }

    fn stack_push(&mut self, value: u8) {
        self.write(STACK + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self, value: u16) {
        let high_byte = (value >> 8) as u8;
        let low_byte = (value & 0xFF) as u8;
        self.stack_push(high_byte);
        self.stack_push(low_byte);
    }

    fn set_zero_flag(&mut self, value: u8) {
        if value == 0 {
            self.status.insert(StatusFlags::ZERO);
        } else {
            self.status.remove(StatusFlags::ZERO);
        }
    }

    fn set_negative_flag(&mut self, value: u8) {
        if value & 0x80 != 0 {
            self.status.insert(StatusFlags::NEGATIVE);
        } else {
            self.status.remove(StatusFlags::NEGATIVE);
        }
    }

    fn set_zero_and_negative_flags(&mut self, value: u8) {
        self.set_zero_flag(value);
        self.set_negative_flag(value);
    }

    fn set_register_a(&mut self, value: u8) {
        self.a = value;
        self.set_zero_and_negative_flags(value);
    }
    
    fn set_register_x(&mut self, value: u8) {
        self.x = value;
        self.set_zero_and_negative_flags(value);
    }
    
    fn set_register_y(&mut self, value: u8) {
        self.y = value;
        self.set_zero_and_negative_flags(value);
    }

    fn handle_interrupts(&mut self) {
        if self.bus.poll_nmi() {
            self.handle_interrupt(0xFFFA); // Jump to NMI vector $FFFA-$FFFB
        } else if self.bus.poll_irq() && !self.status.contains(StatusFlags::INTERRUPT_DISABLE) {
            self.handle_interrupt(0xFFFE); // Jump to IRQ vector $FFFE-$FFFF
        }
    }

    fn handle_interrupt(&mut self, interrupt_vector: u16) {
        self.stack_push_u16(self.pc);

        let mut status = self.status;
        status.remove(StatusFlags::BREAK_COMMAND);
        status.insert(StatusFlags::UNUSED);
        self.stack_push(status.bits());

        self.status.insert(StatusFlags::INTERRUPT_DISABLE);

        self.pc = self.read_u16(interrupt_vector);
        self.cycles_remaining = 7; // NMI/IRQ takes 7 cycles
    }
}
