pub enum JoypadButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

pub const SELECT_BUTTON_BIT: u8 = 0x20;
pub const SELECT_DIRECTION_BIT: u8 = 0x10;

pub const JOYPAD_RIGHT_BIT: u8 = 0x01;
pub const JOYPAD_LEFT_BIT: u8 = 0x02;
pub const JOYPAD_UP_BIT: u8 = 0x04;
pub const JOYPAD_DOWN_BIT: u8 = 0x08;
pub const JOYPAD_A_BIT: u8 = 0x01;
pub const JOYPAD_B_BIT: u8 = 0x02;
pub const JOYPAD_SELECT_BIT: u8 = 0x04;
pub const JOYPAD_START_BIT: u8 = 0x08;

pub struct Joypad {
    select_buttons: u8,

    direction_buttons: u8,
    action_buttons: u8,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            select_buttons: 0x30,

            direction_buttons: 0x0F,
            action_buttons: 0x0F,
        }
    }

    pub fn press_button(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Right => self.direction_buttons &= !JOYPAD_RIGHT_BIT,
            JoypadButton::Left => self.direction_buttons &= !JOYPAD_LEFT_BIT,
            JoypadButton::Up => self.direction_buttons &= !JOYPAD_UP_BIT,
            JoypadButton::Down => self.direction_buttons &= !JOYPAD_DOWN_BIT,

            JoypadButton::A => self.action_buttons &= !JOYPAD_A_BIT,
            JoypadButton::B => self.action_buttons &= !JOYPAD_B_BIT,
            JoypadButton::Select => self.action_buttons &= !JOYPAD_SELECT_BIT,
            JoypadButton::Start => self.action_buttons &= !JOYPAD_START_BIT,
        }
    }

    pub fn release_button(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Right => self.direction_buttons |= JOYPAD_RIGHT_BIT,
            JoypadButton::Left => self.direction_buttons |= JOYPAD_LEFT_BIT,
            JoypadButton::Up => self.direction_buttons |= JOYPAD_UP_BIT,
            JoypadButton::Down => self.direction_buttons |= JOYPAD_DOWN_BIT,

            JoypadButton::A => self.action_buttons |= JOYPAD_A_BIT,
            JoypadButton::B => self.action_buttons |= JOYPAD_B_BIT,
            JoypadButton::Select => self.action_buttons |= JOYPAD_SELECT_BIT,
            JoypadButton::Start => self.action_buttons |= JOYPAD_START_BIT,
        }
    }

    pub fn read(&self) -> u8 {
        let mut result: u8 = 0xFF;

        if (self.select_buttons & SELECT_BUTTON_BIT) == 0 {
            result &= self.action_buttons | 0xF0;
        } else if (self.select_buttons & SELECT_DIRECTION_BIT) == 0 {
            result &= self.direction_buttons | 0xF0;
        }

        return result;
    }

    pub fn write(&mut self, value: u8) {
        self.select_buttons = value & 0x30;
    }
}
