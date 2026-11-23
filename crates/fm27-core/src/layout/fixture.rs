//! The fixture calendar in the unlisted span after `non_pl_hist_ls`.

/// A fixture-calendar record in the unlisted span after `non_pl_hist_ls`.
///
/// The record's named offset origin is the home team id, so the locator marker
/// and stage id sit before it. FM27 uses the FM26 shape with a two-byte
/// shorter record: 66 bytes, marker `0x1F`, stage-relative fields unchanged.
/// The kick-off date is two packed u16 words at +13; the year is the high word
/// at +15. The season start year at +22 is the field used to keep only the
/// current season for squad labels.
#[derive(Debug, Clone, Copy)]
pub struct FixtureCalendarLayout;

impl FixtureCalendarLayout {
    pub const RECORD_BYTES: usize = 66;
    pub const MARKER_OFFSET: isize = -12;
    pub const MARKER: u8 = 0x1F;
    pub const SENTINEL_OFFSETS: [(usize, u8); 2] = [(4, 0xFF), (11, 0xFF)];
    pub const STAGE_ID_OFFSET: isize = -11;
    pub const HOME_TEAM_ID_OFFSET: usize = 0;
    pub const AWAY_TEAM_ID_OFFSET: usize = 7;
    pub const KICK_OFF_YEAR_OFFSET: usize = 15;
    pub const SEASON_START_YEAR_OFFSET: usize = 22;
    pub const STAGE_ID_LIMIT: u32 = 100_000_000;
    pub const TEAM_ID_LIMIT: u32 = 3_000_000;
    pub const YEARS_BEFORE_CLOCK: i32 = 7;
    pub const YEARS_AFTER_CLOCK: i32 = 8;
    pub const CLUSTER_GAP_BYTES: usize = 1_048_576;
    pub const HEAD_BYTES: usize = 16 * 1024;
    pub const MIN_HEAD_RECORDS: usize = 2;
    pub const MIN_HEAD_FALLBACK: usize = 10;
    pub const LARGE_FRAME_FALLBACK: usize = 4;
    pub const MIN_SEASON_RECORDS: usize = 20;
    pub const MIN_CALENDAR_RECORDS: usize = 100;
}
