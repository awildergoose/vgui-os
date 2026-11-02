use pc_keyboard::{HandleControl, KeyEvent, PS2Keyboard, ScancodeSet1, layouts};
use spin::{LazyLock, Mutex};

type ScancodeSet = ScancodeSet1;
type Layout = layouts::Us104Key;

pub struct Keyboard {
    kb: PS2Keyboard<Layout, ScancodeSet>,
}

impl Keyboard {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            kb: PS2Keyboard::new(ScancodeSet::new(), Layout {}, HandleControl::Ignore),
        }
    }

    pub fn poll(&mut self) -> Option<KeyEvent> {
        let scancode = crate::ps2::read_keyboard()?;
        // let's hope nothing bad happens here
        self.kb.add_byte(scancode).ok().flatten()
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

pub static KEYBOARD: LazyLock<Mutex<Keyboard>> = LazyLock::new(|| Mutex::new(Keyboard::new()));
