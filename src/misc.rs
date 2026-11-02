use core::panic::PanicInfo;

use alloc::format;
use limine::{
    BaseRevision, RequestsEndMarker, RequestsStartMarker,
    request::{FramebufferRequest, HhdmRequest, MemmapRequest},
};
use linked_list_allocator::LockedHeap;
use x86_64::instructions::hlt;

use crate::{
    color::Color,
    framebuffer::{clear_framebuffer, swap_buffers},
    serial_println,
    text::{
        draw_text, draw_text_formatted, get_formatted_text_height, get_formatted_text_width,
        get_text_width,
    },
};

#[used]
#[unsafe(link_section = ".requests_start_marker")]
pub static REQUEST_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
pub static REQUEST_END: RequestsEndMarker = RequestsEndMarker::new();

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub unsafe fn init_allocator(virtual_start: *mut u8, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(virtual_start, heap_size);
    }
}

pub fn halt() -> ! {
    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    fatal_fail(&format!("PANIC: {info:#?}"));
}

pub fn fatal_fail(big_text: &str) -> ! {
    // Gotta make it die in style B)
    serial_println!();
    serial_println!("{big_text}");

    clear_framebuffer(Color::BLUE);

    let base_y = 125;
    let ttext = "FUCK FUCK FUCK FUCK FUCK FUCK FUCK FUCK FUCK FUCK";
    let tmeasured = get_text_width(ttext);
    let tx = (800 - tmeasured) / 2;

    for y in 0..4 {
        draw_text(tx, base_y + (8 * y), ttext, Color::WHITE);
    }

    let measured = get_formatted_text_width(big_text);
    draw_text_formatted((800 - measured) / 2, base_y + 40, big_text, Color::WHITE);

    let measured_y = get_formatted_text_height(big_text);
    for y in 0..4 {
        draw_text(
            tx,
            measured_y + base_y + 40 + (8 * y) + 16,
            ttext,
            Color::WHITE,
        );
    }

    swap_buffers();
    halt();
}
