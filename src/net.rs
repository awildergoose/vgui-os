use core::{hint::spin_loop, ptr::NonNull, time::Duration};

use alloc::vec;
use ixgbe_driver::{IxgbeDevice, IxgbeHal, IxgbeNetBuf, MemPool, NicDevice};

use crate::serial_println;

struct MyHal;

unsafe impl IxgbeHal for MyHal {
    fn dma_alloc(size: usize) -> (usize, NonNull<u8>) {
        serial_println!("dma_alloc {size}");
        let layout = alloc::alloc::Layout::from_size_align(size, 4096).expect("Invalid layout");
        let ptr = unsafe { alloc::alloc::alloc(layout) };

        let phys_addr = ptr as usize;
        serial_println!("Are we ok?");
        (phys_addr, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(_paddr: usize, vaddr: NonNull<u8>, size: usize) -> i32 {
        let layout = alloc::alloc::Layout::from_size_align(size, 4096).expect("Invalid layout");
        serial_println!("dealloc {vaddr:?}!");
        unsafe { alloc::alloc::dealloc(vaddr.as_ptr(), layout) };
        serial_println!("done dealloc!");
        0 // Success
    }

    unsafe fn mmio_phys_to_virt(paddr: usize, _size: usize) -> NonNull<u8> {
        // Simple identity mapping for testing
        serial_println!("map bs: {paddr}");
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> usize {
        // Simple identity mapping for testing
        serial_println!("map bs2: {vaddr:?}");
        vaddr.as_ptr() as usize
    }

    fn wait_until(duration: Duration) -> Result<(), &'static str> {
        // TODO: RTC implementation
        // serial_println!("wait {duration:?}");
        for _ in 0..duration.as_millis() * 1000 {
            spin_loop();
        }
        Ok(())
    }
}

pub fn init_network() -> ixgbe_driver::IxgbeResult {
    let buffer = vec![0u8; 16384].into_boxed_slice();

    let pool = MemPool::allocate::<MyHal>(1, 16384)?;
    serial_println!("pool init!");
    let mut device =
        IxgbeDevice::<MyHal, 512>::init(buffer.as_ptr().addr(), buffer.len(), 1, 1, &pool)?;

    serial_println!("device init!");

    // make a packet
    let packet: &[u8] = &[];
    let packet_size = size_of_val(packet);
    let queue_id = 0;
    let num_packets = 1;

    // send a packet
    let tx_buf = IxgbeNetBuf::alloc(&pool, packet_size)?;
    serial_println!("tx buf allocated");
    device.send(queue_id, tx_buf)?;
    device.receive_packets(queue_id, num_packets, |rx_buf| {
        serial_println!("net rx: {:?}", rx_buf.packet());
    })?;

    Ok(())
}
