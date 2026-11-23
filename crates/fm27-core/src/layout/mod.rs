//! FM27 layout constants, ported from the verified fmsave reference implementation
//! and checked against real career saves.
//!
//! Submodules group the verified offsets and key tables by the parser area that
//! owns them. Decoders import through this module, so constants can move between
//! submodules without touching call sites.

mod bytes;
mod club;
mod competition;
mod contract;
mod date;
mod finance;
mod fixture;
mod managed;
mod name_pool;
mod player;
mod sponsorship;
mod stadium;
mod staff;
#[cfg(test)]
mod tests;

pub use bytes::{read_i16, read_i32, read_u8, read_u16, read_u32, read_u64};
pub use club::ClubLayout;
pub use competition::{CompetitionIdPairLayout, StageTableLayout};
pub use contract::ContractLayout;
pub use date::{EARLIEST_YEAR, LATEST_YEAR, decode_date, encode_date};
pub use finance::FinanceLayout;
pub use fixture::FixtureCalendarLayout;
pub use managed::{HumansLayout, SummaryStringsLayout};
pub use name_pool::{NAME_POOL_PREFIX, NAME_POOL_SIGNATURE};
pub use player::{
    ATTRIBUTE_KEYS, HIDDEN_ATTRIBUTE_KEYS, PERSONALITY_KEYS, POSITION_KEYS, POSITION_NAMES,
    PersonLayout, PlayerLayout, TRAIT_NAMES, is_ca_exempt_attribute, is_hidden_attribute_key,
    scale_attribute, stored_attribute_index, trait_name,
};
pub use sponsorship::{SPONSOR_SUFFIX_68, SPONSOR_SUFFIX_73, SponsorLayout, SponsorSuffixLayout};
pub use stadium::StadiumLayout;
pub use staff::{
    STAFF_ATTRIBUTE_KEYS, STAFF_ROLE_KEYS, STAFF_ROLE_NAMES, StaffLayout, staff_job_name,
    staff_role_name,
};

pub const GAME: &str = "FM27";
/// Latest FM27 build whose save layout has been verified.
pub const BUILD: &str = "27.1.0+2505117";
/// Builds that share the verified FM27 save layout. Editing is limited to these
/// exact builds so a layout change fails closed instead of guessing offsets.
pub const EDITABLE_BUILDS: &[&str] = &[
    "27.0.0+2486695",
    "27.0.0+2493426",
    "27.0.1+2494687",
    "27.0.1+2503222",
    BUILD,
];
pub const GAME_DB_SCHEMA: u32 = 4000;

pub fn is_editable_build(build: &str) -> bool {
    EDITABLE_BUILDS.contains(&build)
}

pub fn editable_builds() -> String {
    EDITABLE_BUILDS.join(", ")
}

/// Cache/parser revision. This is intentionally independent of the app version:
/// bump the leading version and/or the suffix only when parsing behavior or the
/// cached data shape changes. The suffix invalidates caches written before the
/// complete player-trait and nation-name tables landed, before the current
/// injury windows were read from the stride-14 arrays, before the bundled
/// competition name table was populated, before club leagues were resolved,
/// before the closing person header accepted descending Unique IDs, before
/// staff objects were decoded into their own cached table, before players
/// carried resolved names for their other nationalities, before the managed
/// club was read from the first human manager, before build 27.0.1+2494687
/// was verified and the managed club was read for it, and before build
/// 27.0.1+2503222 was verified against 27.0.1+2494687 and the managed club was
/// read for it, before players carried their registered team slot, and before
/// the fixture calendar was joined to competitions to label reserve and youth
/// squads, and before facility rows accepted the second supporter-share block
/// that build 27.0.1+2503222 stores after the supporter traits, and before build
/// 27.1.0+2505117 was verified and its full-finance closing block was accepted
/// at both the 67-byte and 72-byte sizes, before club sponsorship contracts
/// were decoded from the primary sponsor list after each finance chain, and
/// before that list was scanned across every club record and its stored terms
/// were decoded. The current cache also carries each player's home, current and
/// world reputation for table display and filtering.
pub const PARSER_VERSION: &str = concat!(
    "1.2.0",
    "+nation-names+listing+listing-flags+injury-windows+club-finance+stadiums",
    "+club-facilities+youth-level-edt+competition-names+club-leagues+season-stats+unique-ids",
    "+staff+second-nations+managed-club+build-2494687+build-2503222+squad-slots",
    "+squad-fixtures+club-history-gender+identity-gender+facility-share-tail+staff-job-titles",
    "+build-2505117-finance-tail+player-reputation+club-sponsorships",
    "+sponsor-record-scan+sponsor-terms+stadium-names"
);
pub const MISSING_U32: u32 = 0xFFFF_FFFF;
