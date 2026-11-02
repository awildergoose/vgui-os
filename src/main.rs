#![feature(abi_x86_interrupt)]
#![feature(core_float_math)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)]
#![no_std]
#![no_main]

use alloc::format;
use alloc::string::String;
use core::hint::spin_loop;
use limine::memmap::MEMMAP_USABLE;
use log::{info, trace};

use pc_keyboard::{KeyCode, KeyState};

use crate::bump_frame_allocator::BumpFrameAllocator;
use crate::cmos::CMOS;
use crate::framebuffer::swap_buffers;
use crate::logger::SERIAL_LOGGER;
use crate::misc::{
    BASE_REVISION, FRAMEBUFFER_REQUEST, HHDM_REQUEST, MEMORY_MAP_REQUEST, halt, init_allocator,
};
use crate::net::init_network;
use crate::{
    color::Color,
    framebuffer::{clear_framebuffer, init_fb},
    idt::init_idt,
    keyboard::KEYBOARD,
    mouse::{MouseFlags, init_mouse, poll_mouse},
    text::{draw_char, draw_text},
};

extern crate alloc;

pub mod bump_frame_allocator;
pub mod cmos;
pub mod color;
pub mod font;
pub mod framebuffer;
pub mod idt;
pub mod keyboard;
pub mod logger;
pub mod misc;
pub mod mouse;
pub mod net;
pub mod net_pci;
pub mod ps2;
pub mod serial;
pub mod text;
pub mod wrapped_sync_cell;

#[unsafe(no_mangle)]
#[allow(clippy::too_many_lines)]
pub extern "C" fn _start() -> ! {
    assert!(
        BASE_REVISION.is_supported(),
        "BASE_REVISION is not supported!"
    );

    let hhdm_response = HHDM_REQUEST.response().expect("HHDM unavailable");
    let hhdm_offset = hhdm_response.offset;

    serial_println!("HHDM OK");

    let mmap_response = MEMORY_MAP_REQUEST.response().expect("MMAP unavailable");
    let heap_entry = mmap_response
        .entries()
        .iter()
        .filter(|entry| entry.type_ == MEMMAP_USABLE)
        .max_by_key(|e| e.length)
        .expect("no usable memory region found for heap");
    let frame_entry = mmap_response
        .entries()
        .iter()
        .filter(|entry| entry.base != heap_entry.base && entry.type_ == MEMMAP_USABLE)
        .max_by_key(|entry| entry.length)
        .expect("no usable memory region found for frame allocator");

    serial_println!(
        "MMAP OK ({} usable entries)",
        mmap_response
            .entries()
            .iter()
            .filter(|entry| entry.type_ == MEMMAP_USABLE)
            .count(),
    );

    let physical_start = heap_entry.base;
    let heap_size = heap_entry.length as usize;

    let virtual_start = (physical_start + hhdm_offset) as *mut u8;

    serial_println!("MEM START: {virtual_start:?}");
    serial_println!("MEM SIZE : {heap_size}");

    unsafe { init_allocator(virtual_start, heap_size) };
    serial_println!("ALLOCATOR OK");

    serial_println!("FRAME ALLOCATOR BASE: {}", frame_entry.base);
    serial_println!("FRAME ALLOCATOR SIZE: {}", frame_entry.length);

    let mut frame_allocator = BumpFrameAllocator::new(frame_entry.base, frame_entry.length);

    serial_println!("FRAME ALLOCATOR OK");

    log::set_logger(&SERIAL_LOGGER).expect("setting logger");
    log::set_max_level(log::LevelFilter::Trace);
    info!("LOGGER OK");

    init_idt();
    info!("IDT OK");

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

    info!("FB OK");

    init_fb(fb);
    info!("FB INIT");
    clear_framebuffer(Color::GRAY);
    info!("FB CLEAR");

    init_mouse().expect("failed to init mouse");
    info!("MOUSE OK");

    info!("CMOS: {:?}", CMOS.with_mut(cmos::CMOS::get_time));

    init_network(hhdm_offset, &mut frame_allocator);
    info!("NET OK");

    let mut mouse_x = 0u16;
    let mut mouse_y = 0u16;
    let mut mouse_l = false;
    let mut text = String::new();

    loop {
        clear_framebuffer(if mouse_l { Color::BLACK } else { Color::GRAY });

        if let Some(key) = KEYBOARD.lock().poll() {
            trace!("key: {key:?}");

            if key.state == KeyState::Down {
                if key.code == KeyCode::Backspace {
                    text.pop();
                } else {
                    let key_str = match key.code {
                        KeyCode::LShift
                        | KeyCode::RShift
                        | KeyCode::RControl
                        | KeyCode::LControl
                        | KeyCode::LAlt => "",
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
            trace!("mouse: {res:?}");

            mouse_x = (mouse_x.cast_signed() + res.dx).max(0).min(fb.width as i16) as u16;
            mouse_y = (mouse_y.cast_signed() + res.dy)
                .max(0)
                .min(fb.height as i16) as u16;
            mouse_l = res.flags.contains(MouseFlags::LEFT);
        }

        let t = CMOS.with_mut(cmos::CMOS::get_time);
        draw_text(0, 0, format!("Time: {t:?}"), Color::WHITE);
        draw_text(100, 100, &text, Color::GREEN);
        draw_char(mouse_x as usize, mouse_y as usize, b'0', Color::ORANGE);
        swap_buffers();

        spin_loop();
    }

    halt()
}
