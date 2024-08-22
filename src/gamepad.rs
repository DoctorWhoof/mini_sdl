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
}

impl GamePad {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, button:Button) -> bool{
        (self.current & button as u16) != 0
    }

    pub fn is_just_pressed(&self, button:Button) -> bool{
        self.is_pressed(button) && (self.previous & button as u16 == 0)
    }

    pub(crate) fn set(&mut self, button:Button, value:bool){
        if value {
            self.current |= button as u16;
        } else {
            self.current &= !(button as u16);
        }
    }

    pub(crate) fn set_previous_state(&mut self){
        self.previous = self.current;
    }
}
