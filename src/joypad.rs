bitflags::bitflags! {
    #[derive(Copy, Clone)]
    pub struct JoypadButton: u8 {
        const A = 0b0000_0001;
        const B = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START = 0b0000_1000;
        const UP = 0b0001_0000;
        const DOWN = 0b0010_0000;
        const LEFT = 0b0100_0000;
        const RIGHT = 0b1000_0000;
    }
}

pub struct Joypad {
    strobe: bool,
    button_index: u8,
    pub button_states: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            strobe: false,
            button_index: 0,
            button_states: JoypadButton::empty(),
        }
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        let response = (self.button_states.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }

        response
    }

    pub fn set_button_state(&mut self, button: JoypadButton, pressed: bool) {
        self.button_states.set(button, pressed);
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}
