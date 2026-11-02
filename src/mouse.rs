use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use ps2::error::ControllerError;
use ps2::flags::ControllerConfigFlags;

use crate::{ps2::CONTROLLER, serial_println};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseResult {
    pub flags: MouseFlags,
    pub dx: i16,
    pub dy: i16,
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
    pub struct MouseFlags: u8 {
        // why does the OSDev Wiki list this so weirdly
        const Y_OVERFLOW = 1 << 7;
        const X_OVERFLOW = 1 << 6;
        const Y_NEGATIVE = 1 << 5;
        const X_NEGATIVE = 1 << 4;
        const ALWAYS_ONE = 1 << 3;
        const MIDDLE      = 1 << 2;
        const RIGHT       = 1 << 1;
        const LEFT        = 1 << 0;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct MousePacket {
    pub flags: MouseFlags,
    pub dx: u8,
    pub dy: u8,
}

pub fn init_mouse() -> Result<(), ControllerError> {
    let mut controller = CONTROLLER.lock();

    // First, disable the device
    controller.disable_mouse()?;

    // Now, flush the data buffer
    let _ = controller.read_data();

    // It's time for us to write our config!
    let mut config = controller.read_config()?;

    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT,
        false,
    );
    config.set(ControllerConfigFlags::ENABLE_TRANSLATE, true);
    controller.write_config(config)?;

    controller.test_controller()?;

    // Write the config again incase the controller resets it
    controller.write_config(config)?;

    // Now, enable the mouse
    controller.enable_mouse()?;
    config = controller.read_config()?;

    // Check if the mouse actually got enabled!
    if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
        serial_println!("Mouse port unavailable");
        return Ok(());
    }

    // Test the mouse!
    controller.disable_mouse()?;
    controller.test_mouse()?;
    controller.enable_mouse()?;

    config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
    config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, false);

    controller
        .mouse()
        .reset_and_self_test()
        .expect("mouse reset failed");
    controller
        .mouse()
        .enable_data_reporting()
        .expect("mouse reporting failed");

    controller.write_config(config)?;

    Ok(())
}

pub fn poll_mouse() -> Option<MouseResult> {
    static mut PACKET: [u8; 3] = [0; 3];
    static mut PACKET_INDEX: usize = 0;

    let byte = crate::ps2::read_mouse()?;

    unsafe {
        // Check if there's anything in the queue yet
        if PACKET_INDEX == 0 && byte & MouseFlags::ALWAYS_ONE.bits() == 0 {
            return None;
        }

        // Set the byte, and increment the counter
        PACKET[PACKET_INDEX] = byte;
        PACKET_INDEX += 1;

        // Have we reached the end of the packet?
        if PACKET_INDEX == 3 {
            let packet: MousePacket = bytemuck::cast(PACKET);
            let dx = if packet.flags.contains(MouseFlags::X_NEGATIVE) {
                i16::from(packet.dx) - 256
            } else {
                i16::from(packet.dx)
            };
            let dy = -if packet.flags.contains(MouseFlags::Y_NEGATIVE) {
                i16::from(packet.dy) - 256
            } else {
                i16::from(packet.dy)
            };

            // reset the counter!
            PACKET_INDEX = 0;

            return Some(MouseResult {
                flags: packet.flags,
                dx,
                dy,
            });
        }
    }

    None
}
