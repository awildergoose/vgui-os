use alloc::vec;
use alloc::vec::Vec;

use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    socket::dns,
    time::Instant,
    wire::{DnsQueryType, EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address},
};

use e1000_driver::e1000::{E1000Device, KernelFunc};

use crate::net::smoltcp_device::E1000SmolDevice;

// Stuff from QEMU defaults (https://www.qemu.org/docs/master/system/devices/net.html#using-the-user-mode-network-stack)
const MAC: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
const IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 15);
const GATEWAY: Ipv4Address = Ipv4Address::new(10, 0, 2, 2);
const DNS_SERVER: Ipv4Address = Ipv4Address::new(10, 0, 2, 3);

pub struct NetworkStack<'nic, H: KernelFunc> {
    device: E1000SmolDevice<'nic, H>,
    iface: Interface,
    sockets: SocketSet<'static>,
    dns_handle: SocketHandle,
}

impl<'nic, H: KernelFunc> NetworkStack<'nic, H> {
    pub fn new(nic: E1000Device<'nic, H>, now: Instant) -> Self {
        let mut device = E1000SmolDevice::new(nic);
        let config = Config::new(HardwareAddress::Ethernet(EthernetAddress(MAC)));
        let mut iface = Interface::new(config, &mut device, now);

        iface.update_ip_addrs(|addrs| addrs.push(IpCidr::new(IpAddress::Ipv4(IP), 24)).unwrap());
        iface
            .routes_mut()
            .add_default_ipv4_route(GATEWAY)
            .expect("failed to add default route");

        let dns_socket = dns::Socket::new(&[IpAddress::Ipv4(DNS_SERVER)], vec![]);
        let mut sockets = SocketSet::new(vec![]);
        let dns_handle = sockets.add(dns_socket);

        Self {
            device,
            iface,
            sockets,
            dns_handle,
        }
    }

    pub fn poll(&mut self, now: Instant) {
        self.iface.poll(now, &mut self.device, &mut self.sockets);
    }

    pub fn dns_query(&mut self, hostname: &str) -> Result<dns::QueryHandle, dns::StartQueryError> {
        let iface = &mut self.iface;
        let sockets = &mut self.sockets;

        sockets.get_mut::<dns::Socket>(self.dns_handle).start_query(
            iface.context(),
            hostname,
            DnsQueryType::A,
        )
    }

    pub fn dns_result(
        &mut self,
        query: dns::QueryHandle,
    ) -> Result<Vec<IpAddress>, dns::GetQueryResultError> {
        self.sockets
            .get_mut::<dns::Socket>(self.dns_handle)
            .get_query_result(query)
            .map(|addresses| addresses.into_iter().collect())
    }
}
