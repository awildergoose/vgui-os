use core::{alloc::Layout, hint::spin_loop, time::Duration};

use log::trace;

use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
};

use crate::{
    bump_frame_allocator::BumpFrameAllocator,
    cmos::CMOS,
    net_pci::{map_e1000, pci_init},
};

struct MyHal {
    hhdm_offset: usize,
}

impl e1000_driver::e1000::KernelFunc for MyHal {
    const PAGE_SIZE: usize = 4 << 10; // the Odin discord B)

    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize) {
        let bytes = Self::PAGE_SIZE * pages;
        let layout = Layout::from_size_align(bytes, Self::PAGE_SIZE).expect("bad DMA layout");
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };

        if ptr.is_null() {
            alloc::alloc::handle_alloc_error(layout);
        }

        let vaddr = ptr as usize;
        let paddr = vaddr
            .checked_sub(self.hhdm_offset)
            .expect("out of bounds dma alloc");

        (vaddr, paddr)
    }

    fn dma_free_coherent(&mut self, vaddr: usize, pages: usize) {
        trace!("todo: dma_free({vaddr}/{pages})");
    }
}

// if t < 1000 this will be imprecise
fn msdelay(t: u64) {
    CMOS.with_mut(|cmos| {
        let start = cmos.get_time();
        let end = start + Duration::from_millis(t);

        while cmos.get_time() < end {
            spin_loop();
        }
    });
}

unsafe fn active_level_4_table(hhdm_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();

    let virt = hhdm_offset + phys.as_u64();
    let ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *ptr }
}

// MAC 52:54:00:12:34:56
const TEST_ARP_REQUEST: [u8; 42] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x08, 0x06, 0x00, 0x01,
    0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x0a, 0x00, 0x02, 0x0f,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x02, 0x02,
]; // ping 10.0.2.2

pub fn init_network(hhdm_offset: u64, frame_allocator: &mut BumpFrameAllocator) {
    pci_init();
    trace!("PCI OK");

    let hal = MyHal {
        hhdm_offset: hhdm_offset as usize,
    };

    let page_table = unsafe { active_level_4_table(VirtAddr::new(hhdm_offset)) };
    let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(hhdm_offset)) };
    let e1000_virt = map_e1000(&mut mapper, frame_allocator, hhdm_offset);
    let mut e1000_device =
        e1000_driver::e1000::E1000Device::<MyHal>::new(hal, e1000_virt.try_into().unwrap())
            .unwrap();

    trace!("device made!");
    msdelay(500);

    let sent = e1000_device.e1000_transmit(&TEST_ARP_REQUEST);
    trace!("sent {sent} bytes");

    for _ in 0..12 {
        if let Some(packets) = e1000_device.e1000_recv() {
            for packet in packets {
                trace!("received {} bytes!", packet.len());
            }
        }

        msdelay(100);
    }
}
