use log::{error, info};

use x86_64::{
    PhysAddr, VirtAddr,
    instructions::port::{Port, PortWriteOnly},
    structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
};

use crate::bump_frame_allocator::BumpFrameAllocator;

struct Pci {
    address_port: PortWriteOnly<u32>,
    data_port: Port<u32>,
}

impl Pci {
    #[must_use]
    pub const fn new(address_port: u16, data_port: u16) -> Self {
        Self {
            address_port: PortWriteOnly::<u32>::new(address_port),
            data_port: Port::<u32>::new(data_port),
        }
    }

    fn locate(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        // Fucking what the fuck
        0x8000_0000
            | (u32::from(bus) << 16)
            | (u32::from(device) << 11)
            | (u32::from(function) << 8)
            | (u32::from(offset) & 0xFC)
    }

    pub fn read_u32(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address = Self::locate(bus, device, function, offset);

        unsafe {
            self.address_port.write(address);
            self.data_port.read()
        }
    }

    pub fn write_u32(&mut self, bus: u8, device: u8, function: u8, offset: u8, value: u32) {
        let address = Self::locate(bus, device, function, offset);

        unsafe {
            self.address_port.write(address);
            self.data_port.write(value);
        }
    }
}

// in QEMU, the physical address of E1000 seems to always be this ^-^
const E1000_PHYS: u64 = 0xFEBC_0000;
const E1000_SIZE: u64 = 0x2_0000;

pub fn map_e1000<M>(
    mapper: &mut M,
    frame_allocator: &mut BumpFrameAllocator,
    hhdm_offset: u64,
) -> u64
where
    M: Mapper<Size4KiB>,
{
    let virt_base = hhdm_offset + E1000_PHYS;

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_CACHE
        | PageTableFlags::WRITE_THROUGH;

    for offset in (0..E1000_SIZE).step_by(4096) {
        let page = Page::containing_address(VirtAddr::new(virt_base + offset));
        let frame = PhysFrame::containing_address(PhysAddr::new(E1000_PHYS + offset));

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("failed to map E1000 MMIO")
                .flush();
        }
    }

    virt_base
}

pub fn pci_init() {
    let mut pci = Pci::new(0xCF8, 0xCFC);
    let mut e1000 = None;

    for device in 0u8..32 {
        let id = pci.read_u32(0, device, 0, 0);
        let vendor = id as u16;
        let device_id = (id >> 16) as u16;

        if vendor == 0xFFFF {
            continue; // No device!
        }

        info!("PCI device={device:#x} vendor={vendor:#06x} device={device_id:#06x}");

        // QEMU's e1000 (E1000_DEV)
        if vendor == 0x8086 && device_id == 0x100E {
            e1000 = Some(device);
        }
    }

    if let Some(e1000) = e1000 {
        info!("e1000 device: {e1000:?}");

        let command = pci.read_u32(0, e1000, 0, 0x04) as u16;
        let command = command | (1 << 1) | (1 << 2);
        pci.write_u32(0, e1000, 0, 0x04, u32::from(command));
    } else {
        error!("failed to find e1000");
    }
}
