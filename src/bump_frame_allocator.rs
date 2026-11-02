use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

pub struct BumpFrameAllocator {
    next: u64,
    end: u64,
}

impl BumpFrameAllocator {
    #[must_use]
    pub const fn new(start: u64, size: u64) -> Self {
        let next = (start + 4095) & !4095;
        let end = (start + size) & !4095;

        Self { next, end }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if self.next + 4096 > self.end {
            return None;
        }

        let frame = PhysFrame::containing_address(PhysAddr::new(self.next));
        self.next += 4096;
        Some(frame)
    }
}
