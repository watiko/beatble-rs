use bitflags::bitflags;

bitflags! {
    pub struct NormalButton: u8 {
        const B1 = 0b00000001;
        const B2 = 0b00000010;
        const B3 = 0b00000100;
        const B4 = 0b00001000;
        const B5 = 0b00010000;
        const B6 = 0b00100000;
        const B7 = 0b01000000;
    }
}

bitflags! {
    pub struct OptionButton: u8 {
        const E1 = 0b0001;
        const E2 = 0b0010;
        const E3 = 0b0100;
        const E4 = 0b1000;
    }
}

#[repr(align(4))] // for AtomicCell
#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    pub scratch: u8,
    pub normal_button: NormalButton,
    pub option_button: OptionButton,
}

impl Default for KeyInput {
    fn default() -> Self {
        Self {
            scratch: 0,
            normal_button: NormalButton::empty(),
            option_button: OptionButton::empty(),
        }
    }
}

impl KeyInput {
    pub fn init() -> Self {
        Self {
            scratch: 0x00,
            normal_button: NormalButton::empty(),
            option_button: OptionButton::empty(),
        }
    }

    pub fn to_payload(&self, counter: u8) -> [u8; 10] {
        [
            self.scratch,
            0x00,
            self.normal_button.bits,
            self.option_button.bits,
            counter,
            self.scratch,
            0x00,
            self.normal_button.bits,
            self.option_button.bits,
            counter + 1,
        ]
    }
}
