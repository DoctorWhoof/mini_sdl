/// A tiny (4 bytes) struct that contains the state of a "virtual gamepad".
/// Currently mini_sdl simply maps keyboard keys to it, but in the future it
/// will allow actual gamepads and control remapping.

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct GamePad{
    current:u16,
    previous:u16
}

/// A virtual gamepad button. Currently hard coded to keyboard keys, it will be mappable in the future.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum Button {
    None = 0,
    Up = 1,
    Down = 2,
    Left = 4,
    Right = 8,
    A = 16,
    B = 32,
    X = 64,
    Y = 128,
    Start = 256,
    Select = 512,
    LeftTrigger = 1024,
    RightTrigger = 2048,
    LeftShoulder = 4096,
    RightShoulder = 8192,
    Menu = 16384,
}

impl From<u16> for Button {
    fn from(val: u16) -> Self {
        match val {
            1 => Button::Up,
            2 => Button::Down,
            4 => Button::Left,
            8 => Button::Right,
            16 => Button::A,
            32 => Button::B,
            64 => Button::X,
            128 => Button::Y,
            256 => Button::Start,
            512 => Button::Select,
            1024 => Button::LeftTrigger,
            2048 => Button::RightTrigger,
            4096 => Button::LeftShoulder,
            8192 => Button::RightShoulder,
            16384 => Button::Menu,
            _ => Button::None
        }
    }
}

impl GamePad {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, button:Button) -> bool{
        (self.current & button as u16) != 0
    }

    pub fn is_released(&self, button:Button) -> bool{
        (self.current & button as u16) == 0
    }

    pub fn is_just_pressed(&self, button:Button) -> bool{
        self.is_pressed(button) && (self.previous & button as u16 == 0)
    }

    pub fn is_just_released(&self, button:Button) -> bool{
        !self.is_pressed(button) && (self.previous & button as u16 != 0)
    }

    pub fn state_to_str(&self) -> &'static str {
        let state:Button = self.current.into();
        match state {
            Button::None => "None or Multiple",
            Button::Up => "Up",
            Button::Down => "Down",
            Button::Left => "Left",
            Button::Right => "Right",
            Button::A => "A",
            Button::B => "B",
            Button::X => "X",
            Button::Y => "Y",
            Button::Start => "Start",
            Button::Select => "Select",
            Button::LeftTrigger => "LeftTrigger",
            Button::RightTrigger => "RightTrigger",
            Button::LeftShoulder => "LeftShoulder",
            Button::RightShoulder => "RightShoulder",
            Button::Menu => "Menu",
        }
    }

    pub(crate) fn set(&mut self, button:Button, value:bool){
        if value {
            self.current |= button as u16;
        } else {
            self.current &= !(button as u16);
        }
    }

    pub(crate) fn copy_current_to_previous_state(&mut self){
        self.previous = self.current;
    }
}
