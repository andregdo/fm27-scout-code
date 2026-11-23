//! A club's primary sponsor list, stored inside the club record.
//!
//! The prefix is a null-date marker, two dates, a counted collection of
//! fixed-size entries and a fixed suffix. The suffix closes with the primary
//! sponsor count, and that many 25-byte contract rows follow. The list usually
//! follows the monthly finance chain, but it is owned by the club record and
//! exists for clubs without a full finance series. Every list must be
//! structurally owned; later sponsor-shaped bytes are never a fallback.

/// One shape of the fixed sponsor-prefix suffix. Build 27.1.0+2505117 inserts
/// five bytes at suffix offset 22 (a status byte and a game date), so the later
/// fixed bytes, the dates and the primary sponsor count move by five. The
/// suffix is the same closing block the club table parser accepts at 67 and 72
/// bytes, followed by the primary sponsor count byte.
#[derive(Debug, Clone, Copy)]
pub struct SponsorSuffixLayout {
    pub bytes: usize,
    /// `(offset from the end of the counted prefix entries, expected bytes)`.
    pub fixed: &'static [(usize, &'static [u8])],
    /// Offsets from the end of the counted prefix entries that hold a game date
    /// or the null-date marker.
    pub dates: &'static [usize],
}

const SPONSOR_SUFFIX_68_FIXED: &[(usize, &[u8])] = &[
    (11, &[0]),
    (15, &[0]),
    (20, &[0xff, 0]),
    (43, &[0, 0, 0, 0]),
];
const SPONSOR_SUFFIX_73_FIXED: &[(usize, &[u8])] = &[
    (11, &[0]),
    (15, &[0]),
    (20, &[0xff, 0]),
    (48, &[0, 0, 0, 0]),
];

pub const SPONSOR_SUFFIX_68: SponsorSuffixLayout = SponsorSuffixLayout {
    bytes: 68,
    fixed: SPONSOR_SUFFIX_68_FIXED,
    dates: &[30, 34, 38],
};
pub const SPONSOR_SUFFIX_73: SponsorSuffixLayout = SponsorSuffixLayout {
    bytes: 73,
    fixed: SPONSOR_SUFFIX_73_FIXED,
    dates: &[35, 39, 43],
};

#[derive(Debug, Clone, Copy)]
pub struct SponsorLayout;

impl SponsorLayout {
    pub const PREFIX_MARKER: [u8; 4] = [0x01, 0x00, 0x6c, 0x07];
    pub const PREFIX_DATE_OFFSETS: [usize; 2] = [25, 29];
    pub const PREFIX_COUNT_OFFSET: usize = 71;
    pub const PREFIX_ENTRY_BYTES: usize = 19;
    pub const PREFIX_ENTRY_RESERVED_OFFSET: usize = 8;
    pub const PREFIX_ENTRY_RESERVED: [u8; 7] = [0; 7];
    pub const SUFFIXES: [SponsorSuffixLayout; 2] = [SPONSOR_SUFFIX_68, SPONSOR_SUFFIX_73];
    pub const ROW_BYTES: usize = 25;
    pub const ROW_TAG: u8 = 0x02;
    pub const ROW_TYPE_OFFSET: usize = 1;
    pub const ROW_START_OFFSET: usize = 2;
    pub const ROW_END_OFFSET: usize = 6;
    pub const ROW_FLAG_OFFSET: usize = 10;
    pub const ROW_FLAG_MAXIMUM: u8 = 1;
    pub const ROW_TOTAL_OFFSET: usize = 11;
    /// Club reputation when the deal was signed, as on the game's screen.
    pub const ROW_REPUTATION_OFFSET: usize = 15;
    /// Percentage the amount changes by on relegation (or on promotion for a
    /// kit manufacturer). Zero means no change.
    pub const ROW_PERCENT_OFFSET: usize = 19;
    pub const ROW_PERCENT_MAXIMUM: u16 = 100;
    pub const ROW_ANNUAL_OFFSET: usize = 21;
    pub const ROW_VALUE_MAXIMUM: u32 = 2_000_000_000;
    /// FM27 can store a row that ends one day before it starts.
    pub const MINIMUM_ROW_DAYS: i64 = -1;
}
