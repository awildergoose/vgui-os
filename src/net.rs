use log::trace;
use spin::Mutex;

use crate::{
    bump_frame_allocator::BumpFrameAllocator,
    misc::{get_instant_now, msdelay},
    net::{e1000_hal::MyHal, pci::pci_init, smoltcp_interface::NetworkStack},
};

pub mod e1000_driver;
pub mod e1000_hal;
pub mod pci;
pub mod smoltcp_device;
pub mod smoltcp_interface;

pub static NETWORK_STACK: Mutex<Option<NetworkStack<'_, MyHal>>> = Mutex::new(None);

pub fn init_network(hhdm_offset: u64, frame_allocator: &'static mut BumpFrameAllocator) {
    pci_init();
    trace!("PCI OK");

    let e1000_device = e1000_driver::create_e1000_device(hhdm_offset, frame_allocator);
    trace!("e1000 device OK");

    msdelay(500);

    *NETWORK_STACK.lock() = Some(NetworkStack::new(e1000_device, get_instant_now()));
}
