//! Player contract chains: the selector header, the squad-number tail, loan
//! blocks and release-clause entries.

#[derive(Debug, Clone, Copy)]
pub struct ContractLayout;

impl ContractLayout {
    pub const TAG: &'static [u8] = &[0x01, 0x00, 0x6c, 0x07];
    pub const CHAIN_WINDOW_START_OFFSET: i32 = -30;
    pub const CHAIN_WINDOW_END_OFFSET: i32 = -30;
    pub const SELECTOR_OFFSET: usize = 5;
    pub const START_OFFSET: i32 = -19;
    /// Squad number in the current contract: 1..=99, `0` or `255` for none.
    pub const SQUAD_NUMBER_OFFSET: i32 = -12;
    pub const TEAM_ID_OFFSET: usize = 9;
    pub const WAGE_OFFSET: usize = 17;
    pub const TAIL_BASE_OFFSET: usize = 66;
    pub const TAIL_STEP_BYTES: usize = 29;
    pub const TAIL_MAX_EVENT_COUNT: usize = 200;
    pub const TAIL_SENTINEL_BYTE_OFFSET: usize = 3;
    pub const TAIL_SENTINEL_BYTE_VALUE: u8 = 3;
    pub const TAIL_SENTINEL_WORD_OFFSET: usize = 4;
    pub const TAIL_SENTINEL_WORD_VALUE: u32 = 4;
    pub const TAIL_END_OFFSET: usize = 28;
    pub const TAIL_PRINTED_START_OFFSET: usize = 32;
    pub const TAIL_SQUAD_STATUS_OFFSET: usize = 36;
    /// Bit 0 is transfer listed, bit 1 is loan listed. Other bits are game flags
    /// (for example 0x40 on players the club will not sell).
    pub const TAIL_LIST_FLAGS_OFFSET: usize = 39;
    /// A status byte that is often `5`, but also on players who are not listed,
    /// including key players. It is not a transfer-listing indicator.
    pub const TAIL_STATUS_CODE_OFFSET: usize = 40;
    pub const TAIL_E24_OFFSET: usize = 24;
    pub const TAIL_EVENT_COUNT_OFFSET: usize = 42;
    pub const TAIL_STRUCT_END: usize = 46;
    pub const LOAN_BLOCK_MAX_EVENT_COUNT: usize = 60;
    pub const CLAUSE_STEP_BYTES: usize = 8;
    pub const CLAUSE_MAX_COUNT: usize = 23;
    pub const CLAUSE_FF_OFFSET: i32 = -16;
    pub const CLAUSE_FF_COUNT: usize = 8;
    pub const CLAUSE_ZERO_OFFSET: i32 = -8;
    pub const CLAUSE_ZERO_COUNT: usize = 3;
    pub const CLAUSE_COUNT_OFFSET: i32 = -5;
    pub const CLAUSE_ENTRIES_OFFSET: i32 = -4;
    pub const CLAUSE_ENTRY_BYTES: usize = 8;
    pub const CLAUSE_COMPETITION_COUNT_BYTES: usize = 1;
    pub const CLAUSE_COMPETITION_ITEM_BYTES: usize = 31;
    pub const CLAUSE_COMPETITION_ITEM_PREFIX: &'static [u8] = &[0x01, 0x00, 0x01, 0x00, 0x6c, 0x07];
    pub const CLAUSE_COMPETITION_MAX_COUNT: usize = 32;
    pub const CLAUSE_AWARD_FLAG_BYTES: usize = 1;
    pub const CLAUSE_AWARD_COUNT_BYTES: usize = 4;
    pub const CLAUSE_AWARD_ITEM_BYTES: usize = 15;
    pub const CLAUSE_AWARD_ITEM_PREFIX: &'static [u8] = &[0x01, 0x00];
    pub const CLAUSE_AWARD_MAX_COUNT: usize = 32;
    pub const CLAUSE_TRAILER_BYTES: usize = 2;
    pub const CLAUSE_TEAM_MARKER_ID_BYTES: usize = 4;
    pub const HEAD_GATE_OFFSET: i32 = -35;
    pub const HEAD_GATE_VALUE: u16 = 5;
    pub const HEAD_TYPE_OFFSET: i32 = -33;
    pub const FALLBACK_START_FROM_RECORD: usize = 110;
    pub const FALLBACK_END_MARGIN: usize = 90;
    pub const FALLBACK_NONZERO_OFFSET: usize = 8;
    pub const FALLBACK_NONZERO_LENGTH: usize = 8;
    pub const FALLBACK_GATE_LENGTH: usize = 16;
}
