//! The stadium table in `game_db`. The table is found by its own first rows;
//! rows are 181 bytes unless an inline-name flag adds a counted UTF-8 name.

#[derive(Debug, Clone, Copy)]
pub struct StadiumLayout;

impl StadiumLayout {
    pub const ROW_BYTES: usize = 181;
    pub const ORDINAL_OFFSET: usize = 0;
    pub const UID_OFFSET: usize = 4;
    pub const UID_COPY_OFFSET: usize = 8;
    pub const ZERO_BYTE_OFFSET: usize = 12;
    pub const ALL_SEATER_OFFSET: usize = 13;
    pub const EXPANSION_OFFSET: usize = 21;
    pub const OWNER_OFFSET: usize = 29;
    pub const CAPACITY_OFFSET: usize = 34;
    pub const PITCH_LENGTH_OFFSET: usize = 38;
    pub const PITCH_WIDTH_OFFSET: usize = 40;
    pub const BUILT_OFFSET: usize = 50;
    pub const REBUILT_OFFSET: usize = 54;
    pub const PITCH_MIN_LENGTH_OFFSET: usize = 67;
    pub const PITCH_MIN_WIDTH_OFFSET: usize = 69;
    pub const PITCH_MAX_LENGTH_OFFSET: usize = 71;
    pub const PITCH_MAX_WIDTH_OFFSET: usize = 73;
    pub const FLAGS_OFFSET: usize = 156;
    pub const INLINE_NAME_FLAG: u8 = 0x10;
    pub const NAME_LENGTH_OFFSET: usize = 158;
    pub const NAME_OFFSET: usize = 162;
    pub const NAMED_ROW_EXTRA_BYTES: usize = 4;
    pub const NAME_LENGTH_RANGE: (u32, u32) = (1, 256);
    pub const LOCATOR_ROWS: usize = 12;
    pub const PITCH_LENGTH_RANGE: (u16, u16) = (900, 1_300);
    pub const CLOSING_MARKER_OFFSET: usize = 12;
    pub const CLOSING_MARKER: [u8; 5] = [0, 0, 0, 0, 1];
    pub const CLOSING_UID_OFFSET: usize = 17;
}
