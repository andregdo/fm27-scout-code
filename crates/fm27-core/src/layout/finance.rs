//! Monthly finance snapshots inside each club record. The row layout is shared
//! with FM26; FM27 changes only the club record framing that surrounds it.

#[derive(Debug, Clone, Copy)]
pub struct FinanceLayout;

impl FinanceLayout {
    pub const ROW_BYTES: usize = 49;
    pub const TAG: u8 = 0x01;
    pub const COUNT_OFFSET: i32 = -4;
    pub const COUNT_RANGE: (u32, u32) = (3, 1_000);
    pub const SHORT_COUNT_RANGE: (u32, u32) = (1, 2);
    pub const BALANCE_RANGE: (i32, i32) = (-400_000_000, 2_000_000_000);
    pub const WEEKLY_MAXIMUM: u32 = 20_000_000;
    pub const BALANCE_OFFSET: usize = 1;
    pub const TRANSFER_ALLOCATED_OFFSET: usize = 5;
    pub const TRANSFER_REMAINING_OFFSET: usize = 9;
    pub const WAGE_BUDGET_OFFSET: usize = 13;
    pub const WAGE_PAYROLL_OFFSET: usize = 17;
    pub const INCOME_EXCLUDING_TRANSFERS_OFFSET: usize = 21;
    pub const NET_TRANSFERS_OFFSET: usize = 25;
    pub const WAGE_BILL_OFFSET: usize = 29;
    pub const NET_OFFSET: usize = 33;
    pub const EXPENDITURE_EXCLUDING_TRANSFERS_OFFSET: usize = 37;
    pub const TOTAL_INCOME_OFFSET: usize = 41;
    pub const TOTAL_EXPENDITURE_OFFSET: usize = 45;
    /// Records shorter than this never hold a finance chain on the saves
    /// measured, so skipping them keeps the per-club search cheap.
    pub const MINIMUM_RECORD_BYTES: usize = 1_500;
    /// Corporate facilities sit this many bytes past the end of the chain.
    pub const FACILITY_OFFSET_AFTER_CHAIN: usize = 50;
    pub const FACILITY_RANGE: (u8, u8) = (1, 20);
}
