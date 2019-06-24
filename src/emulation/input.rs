use crate::emulation::bitutils::BitExtensions;

const RIGHT_OR_A_BIT: u8 = 0;
const LEFT_OR_B_BIT: u8 = 1;
const UP_OR_SELECT_BIT: u8 = 2;
const DOWN_OR_START_BIT: u8 = 3;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ButtonSet {
    None,
    Arrows,
    Actions
}

fn encode_joypad_state(state: InputState, selected_set: ButtonSet) -> u8 {
    let mut value = match selected_set {
        ButtonSet::None => 0b1111_1111,
        ButtonSet::Arrows => 0b1110_1111,
        ButtonSet::Actions => 0b1101_1111
    };

    if (state.right && selected_set == ButtonSet::Arrows)
        || (state.a && selected_set == ButtonSet::Actions)
    {
        value = value.clear_bit(RIGHT_OR_A_BIT);
    }

    if (state.left && selected_set == ButtonSet::Arrows)
        || (state.b && selected_set == ButtonSet::Actions)
    {
        value = value.clear_bit(LEFT_OR_B_BIT);
    }

    if (state.up && selected_set == ButtonSet::Arrows)
        || (state.select && selected_set == ButtonSet::Actions)
    {
        value = value.clear_bit(UP_OR_SELECT_BIT);
    }

    if (state.down && selected_set == ButtonSet::Arrows)
        || (state.start && selected_set == ButtonSet::Actions)
    {
        value = value.clear_bit(DOWN_OR_START_BIT);
    }

    value
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
        let masked = value & 0b0011_0000;
        self.selected_set = match masked {
            0b0010_0000 => ButtonSet::Arrows,
            0b0001_0000 => ButtonSet::Actions,
            _ => ButtonSet::None
        };
    }

    pub fn read_8(&self) -> u8 {
        encode_joypad_state(self.input_state, self.selected_set)
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
