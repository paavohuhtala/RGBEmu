bitflags! {
  pub struct JoypadRegister: u8 {
    const RightOrA    = 0b0000_0001;
    const LeftOrB     = 0b0000_0010;
    const UpOrSelect  = 0b0000_0100;
    const DownOrStart = 0b0000_1000;
    const UseSet1     = 0b0001_0000;
    const UseSet2     = 0b0010_0000;
    const Unused      = 0b1100_0000;
  }
}

#[derive(Debug, Clone, Copy)]
enum ButtonSet {
    None,
    Arrows,
    Actions
}

impl JoypadRegister {
    fn from_input_state(state: InputState, selected_set: ButtonSet) -> JoypadRegister {
        let mut base = JoypadRegister::from_bits(0b1100_1111).unwrap();
        match selected_set {
            ButtonSet::Arrows => {
                if state.up {
                    base.remove(JoypadRegister::UpOrSelect);
                }
                if state.down {
                    base.remove(JoypadRegister::DownOrStart);
                }
                if state.left {
                    base.remove(JoypadRegister::LeftOrB);
                }
                if state.right {
                    base.remove(JoypadRegister::RightOrA);
                }
            }
            ButtonSet::Actions => {
                if state.a {
                    base.remove(JoypadRegister::RightOrA);
                }
                if state.b {
                    base.remove(JoypadRegister::LeftOrB);
                }
                if state.select {
                    base.remove(JoypadRegister::UpOrSelect);
                }
                if state.start {
                    base.remove(JoypadRegister::DownOrStart);
                }
            }
            ButtonSet::None => ()
        }

        base
    }
}

pub struct InputRegister {
    input_state: InputState,
    selected_set: ButtonSet
}

impl InputRegister {
    pub fn new() -> InputRegister {
        InputRegister {
            input_state: InputState::default(),
            selected_set: ButtonSet::None
        }
    }

    pub fn write_8(&mut self, value: u8) {
        match value {
            _ if value & 0b01_0000 == 0 => self.selected_set = ButtonSet::Arrows,
            _ if value & 0b10_0000 == 0 => self.selected_set = ButtonSet::Actions,
            _ => return
        }
    }

    pub fn read_8(&self) -> u8 {
        JoypadRegister::from_input_state(self.input_state, self.selected_set).bits()
    }

    pub fn update(&mut self, new_state: InputState) {
        self.input_state = new_state;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool
}

trait InputHandler {
    fn get_input_state(&self) -> InputState;
}
