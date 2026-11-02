use core::panic::PanicInfo;

use limine::{
    BaseRevision, RequestsEndMarker, RequestsStartMarker,
    request::{FramebufferRequest, HhdmRequest, MemmapRequest},
};
use linked_list_allocator::LockedHeap;
use x86_64::instructions::hlt;

use crate::serial_println;

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
    serial_println!("PANIC: {info:#?}");
    halt()
}
