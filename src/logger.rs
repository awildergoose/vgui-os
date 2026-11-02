use log::{Metadata, Record};

use crate::{cmos::CMOS, serial_println};

pub struct SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true //metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            serial_println!(
                "[{}] [{}]: {}",
                CMOS.with_mut(super::cmos::CMOS::get_time)
                    .format("%H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub static SERIAL_LOGGER: SerialLogger = SerialLogger;
