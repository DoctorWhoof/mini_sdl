#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct GamePad{
    current:u16,
    previous:u16
}

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


// Old style, one getter/setter per bit
// macro_rules! flag {
//     ($getter_name:ident, $setter_name:ident, $bit:expr) => {
//         impl GamePad {
//             /// Getter method to check if the bit is set.
//             pub fn $getter_name(&self) -> bool {
//                 (self.0 & (1 << $bit)) != 0
//             }

//             /// Setter method to set or clear the bit.
//             pub fn $setter_name(&mut self, value: bool) {
//                 if value {
//                     self.0 |= 1 << $bit;
//                 } else {
//                     self.0 &= !(1 << $bit);
//                 }
//             }
//         }
//     };
// }
// flag!(up, set_up, 0);
// flag!(down, set_down, 1);
// flag!(left, set_left, 2);
// flag!(right, set_right, 3);
// flag!(button_a, set_button_a, 4);
// flag!(button_b, set_button_b, 5);
// flag!(button_x, set_button_x, 6);
// flag!(button_y, set_button_y, 7);
// flag!(start, set_start, 8);
// flag!(select, set_select, 9);
