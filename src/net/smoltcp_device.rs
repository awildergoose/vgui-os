use alloc::{collections::VecDeque, vec, vec::Vec};

use smoltcp::{
    phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken},
    time::Instant,
};

use e1000_driver::e1000::{E1000Device, KernelFunc};

pub struct E1000RxToken {
    packet: Vec<u8>,
}

impl RxToken for E1000RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.packet)
    }
}

pub struct E1000TxToken<'a, 'nic, H: KernelFunc> {
    nic: &'a mut E1000Device<'nic, H>,
}

impl<H: KernelFunc> TxToken for E1000TxToken<'_, '_, H> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut packet = vec![0u8; len];
        let result = f(&mut packet);

        // TODO: should we have some sort of sanity check here for
        // wanted-to-send byte count vs actually-sent byte count?
        let _sent = self.nic.e1000_transmit(&packet);

        result
    }
}

pub struct E1000SmolDevice<'nic, H: KernelFunc> {
    nic: E1000Device<'nic, H>,
    rx_queue: VecDeque<Vec<u8>>,
}

impl<'nic, H: KernelFunc> E1000SmolDevice<'nic, H> {
    #[must_use]
    pub const fn new(nic: E1000Device<'nic, H>) -> Self {
        Self {
            nic,
            rx_queue: VecDeque::new(),
        }
    }

    fn poll_rx(&mut self) {
        if let Some(packets) = self.nic.e1000_recv() {
            self.rx_queue.extend(packets);
        }
    }
}

impl<'nic, H: KernelFunc> Device for E1000SmolDevice<'nic, H> {
    type RxToken<'a>
        = E1000RxToken
    where
        Self: 'a;

    type TxToken<'a>
        = E1000TxToken<'a, 'nic, H>
    where
        Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.poll_rx();

        let packet = self.rx_queue.pop_front()?;

        Some((E1000RxToken { packet }, E1000TxToken { nic: &mut self.nic }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(E1000TxToken { nic: &mut self.nic })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}
