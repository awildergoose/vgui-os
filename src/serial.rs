use core::hint::spin_loop;

use spin::LazyLock;
use x86_64::instructions::port::Port;

use crate::wrapped_sync_cell::WrappedSyncCell;

pub struct SerialPort {
    data: Port<u8>,
    line_status: Port<u8>,
}

impl SerialPort {
    #[must_use]
    pub unsafe fn init(port: u16) -> Self {
        let mut serial = Self {
            data: Port::new(port),
            line_status: Port::new(port + 5),
        };

        let mut interrupt_enable = Port::<u8>::new(port + 1);
        let mut line_control = Port::<u8>::new(port + 3);
        let mut fifo_control = Port::<u8>::new(port + 2);
        let mut modem_control = Port::<u8>::new(port + 4);

        unsafe {
            // P+1
            interrupt_enable.write(0x00); // disable interrupts
            // P+3
            line_control.write(0x80); // enable dlab (set baud rate divisor)
            // P+0
            serial.data.write(0x03); // set divisor to 38400 baud (lo byte only)
            // P+3
            line_control.write(0x03); // 8 bits, no parity, one stop bit
            // P+2
            fifo_control.write(0xC7); // enable fifo, clear them, with 14-byte threshold
            // P+4
            modem_control.write(0x0B); // IRQs enabled, RTS/DSR set

            // We could test the serial port here but fuck that for now
        }

        serial
    }

    fn is_transmit_empty(&mut self) -> bool {
        unsafe { (self.line_status.read() & 0x20) != 0 }
    }

    pub fn send(&mut self, byte: u8) {
        while !self.is_transmit_empty() {
            spin_loop();
        }

        unsafe {
            self.data.write(byte);
        }
    }
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send(b'\r');
            }

            self.send(byte);
        }

        Ok(())
    }
}

pub static SERIAL1: LazyLock<WrappedSyncCell<SerialPort>> =
    LazyLock::new(|| unsafe { WrappedSyncCell::new(SerialPort::init(0x3F8)) });

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::serial::SERIAL1.with_mut(|s| {
            s.write_fmt(format_args!($($arg)*)).unwrap();
        });
    }};
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
