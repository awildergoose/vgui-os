use bitflags::bitflags;
use ps2::Controller;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub static CONTROLLER: Mutex<Controller> = Mutex::new(unsafe { Controller::new() });

// PS/2 Controller IO Status Register
static STATUS: Mutex<Port<u8>> = Mutex::new(Port::new(0x64));

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusRegister: u8 {
        const OUTPUT_BUFFER_STATUS = 1 << 0;
        const INPUT_BUFFER_STATUS  = 1 << 1;
        const SYSTEM_FLAG          = 1 << 2;
        const COMMAND_DATA         = 1 << 3;
        const KEYBOARD_LOCK        = 1 << 4;
        // From OSDev:
        // ```
        // Unknown (chipset specific)
        // May be "receive time-out" or "second PS/2 port output buffer full"
        // ```
        // What the fuck?
        const IS_THIS_A_KEYBOARD   = 1 << 5;
        const TIMEOUT_ERROR        = 1 << 6;
        const PARITY_ERROR         = 1 << 7;
    }
}

fn read_device(keyboard: bool) -> Option<u8> {
    unsafe {
        let status = StatusRegister::from_bits_truncate(STATUS.lock().read());

        // nothing there!
        if !status.contains(StatusRegister::OUTPUT_BUFFER_STATUS) {
            return None;
        }

        if status.contains(StatusRegister::IS_THIS_A_KEYBOARD) == keyboard {
            return None;
        }

        CONTROLLER.lock().read_data().ok()
    }
}

#[must_use]
pub fn read_keyboard() -> Option<u8> {
    read_device(true)
}

#[must_use]
pub fn read_mouse() -> Option<u8> {
    read_device(false)
}
