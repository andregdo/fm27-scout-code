//! The game's packed dates: the low 9 bits of the first word hold the day of
//! the year, and the second word holds the year.

use chrono::{Datelike, NaiveDate};

use super::bytes::read_u16;

pub const EARLIEST_YEAR: i32 = 1901;
pub const LATEST_YEAR: i32 = 2200;

/// Decode a stored game date. `offset` points at the low 16-bit day/slot word.
pub fn decode_date(bytes: &[u8], offset: usize) -> Option<NaiveDate> {
    let packed = read_u16(bytes, offset)?;
    let year = i32::from(read_u16(bytes, offset + 2)?);
    let day_of_year = u32::from(packed & 0x1FF);
    if !(EARLIEST_YEAR..=LATEST_YEAR).contains(&year) {
        return None;
    }
    NaiveDate::from_yo_opt(year, day_of_year)
}

/// Encode a game date, preserving any flags stored in the high bits of the
/// existing low word. The low 9 bits are the day of the year (1..=366); the
/// next word is the year.
pub fn encode_date(date: NaiveDate, existing_packed: u16) -> Option<[u8; 4]> {
    let year = u16::try_from(date.year()).ok()?;
    if !(EARLIEST_YEAR..=LATEST_YEAR).contains(&i32::from(year)) {
        return None;
    }
    let packed = (existing_packed & !0x1FF) | date.ordinal() as u16;
    let mut bytes = [0u8; 4];
    bytes[..2].copy_from_slice(&packed.to_le_bytes());
    bytes[2..].copy_from_slice(&year.to_le_bytes());
    Some(bytes)
}
