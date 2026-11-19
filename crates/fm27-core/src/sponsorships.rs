//! Primary sponsor contracts stored inside each club record.
//!
//! The list is owned by the club record: a fixed prefix, a primary sponsor
//! count, and that many 25-byte contract rows. It usually follows the monthly
//! finance chain, but it also exists for clubs without a full finance series,
//! so the reader scans the whole club record rather than only the chain end.
//! Missing, truncated or invalid prefix data yields no list; later
//! sponsor-shaped bytes are never a fallback. The two known suffix shapes cover
//! the FM27 builds through 27.0.1 and build 27.1.0, whose closing block grew by
//! five bytes.

use chrono::{Datelike, Days, NaiveDate};

use crate::bytes::find_bytes;
use crate::layout::{SponsorLayout, SponsorSuffixLayout, decode_date, read_u8, read_u16, read_u32};
use crate::model::Sponsorship;

/// The result of looking for one club's primary sponsor list.
#[derive(Debug, Clone)]
pub enum SponsorListLookup {
    /// No structurally owned list exists in the club record.
    Missing,
    /// The list is explicitly empty.
    Empty,
    /// One structurally valid list, in the order the save stores it.
    Valid(Vec<Sponsorship>),
    /// The prefix established ownership but at least one row failed the checks,
    /// or the two suffix shapes were both structurally valid.
    Rejected,
}

enum Candidate {
    Missing,
    Empty,
    Valid(Vec<Sponsorship>),
    Rejected,
}

/// Find one club's primary sponsor list inside its own record.
///
/// The list is usually after the monthly finance chain but belongs to the club
/// record, so clubs with a shorter finance series still have one. At most one
/// complete list is accepted; two valid lists, or one valid list beside a
/// structurally owned but invalid list, fail closed.
pub fn scan_sponsor_list(
    game_db: &[u8],
    record_start: usize,
    record_end: usize,
    club_uid: u32,
    club_name: &str,
) -> SponsorListLookup {
    let record_end = record_end.min(game_db.len());
    if record_start >= record_end {
        return SponsorListLookup::Missing;
    }
    let marker = SponsorLayout::PREFIX_MARKER;
    let mut found: Option<SponsorListLookup> = None;
    let mut rejected = false;
    let mut at = record_start;
    while at < record_end {
        let Some(relative) = find_bytes(&game_db[at..record_end], &marker) else {
            break;
        };
        let start = at + relative;
        match read_sponsor_list(game_db, start, record_end, club_uid, club_name) {
            SponsorListLookup::Missing => {}
            SponsorListLookup::Rejected => rejected = true,
            candidate => {
                if found.is_some() || rejected {
                    return SponsorListLookup::Rejected;
                }
                found = Some(candidate);
            }
        }
        at = start + 1;
    }
    if rejected {
        return SponsorListLookup::Rejected;
    }
    found.unwrap_or(SponsorListLookup::Missing)
}

/// Read one club's primary sponsor list starting at a known prefix.
pub fn read_sponsor_list(
    game_db: &[u8],
    start: usize,
    record_end: usize,
    club_uid: u32,
    club_name: &str,
) -> SponsorListLookup {
    let mut valid: Option<Vec<Sponsorship>> = None;
    let mut empty = false;
    let mut rejected = false;
    for suffix in SponsorLayout::SUFFIXES {
        match inspect(game_db, start, record_end, suffix, club_uid, club_name) {
            Candidate::Missing => {}
            Candidate::Rejected => rejected = true,
            Candidate::Empty => {
                if valid.is_some() || empty {
                    return SponsorListLookup::Rejected;
                }
                empty = true;
            }
            Candidate::Valid(rows) => {
                if valid.is_some() || empty {
                    return SponsorListLookup::Rejected;
                }
                valid = Some(rows);
            }
        }
    }
    if let Some(rows) = valid {
        SponsorListLookup::Valid(rows)
    } else if empty {
        SponsorListLookup::Empty
    } else if rejected {
        SponsorListLookup::Rejected
    } else {
        SponsorListLookup::Missing
    }
}

fn inspect(
    game_db: &[u8],
    start: usize,
    record_end: usize,
    suffix: SponsorSuffixLayout,
    club_uid: u32,
    club_name: &str,
) -> Candidate {
    if start >= record_end || record_end > game_db.len() {
        return Candidate::Missing;
    }
    let marker = SponsorLayout::PREFIX_MARKER;
    if game_db.get(start..start + marker.len()) != Some(marker.as_slice()) {
        return Candidate::Missing;
    }
    if SponsorLayout::PREFIX_DATE_OFFSETS
        .iter()
        .any(|&offset| !prefix_date_is_valid(game_db, start + offset))
    {
        return Candidate::Missing;
    }
    let Some(count_at) = start.checked_add(SponsorLayout::PREFIX_COUNT_OFFSET) else {
        return Candidate::Missing;
    };
    if count_at >= record_end {
        return Candidate::Missing;
    }
    let Some(entry_count) = game_db.get(count_at).copied() else {
        return Candidate::Missing;
    };
    let Some(entries_end) =
        (count_at + 1).checked_add(entry_count as usize * SponsorLayout::PREFIX_ENTRY_BYTES)
    else {
        return Candidate::Missing;
    };
    let Some(primary_count_at) = entries_end.checked_add(suffix.bytes - 1) else {
        return Candidate::Missing;
    };
    if entries_end > record_end || primary_count_at >= record_end {
        return Candidate::Missing;
    }
    for entry in (count_at + 1..entries_end).step_by(SponsorLayout::PREFIX_ENTRY_BYTES) {
        let reserved_at = entry + SponsorLayout::PREFIX_ENTRY_RESERVED_OFFSET;
        if game_db.get(reserved_at..reserved_at + SponsorLayout::PREFIX_ENTRY_RESERVED.len())
            != Some(SponsorLayout::PREFIX_ENTRY_RESERVED.as_slice())
        {
            return Candidate::Missing;
        }
    }
    for &(offset, expected) in suffix.fixed {
        if game_db.get(entries_end + offset..entries_end + offset + expected.len())
            != Some(expected)
        {
            return Candidate::Missing;
        }
    }
    for &offset in suffix.dates {
        if !prefix_date_is_valid(game_db, entries_end + offset) {
            return Candidate::Missing;
        }
    }
    let first_row = primary_count_at + 1;
    let Some(row_count) = game_db.get(primary_count_at).copied() else {
        return Candidate::Missing;
    };
    let row_count = row_count as usize;
    let Some(run_end) = first_row.checked_add(row_count * SponsorLayout::ROW_BYTES) else {
        return Candidate::Rejected;
    };
    if run_end > record_end {
        return Candidate::Rejected;
    }
    if row_count == 0 {
        return Candidate::Empty;
    }
    let mut rows = Vec::with_capacity(row_count);
    for row in (first_row..run_end).step_by(SponsorLayout::ROW_BYTES) {
        let Some(tag) = read_u8(game_db, row) else {
            return Candidate::Rejected;
        };
        let Some(kind) = read_u8(game_db, row + SponsorLayout::ROW_TYPE_OFFSET) else {
            return Candidate::Rejected;
        };
        let Some(flag) = read_u8(game_db, row + SponsorLayout::ROW_FLAG_OFFSET) else {
            return Candidate::Rejected;
        };
        let Some(total_value) = read_u32(game_db, row + SponsorLayout::ROW_TOTAL_OFFSET) else {
            return Candidate::Rejected;
        };
        let Some(annual_value) = read_u32(game_db, row + SponsorLayout::ROW_ANNUAL_OFFSET) else {
            return Candidate::Rejected;
        };
        let Some(reputation) = read_u16(game_db, row + SponsorLayout::ROW_REPUTATION_OFFSET) else {
            return Candidate::Rejected;
        };
        let Some(percent) = read_u16(game_db, row + SponsorLayout::ROW_PERCENT_OFFSET) else {
            return Candidate::Rejected;
        };
        if tag != SponsorLayout::ROW_TAG
            || flag > SponsorLayout::ROW_FLAG_MAXIMUM
            || total_value > SponsorLayout::ROW_VALUE_MAXIMUM
            || annual_value > total_value
            || percent > SponsorLayout::ROW_PERCENT_MAXIMUM
        {
            return Candidate::Rejected;
        }
        let Some(start_date) = decode_date(game_db, row + SponsorLayout::ROW_START_OFFSET) else {
            return Candidate::Rejected;
        };
        let Some(end_date) = decode_date(game_db, row + SponsorLayout::ROW_END_OFFSET) else {
            return Candidate::Rejected;
        };
        if end_date.signed_duration_since(start_date).num_days() < SponsorLayout::MINIMUM_ROW_DAYS {
            return Candidate::Rejected;
        }
        rows.push(Sponsorship {
            club_uid,
            club_name: club_name.to_owned(),
            kind,
            start: Some(start_date.format("%Y-%m-%d").to_string()),
            end: Some(end_date.format("%Y-%m-%d").to_string()),
            years: years_between(start_date, end_date),
            total_value: total_value as i64,
            annual_value: (annual_value > 0).then_some(annual_value as i64),
            percent,
            percent_rises: kind == KIT_MANUFACTURER,
            reputation_at_signing: (reputation > 0).then_some(reputation),
        });
    }
    Candidate::Valid(rows)
}

/// The kind of a kit manufacturer's deal, whose percentage the game words as a
/// rise on promotion rather than a fall on relegation.
const KIT_MANUFACTURER: u8 = 22;

/// Whole years a deal runs, as the game's sponsorship screen counts them:
/// months from the start to the day after the end, rounded to years, at least
/// one.
pub fn years_between(start: NaiveDate, end: NaiveDate) -> Option<u32> {
    let after = end.checked_add_days(Days::new(1))?;
    let months = (after.year() - start.year()) * 12 + after.month() as i32
        - start.month() as i32
        - i32::from(after.day() < start.day());
    u32::try_from(months)
        .ok()
        .map(|months| ((months + 6) / 12).max(1))
}

/// Derive a saved deal's term from its stored dates.
pub fn sponsorship_years(start: Option<&str>, end: Option<&str>) -> Option<u32> {
    let start = NaiveDate::parse_from_str(start?, "%Y-%m-%d").ok()?;
    let end = NaiveDate::parse_from_str(end?, "%Y-%m-%d").ok()?;
    years_between(start, end)
}

fn prefix_date_is_valid(game_db: &[u8], at: usize) -> bool {
    game_db.get(at..at + SponsorLayout::PREFIX_MARKER.len())
        == Some(SponsorLayout::PREFIX_MARKER.as_slice())
        || decode_date(game_db, at).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{SPONSOR_SUFFIX_68, SPONSOR_SUFFIX_73, encode_date};
    use chrono::NaiveDate;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("date")
    }

    fn write_date(bytes: &mut [u8], at: usize, value: NaiveDate) {
        bytes[at..at + 4].copy_from_slice(&encode_date(value, 0).expect("encode"));
    }

    fn write_list(
        bytes: &mut [u8],
        start: usize,
        suffix: SponsorSuffixLayout,
        entries: usize,
        rows: usize,
    ) -> usize {
        bytes[start..start + 4].copy_from_slice(&SponsorLayout::PREFIX_MARKER);
        for offset in SponsorLayout::PREFIX_DATE_OFFSETS {
            bytes[start + offset..start + offset + 4]
                .copy_from_slice(&SponsorLayout::PREFIX_MARKER);
        }
        bytes[start + SponsorLayout::PREFIX_COUNT_OFFSET] = entries as u8;
        let entries_end = start
            + SponsorLayout::PREFIX_COUNT_OFFSET
            + 1
            + entries * SponsorLayout::PREFIX_ENTRY_BYTES;
        for &(offset, expected) in suffix.fixed {
            bytes[entries_end + offset..entries_end + offset + expected.len()]
                .copy_from_slice(expected);
        }
        for offset in suffix.dates {
            bytes[entries_end + offset..entries_end + offset + 4]
                .copy_from_slice(&SponsorLayout::PREFIX_MARKER);
        }
        bytes[entries_end + suffix.bytes - 1] = rows as u8;
        entries_end + suffix.bytes
    }

    fn write_row(
        bytes: &mut [u8],
        row: usize,
        kind: u8,
        start: NaiveDate,
        end: NaiveDate,
        total: u32,
        annual: u32,
    ) {
        write_row_terms(bytes, row, kind, start, end, total, annual, 0, 0);
    }

    #[allow(clippy::too_many_arguments)]
    fn write_row_terms(
        bytes: &mut [u8],
        row: usize,
        kind: u8,
        start: NaiveDate,
        end: NaiveDate,
        total: u32,
        annual: u32,
        reputation: u16,
        percent: u16,
    ) {
        bytes[row] = SponsorLayout::ROW_TAG;
        bytes[row + SponsorLayout::ROW_TYPE_OFFSET] = kind;
        write_date(bytes, row + SponsorLayout::ROW_START_OFFSET, start);
        write_date(bytes, row + SponsorLayout::ROW_END_OFFSET, end);
        bytes[row + SponsorLayout::ROW_FLAG_OFFSET] = 1;
        bytes[row + SponsorLayout::ROW_TOTAL_OFFSET..row + SponsorLayout::ROW_TOTAL_OFFSET + 4]
            .copy_from_slice(&total.to_le_bytes());
        bytes[row + SponsorLayout::ROW_REPUTATION_OFFSET
            ..row + SponsorLayout::ROW_REPUTATION_OFFSET + 2]
            .copy_from_slice(&reputation.to_le_bytes());
        bytes[row + SponsorLayout::ROW_PERCENT_OFFSET..row + SponsorLayout::ROW_PERCENT_OFFSET + 2]
            .copy_from_slice(&percent.to_le_bytes());
        bytes[row + SponsorLayout::ROW_ANNUAL_OFFSET..row + SponsorLayout::ROW_ANNUAL_OFFSET + 4]
            .copy_from_slice(&annual.to_le_bytes());
    }

    fn parse(bytes: &[u8]) -> SponsorListLookup {
        read_sponsor_list(bytes, 0, bytes.len(), 42, "Test FC")
    }

    #[test]
    fn reads_a_68_byte_suffix_list_in_stored_order() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 2, 2);
        write_row(
            &mut bytes,
            first,
            4,
            date(2025, 7, 1),
            date(2026, 6, 30),
            10_000_000,
            5_000_000,
        );
        write_row(
            &mut bytes,
            first + SponsorLayout::ROW_BYTES,
            1,
            date(2026, 7, 1),
            date(2027, 6, 30),
            3_000_000,
            3_000_000,
        );
        let SponsorListLookup::Valid(rows) = parse(&bytes) else {
            panic!("expected a valid list");
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, 4);
        assert_eq!(rows[0].club_uid, 42);
        assert_eq!(rows[0].club_name, "Test FC");
        assert_eq!(rows[0].start.as_deref(), Some("2025-07-01"));
        assert_eq!(rows[0].end.as_deref(), Some("2026-06-30"));
        assert_eq!(rows[0].years, Some(1));
        assert_eq!(rows[0].total_value, 10_000_000);
        assert_eq!(rows[0].annual_value, Some(5_000_000));
        assert_eq!(rows[0].percent, 0);
        assert_eq!(rows[0].reputation_at_signing, None);
        assert_eq!(rows[1].kind, 1);
    }

    #[test]
    fn reads_a_73_byte_suffix_list() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_73, 0, 1);
        write_row(
            &mut bytes,
            first,
            9,
            date(2028, 6, 19),
            date(2029, 6, 19),
            4_596_122,
            1_532_374,
        );
        let SponsorListLookup::Valid(rows) = parse(&bytes) else {
            panic!("expected a valid list");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, 9);
        assert_eq!(rows[0].annual_value, Some(1_532_374));
    }

    #[test]
    fn decodes_the_stored_terms_and_leaves_a_missing_annual_value_unknown() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 2);
        write_row_terms(
            &mut bytes,
            first,
            22,
            date(2021, 7, 1),
            date(2027, 6, 30),
            859_031,
            143_171,
            6_000,
            15,
        );
        write_row_terms(
            &mut bytes,
            first + SponsorLayout::ROW_BYTES,
            4,
            date(2025, 7, 1),
            date(2026, 6, 30),
            1_000_000,
            0,
            0,
            0,
        );
        let SponsorListLookup::Valid(rows) = parse(&bytes) else {
            panic!("expected a valid list");
        };
        assert_eq!(rows[0].years, Some(6));
        assert_eq!(rows[0].annual_value, Some(143_171));
        assert_eq!(rows[0].percent, 15);
        assert!(rows[0].percent_rises);
        assert_eq!(rows[0].reputation_at_signing, Some(6_000));
        assert_eq!(rows[1].annual_value, None);
        assert_eq!(rows[1].reputation_at_signing, None);
    }

    #[test]
    fn counts_years_as_the_game_screen_does() {
        assert_eq!(years_between(date(2026, 6, 1), date(2027, 6, 30)), Some(1));
        assert_eq!(years_between(date(2026, 12, 6), date(2028, 12, 5)), Some(2));
        assert_eq!(years_between(date(2025, 7, 1), date(2035, 6, 1)), Some(10));
        assert_eq!(years_between(date(2026, 7, 1), date(2026, 7, 31)), Some(1));
        assert_eq!(
            sponsorship_years(Some("2026-07-01"), Some("2029-06-30")),
            Some(3)
        );
        assert_eq!(sponsorship_years(None, Some("2029-06-30")), None);
    }

    #[test]
    fn accepts_an_explicitly_empty_list() {
        let mut bytes = vec![0u8; 2048];
        write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 0);
        assert!(matches!(parse(&bytes), SponsorListLookup::Empty));
        let mut bytes = vec![0u8; 2048];
        write_list(&mut bytes, 0, SPONSOR_SUFFIX_73, 3, 0);
        assert!(matches!(parse(&bytes), SponsorListLookup::Empty));
    }

    #[test]
    fn accepts_a_row_that_ends_one_day_before_it_starts() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            4,
            date(2025, 7, 1),
            date(2025, 6, 30),
            1_000_000,
            1_000_000,
        );
        assert!(matches!(parse(&bytes), SponsorListLookup::Valid(rows) if rows.len() == 1));
    }

    #[test]
    fn rejects_a_row_that_ends_too_early() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            4,
            date(2025, 7, 1),
            date(2025, 6, 29),
            1_000_000,
            1_000_000,
        );
        assert!(matches!(parse(&bytes), SponsorListLookup::Rejected));
    }

    #[test]
    fn rejects_invalid_rows_and_never_falls_back_to_later_data() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            4,
            date(2025, 7, 1),
            date(2026, 6, 30),
            1_000_000,
            2_000_000,
        );
        assert!(matches!(parse(&bytes), SponsorListLookup::Rejected));

        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            4,
            date(2025, 7, 1),
            date(2026, 6, 30),
            1_000_000,
            1_000_000,
        );
        bytes[first] = 0x03;
        assert!(matches!(parse(&bytes), SponsorListLookup::Rejected));

        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            4,
            date(2025, 7, 1),
            date(2026, 6, 30),
            1_000_000,
            1_000_000,
        );
        bytes[0] = 0;
        assert!(matches!(parse(&bytes), SponsorListLookup::Missing));
    }

    #[test]
    fn scans_the_whole_record_and_finds_a_list_away_from_the_start() {
        let mut bytes = vec![0u8; 2048];
        // A bare marker that owns no list must not stop the scan.
        bytes[10..14].copy_from_slice(&SponsorLayout::PREFIX_MARKER);
        let first = write_list(&mut bytes, 100, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            1,
            date(2025, 7, 1),
            date(2028, 6, 30),
            9_000_000,
            3_000_000,
        );
        let SponsorListLookup::Valid(rows) =
            scan_sponsor_list(&bytes, 0, bytes.len(), 42, "Test FC")
        else {
            panic!("expected the list away from the record start");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].annual_value, Some(3_000_000));
    }

    #[test]
    fn rejects_two_valid_lists_in_one_record() {
        let mut bytes = vec![0u8; 2048];
        let first = write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            first,
            1,
            date(2025, 7, 1),
            date(2028, 6, 30),
            1_000_000,
            1_000_000,
        );
        let second = write_list(&mut bytes, 1024, SPONSOR_SUFFIX_68, 0, 1);
        write_row(
            &mut bytes,
            second,
            4,
            date(2025, 7, 1),
            date(2028, 6, 30),
            1_000_000,
            1_000_000,
        );
        assert!(matches!(
            scan_sponsor_list(&bytes, 0, bytes.len(), 42, "Test FC"),
            SponsorListLookup::Rejected
        ));
    }

    #[test]
    fn rejects_a_truncated_prefix_or_row_run() {
        let mut bytes = vec![0u8; 2048];
        write_list(&mut bytes, 0, SPONSOR_SUFFIX_68, 0, 2);
        let truncated = &bytes[..200];
        assert!(matches!(
            read_sponsor_list(truncated, 0, truncated.len(), 42, "Test FC"),
            SponsorListLookup::Missing | SponsorListLookup::Rejected
        ));
    }
}
