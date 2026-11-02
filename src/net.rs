use core::{alloc::Layout, hint::spin_loop, time::Duration};

use e1000_driver::pci::E1000_REGS;
use log::{debug, trace};

use alloc::boxed::Box;

use crate::{cmos::CMOS, net_pci::pci_init, serial_print};

struct MyHal;

impl e1000_driver::e1000::KernelFunc for MyHal {
    const PAGE_SIZE: usize = 1 << 12;

    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize) {
        #[allow(clippy::cast_ptr_alignment)]
        let paddr: Box<[u32]> = {
            let bytes = Self::PAGE_SIZE * pages;

            let ptr = unsafe {
                alloc::alloc::alloc_zeroed(
                    Layout::from_size_align(bytes, Self::PAGE_SIZE).expect("hal layout failed"),
                )
            };

            if ptr.is_null() {
                alloc::alloc::handle_alloc_error(
                    Layout::from_size_align(bytes, Self::PAGE_SIZE).unwrap(),
                );
            }

            unsafe {
                Box::from_raw(core::ptr::slice_from_raw_parts_mut(
                    ptr.cast::<u32>(),
                    bytes,
                ))
            }
        };

        let len = paddr.len();

        let paddr = Box::into_raw(paddr) as *const u32 as usize;
        let vaddr = paddr;
        trace!("alloc paddr: {paddr:#x}, len={len}");

        (vaddr, paddr)
    }

    fn dma_free_coherent(&mut self, _vaddr: usize, _pages: usize) {
        // todo!()
        trace!("todo: free");
    }
}

fn msdelay(t: u64) {
    CMOS.with_mut(|cmos| {
        let start = cmos.get_time();
        let end = start + Duration::from_millis(t);

        while cmos.get_time() < end {
            spin_loop();
        }
    });
}

pub fn init_network() {
    pci_init();
    trace!("PCI OK");

    let mut e1000_device =
        e1000_driver::e1000::E1000Device::<MyHal>::new(MyHal, E1000_REGS.try_into().unwrap())
            .unwrap();

    trace!("device made!");

    // MAC 52:54:00:12:34:56
    let ping_frame: Box<[u8]> = Box::new([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x08, 0x06, 0x00,
        0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x0a, 0x00,
        0x02, 0x0f, //10.0.2.15
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x02, 0x02,
    ]); //ping 10.0.2.2

    msdelay(500);
    e1000_device.e1000_transmit(&ping_frame);
    e1000_device.e1000_transmit(&ping_frame);
    e1000_device.e1000_transmit(&ping_frame);
    e1000_device.e1000_transmit(&ping_frame);

    let mut c = 12;
    loop {
        let rx_buf = e1000_device.e1000_recv();
        if let Some(vecdeque) = rx_buf {
            debug!("e1000 recv num {}", vecdeque.len());
            for v in &vecdeque {
                print_hex_dump(v, v.len());
            }
        }
        c -= 1;
        if c <= 0 {
            break;
        }
        msdelay(100);
    }
}

pub fn print_hex_dump(buf: &[u8], len: usize) {
    //let mut linebuf: [char; 16] = [0 as char; 16];

    use alloc::string::String;
    let mut linebuf = String::with_capacity(32);
    let buf_len = buf.len();

    for (i, item) in buf.iter().enumerate().take(len) {
        if (i % 16) == 0 {
            serial_print!("\t{:?}\nHEX DUMP: ", linebuf);
            //linebuf.fill(0 as char);
            linebuf.clear();
        }

        if i >= buf_len {
            serial_print!(" {:02x}", 0);
        } else {
            serial_print!(" {:02x}", item);
            //linebuf[i%16] = buf[i] as char;
            linebuf.push(*item as char);
        }
    }
    serial_print!("\t{:?}\n", linebuf);
}
