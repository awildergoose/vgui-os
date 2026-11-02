use log::{info, trace, warn};

use crate::{
    bump_frame_allocator::BumpFrameAllocator,
    misc::{get_instant_now, msdelay},
    net::smoltcp_interface::NetworkStack,
    net_pci::pci_init,
};

pub mod e1000_driver;
pub mod e1000_hal;
pub mod smoltcp_device;
pub mod smoltcp_interface;

pub fn init_network(hhdm_offset: u64, frame_allocator: &mut BumpFrameAllocator) {
    pci_init();
    trace!("PCI OK");

    let e1000_device = e1000_driver::create_e1000_device(hhdm_offset, frame_allocator);
    trace!("e1000 device OK");

    msdelay(500);

    let mut stack = NetworkStack::new(e1000_device, get_instant_now());
    let query = stack
        .dns_query("google.com")
        .expect("failed to query google.com");

    for _ in 0..10 {
        stack.poll(get_instant_now());

        let res = stack.dns_result(query);

        if let Ok(res) = res {
            info!("dns result: {res:#?}");
            break;
        } else if let Err(res) = res {
            warn!("dns error: {res:#?}");
        }

        msdelay(100);
    }
}
