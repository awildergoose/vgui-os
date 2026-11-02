#![feature(array_ptr_get)]
#![feature(core_float_math)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![no_std]
#![no_main]

use alloc::format;
use alloc::string::String;
use core::hint::spin_loop;
use limine::memmap::MEMMAP_USABLE;

use pc_keyboard::{KeyCode, KeyState};

use crate::cmos::CMOS;
use crate::framebuffer::swap_buffers;
use crate::misc::{
    BASE_REVISION, FRAMEBUFFER_REQUEST, HHDM_REQUEST, MEMORY_MAP_REQUEST, halt, init_allocator,
};
use crate::{
    color::Color,
    framebuffer::{clear_framebuffer, init_fb},
    idt::init_idt,
    keyboard::KEYBOARD,
    mouse::{MouseFlags, init_mouse, poll_mouse},
    text::{draw_char, draw_text},
};

extern crate alloc;

pub mod cmos;
pub mod color;
pub mod font;
pub mod framebuffer;
pub mod idt;
pub mod keyboard;
pub mod misc;
pub mod mouse;
pub mod net;
pub mod ps2;
pub mod serial;
pub mod text;
pub mod wrapped_sync_cell;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    assert!(
        BASE_REVISION.is_supported(),
        "BASE_REVISION is not supported!"
    );

    let hhdm_response = HHDM_REQUEST.response().expect("HHDM unavailable");
    let hhdm_offset = hhdm_response.offset;

    serial_println!("HHDM OK");

    let mmap_response = MEMORY_MAP_REQUEST.response().expect("MMAP unavailable");
    let usable_entry = mmap_response
        .entries()
        .iter()
        .find(|entry| entry.type_ == MEMMAP_USABLE)
        .expect("no usable memory region found for heap");

    serial_println!("MMAP OK");

    let physical_start = usable_entry.base;
    let heap_size = usable_entry.length as usize;

    let virtual_start = (physical_start + hhdm_offset) as *mut u8;

    serial_println!("MEM START: {virtual_start:?}");
    serial_println!("MEM SIZE : {heap_size}");

    unsafe { init_allocator(virtual_start, heap_size) };
    serial_println!("ALLOCATOR OK");

    init_idt();
    serial_println!("IDT OK");

    let response = FRAMEBUFFER_REQUEST
        .response()
        .expect("Framebuffer unavailable");

    let fb = response.framebuffers().first().expect("No framebuffer");

    assert!(
        !(fb.bpp != 32 || fb.memory_model != limine::framebuffer::FRAMEBUFFER_RGB),
        "unexpected fb bp / memory model: bp={} mm={}",
        fb.bpp,
        fb.memory_model
    );

    serial_println!("FB OK");

    init_fb(fb);
    serial_println!("FB INIT");
    clear_framebuffer(Color::GRAY);
    serial_println!("FB CLEAR");

    init_mouse().expect("failed to init mouse");
    serial_println!("MOUSE OK");

    serial_println!("CMOS: {:?}", CMOS.lock().get_time());

    // init_network().expect("failed to init network");
    // serial_println!("NET OK");

    let mut mouse_x = 0;
    let mut mouse_y = 0;
    let mut text = String::new();

    loop {
        clear_framebuffer(Color::GRAY);

        if let Some(key) = KEYBOARD.lock().poll() {
            serial_println!("key: {key:?}");

            if key.state == KeyState::Down {
                if key.code == KeyCode::Backspace {
                    text.pop();
                } else {
                    let key_str = match key.code {
                        KeyCode::LShift
                        | KeyCode::RShift
                        | KeyCode::RControl
                        | KeyCode::LControl => "",
                        KeyCode::OemComma => ",",
                        KeyCode::Spacebar => " ",
                        _ => &format!("{:?}", key.code),
                    };

                    text += key_str;
                }
            }

            if key.code == KeyCode::Escape {
                break;
            }
        }

        if let Some(res) = poll_mouse() {
            serial_println!("mouse: {res:?}");

            mouse_x += res.dx;
            mouse_y += res.dy;

            if res.flags.contains(MouseFlags::LEFT) {
                clear_framebuffer(Color::BLACK);
            }
        }

        draw_text(
            0,
            0,
            format!("Time: {:?}", CMOS.lock().get_time()),
            Color::WHITE,
        );
        draw_text(100, 100, &text, Color::GREEN);
        draw_char(mouse_x as usize, mouse_y as usize, b'0', Color::ORANGE);
        swap_buffers();

        spin_loop();
    }

    halt()
}
