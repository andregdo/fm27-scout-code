//! Competition and stage tables in the tail of `game_db`.

/// Fixed-size stage rows in the tail of `game_db`, ported from the verified FM26
/// layout that the FM27 preview build still uses. A row opens with the previous
/// stage id, stores the stage id twice, and carries the competition id at +13.
#[derive(Debug, Clone, Copy)]
pub struct StageTableLayout;

impl StageTableLayout {
    pub const ROW_BYTES: usize = 33;
    pub const PREVIOUS_STAGE_ID_OFFSET: usize = 0;
    pub const STAGE_ID_OFFSET: usize = 4;
    pub const STAGE_ID_COPY_OFFSET: usize = 8;
    pub const ZERO_BYTE_OFFSET: usize = 12;
    pub const COMPETITION_ID_OFFSET: usize = 13;
    pub const STAGE_ID_EXCLUSIVE_RANGE: (u32, u32) = (0, 200_000);
    pub const COMPETITION_ID_LIMIT: u32 = 1_000_000;
    pub const SEARCH_BYTES: usize = 2_000_000;
    pub const CHAIN_ROWS: usize = 200;
    pub const RESYNCHRONISATION_BYTES: usize = 4_096;
}

/// The `FF..FF 01` records that pair a stage-space entity id with the game's
/// editor database id. Every offset is relative to the record start, which sits
/// 30 bytes after the marker.
#[derive(Debug, Clone, Copy)]
pub struct CompetitionIdPairLayout;

impl CompetitionIdPairLayout {
    pub const MARKER: &'static [u8] = &[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x01,
    ];
    pub const RECORD_OFFSET_FROM_MARKER: usize = 30;
    pub const ENTITY_ID_OFFSET: usize = 0;
    pub const DATABASE_ID_OFFSET: usize = 4;
    pub const DATABASE_ID_COPY_OFFSET: usize = 8;
    pub const ENTITY_ID_RANGE: (u32, u32) = (1, 199_999);
    pub const DATABASE_ID_RANGE: (u32, u32) = (1, i32::MAX as u32);
    pub const CONSTANT_BYTES: [(isize, u8); 15] = [
        (-8, 7),
        (-6, 0),
        (-4, 7),
        (-1, 255),
        (12, 0),
        (15, 1),
        (22, 0),
        (23, 0),
        (24, 0),
        (25, 0),
        (26, 1),
        (32, 2),
        (40, 3),
        (55, 6),
        (60, 7),
    ];
}
