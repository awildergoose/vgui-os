use chrono::{DateTime, Utc};
use spin::LazyLock;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

use crate::wrapped_sync_cell::WrappedSyncCell;

pub struct CMOS {
    read_port: PortReadOnly<u8>,
    write_port: PortWriteOnly<u8>,
}

fn to_unix_seconds(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<i64, &'static str> {
    Ok(chrono::NaiveDate::from_ymd_opt(year, month, day)
        .ok_or("from_ymd_opt died")?
        .and_hms_opt(hour, minute, second)
        .ok_or("and_hms_opt died")?
        .and_utc()
        .timestamp())
}

fn decode_bcd(value: u8) -> u32 {
    u32::from(value >> 4) * 10 + u32::from(value & 0x0f)
}

impl CMOS {
    #[must_use]
    pub const fn new(write_port: u16, read_port: u16) -> Self {
        Self {
            read_port: PortReadOnly::new(read_port),
            write_port: PortWriteOnly::new(write_port),
        }
    }

    pub fn read_register(&mut self, reg: u8) -> u8 {
        // outb (0x70, (NMI_disable_bit << 7) | (selected CMOS register number));
        unsafe { self.write_port.write((1 << 7) | reg) }
        unsafe { self.read_port.read() }
    }

    pub fn get_time(&mut self) -> DateTime<Utc> {
        // We're assuming:
        // - BCD
        // - 24H
        let second = decode_bcd(self.read_register(0x00));
        let minute = decode_bcd(self.read_register(0x02));
        let hour = decode_bcd(self.read_register(0x04));
        let day = decode_bcd(self.read_register(0x07));
        let month = decode_bcd(self.read_register(0x08));
        let year = 2000 + decode_bcd(self.read_register(0x09)).cast_signed();

        chrono::DateTime::<Utc>::from_timestamp(
            (to_unix_seconds(year, month, day, hour, minute, second).unwrap() as u64)
                .try_into()
                .unwrap(),
            0,
        )
        .unwrap()
    }
}

impl Default for CMOS {
    fn default() -> Self {
        Self::new(0x70, 0x71)
    }
}

pub static CMOS: LazyLock<WrappedSyncCell<CMOS>> =
    LazyLock::new(|| WrappedSyncCell::new(CMOS::default()));
