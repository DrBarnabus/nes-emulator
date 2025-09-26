pub struct Bus {
    // ram: [u8; 0x800], // 2KB internal RAM

    // Temporary ROM so we can test the CPU, this should be a real cartridge
    // rom: [u8; 0x8000], // 32KB of ROM space (0x8000-0xFFFF)

    // Temporary RAM so we can test the CPU alone
    ram: [u8; 0xFFFF], // 64KB internal RAM

    // Interrupt lines
    pub nmi_pending: bool,
    pub irq_pending: bool,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            // ram: [0; 0x800],
            // rom: [0; 0x8000],
            ram: [0; 0xFFFF],

            nmi_pending: false,
            irq_pending: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[address as usize]

        // match address {
        //     // RAM (mirrored every 0x800 bytes from 0x0000 to 0x1FFF)
        //     0x0000..=0x1FFF => self.ram[(address & 0x7FF) as usize],
        //
        //     // PPU registers
        //     0x2000..=0x3FFF => 0,
        //
        //     // APU & I/O registers
        //     0x4000..=0x4017 => 0,
        //
        //     // Cartridge space
        //     // 0x4020..=0xFFFF => 0,
        //     0x8000..=0xFFFF => self.rom[(address - 0x8000) as usize],
        //
        //     _ => 0,
        // }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;

        // match address {
        //     // RAM (mirrored every 0x800 bytes from 0x0000 to 0x1FFF)
        //     0x0000..=0x1FFF => self.ram[(address & 0x7FF) as usize] = value,
        //
        //     // PPU registers (will implement later)
        //     0x2000..=0x3FFF => {},
        //
        //     // APU and I/O registers (will implement later)
        //     0x4000..=0x4017 => {},
        //
        //     // Cartridge space
        //     0x8000..=0xFFFF => self.rom[(address - 0x8000) as usize] = value,
        //
        //     // Other regions
        //     _ => {},
        // }
    }

    pub fn load_program(&mut self, program: &[u8], start_address: u16) {
        for (i, &byte) in program.iter().enumerate() {
            self.write(start_address + i as u16, byte);
        }
    }

    pub fn trigger_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn trigger_irq(&mut self) {
        self.irq_pending = true;
    }

    pub fn poll_nmi(&mut self) -> bool {
        let pending = self.nmi_pending;
        self.nmi_pending = false;
        pending
    }

    pub fn poll_irq(&mut self) -> bool {
        let pending = self.irq_pending;
        self.irq_pending = false;
        pending
    }
}
