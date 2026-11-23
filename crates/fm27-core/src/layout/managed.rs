//! Managed-club discovery from the `humans` section and `save_game_summary`.

/// The `humans` section names the save's human managers. The first selector is the
/// manager's person id plus one; the matching person header in `game_db` stores that
/// person id followed by the doubled uid.
#[derive(Debug, Clone, Copy)]
pub struct HumansLayout;

impl HumansLayout {
    pub const HUMAN_COUNT_OFFSET: usize = 8;
    pub const FIRST_SELECTOR_OFFSET: usize = 10;
    pub const PERSON_UID_OFFSET: usize = 4;
    pub const PERSON_UID_COPY_OFFSET: usize = 8;
}

/// The readable strings in `save_game_summary`, used as the independent check for
/// the managed club. A string is a u32 byte length followed by that many UTF-8 bytes;
/// every code point is at least a space.
#[derive(Debug, Clone, Copy)]
pub struct SummaryStringsLayout;

impl SummaryStringsLayout {
    pub const STRINGS_START_OFFSET: usize = 8;
    pub const MIN_LENGTH: usize = 2;
    pub const MAX_LENGTH: usize = 120;
    pub const LOWEST_CODE_POINT: char = '\u{20}';
}
