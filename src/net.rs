use log::{info, trace, warn};
use smoltcp::wire::IpEndpoint;

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
        .dns_query("irc.libera.chat")
        .expect("dns query failed");
    let mut result = None;

    for _ in 0..10 {
        stack.poll(get_instant_now());

        let res = stack.dns_result(query);

        if let Ok(res) = res {
            info!("dns result: {res:#?}");
            result = res.first().copied();
            break;
        } else if let Err(res) = res {
            warn!("dns error: {res:#?}");
        }

        msdelay(100);
    }

    if let Some(address) = result {
        msdelay(2000);

        let handle = stack.tcp_connect(IpEndpoint::new(address, 6667));

        while !stack.tcp_may_send(handle) {
            stack.poll(get_instant_now());
            trace!(
                "waiting until we can send... (state: {})",
                stack.tcp_state(handle)
            );
            msdelay(100);
        }

        trace!("sending nick!");
        stack.tcp_send(handle, b"NICK vguios\r\n");
        stack.tcp_send(handle, b"USER vguios 0 * :Vguios\r\n");

        trace!("waiting 5 seconds for that one stupidassm essage!");
        for _ in 0..5 {
            stack.poll(get_instant_now());
            msdelay(1000);
        }
        trace!("wow!");
        msdelay(5000);
        trace!("wait 5 more!");
        stack.poll(get_instant_now());
        trace!("polled again!");
        stack.tcp_send(handle, b"JOIN #testingvgui\r\n");
        trace!("SEND!!");
        for _ in 0..5 {
            stack.poll(get_instant_now());
            msdelay(1000);
        }
    }
}
