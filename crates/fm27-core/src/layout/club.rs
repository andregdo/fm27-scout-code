//! Club record layout in `game_db`.

#[derive(Debug, Clone, Copy)]
pub struct ClubLayout;

impl ClubLayout {
    pub const ANCHOR: &'static [u8] = &[0xFF, 0xFF, 0xFF, 0xFF];
    pub const ANCHOR_OFFSET: usize = 17;
    pub const CLUB_INDEX_OFFSET: usize = 0;
    pub const UID_OFFSET: usize = 4;
    pub const UID_COPY_OFFSET: usize = 8;
    pub const ZERO_BYTE_OFFSET: usize = 12;
    pub const NATION_OFFSET: usize = 13;
    pub const FA_NATION_OFFSET: usize = 21;
    pub const NATION_COPY_OFFSET: usize = 25;
    pub const CITY_OFFSET: usize = 29;
    pub const LONG_NAME_OFFSET: usize = 39;
    pub const NATION_RANGE: (u32, u32) = (0, 999);
    pub const MAX_NAME_BYTES: usize = 255;
    pub const STOP_GAP_BYTES: usize = 64 * 1024;
    pub const NULL_DATE_TRIPLE: [u8; 12] = [1, 0, 0x6c, 0x07, 1, 0, 0x6c, 0x07, 1, 0, 0x6c, 0x07];
    pub const TEAM_COUNTS_OFFSET: usize = 37;
    pub const FIRST_ENTRY_BYTES: usize = 25;
    pub const FIRST_ENTRY_LEAD: u8 = 0x02;
    pub const SECOND_ENTRY_BYTES: usize = 12;
    pub const SECOND_ENTRY_LEAD: u8 = 0x01;
    pub const FILLER_BYTES: usize = 9;
    pub const TEAM_COUNT_RANGE: (u8, u8) = (1, 8);
    pub const TEAM_ID_RANGE: (u32, u32) = (1, 3_000_000);
    pub const AFFILIATE_COUNT_RANGE: (u8, u8) = (0, 8);
    pub const FLOAT_ANCHOR: [u8; 4] = 1.0f32.to_le_bytes();
    pub const FLOAT_ANCHOR_OFFSET: usize = 33;
    pub const MINIMUM_FLOAT_ANCHOR_RECORD_OFFSET: usize = 40;
    pub const GENDER_OFFSET_FROM_TEAM_COUNT: i32 = -1;
}
