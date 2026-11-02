use core::alloc::Layout;

use log::trace;

pub struct MyHal {
    pub hhdm_offset: usize,
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
