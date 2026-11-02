use e1000_driver::e1000::E1000Device;
use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
};

use crate::{bump_frame_allocator::BumpFrameAllocator, net::e1000_hal::MyHal, net::pci::map_e1000};

unsafe fn active_page_table(hhdm_offset: VirtAddr) -> &'static mut PageTable {
    let (l4_frame, _) = Cr3::read();
    let phys = l4_frame.start_address();

    let virt = hhdm_offset + phys.as_u64();
    let ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *ptr }
}

pub fn create_e1000_device(
    hhdm_offset: u64,
    frame_allocator: &mut BumpFrameAllocator,
) -> E1000Device<'_, MyHal> {
    let hal = MyHal {
        hhdm_offset: hhdm_offset as usize,
    };

    let page_table = unsafe { active_page_table(VirtAddr::new(hhdm_offset)) };
    let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(hhdm_offset)) };
    let e1000_virt = map_e1000(&mut mapper, frame_allocator, hhdm_offset);

    e1000_driver::e1000::E1000Device::<MyHal>::new(hal, e1000_virt.try_into().unwrap()).unwrap()
}
