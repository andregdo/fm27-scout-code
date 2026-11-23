use chrono::{Datelike, NaiveDate};

use super::*;

#[test]
fn editing_accepts_only_verified_builds() {
    assert!(is_editable_build(BUILD));
    assert!(is_editable_build("27.0.0+2486695"));
    assert!(is_editable_build("27.0.0+2493426"));
    assert!(is_editable_build("27.0.1+2494687"));
    assert!(is_editable_build("27.0.1+2503222"));
    assert!(is_editable_build("27.1.0+2505117"));
    assert!(!is_editable_build("27.0.0+9999999"));
    assert!(!is_editable_build("27.0.1+2493426"));
    assert!(!is_editable_build("27.0.1+2494688"));
    assert!(!is_editable_build("27.0.1+2503223"));
    assert!(!is_editable_build("27.1.0+2505118"));
    assert_eq!(
        editable_builds(),
        "27.0.0+2486695, 27.0.0+2493426, 27.0.1+2494687, 27.0.1+2503222, 27.1.0+2505117"
    );
}

#[test]
fn encodes_and_decodes_game_dates() {
    for date in [
        NaiveDate::from_ymd_opt(2027, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(2028, 2, 29).unwrap(),
        NaiveDate::from_ymd_opt(2030, 12, 31).unwrap(),
    ] {
        let bytes = encode_date(date, 0).expect("date in range");
        assert_eq!(decode_date(&bytes, 0), Some(date));
    }
}

#[test]
fn encoding_preserves_high_date_bits() {
    let date = NaiveDate::from_ymd_opt(2029, 6, 30).unwrap();
    let bytes = encode_date(date, 0xFE00).expect("date in range");
    let packed = u16::from_le_bytes([bytes[0], bytes[1]]);
    assert_eq!(packed & !0x1FF, 0xFE00);
    assert_eq!(packed & 0x1FF, date.ordinal() as u16);
    assert_eq!(decode_date(&bytes, 0), Some(date));
}

#[test]
fn rejects_dates_outside_the_game_range() {
    assert!(encode_date(NaiveDate::from_ymd_opt(1900, 1, 2).unwrap(), 0).is_none());
    assert!(encode_date(NaiveDate::from_ymd_opt(2201, 1, 1).unwrap(), 0).is_none());
}

#[test]
fn date_round_trips_day_of_year() {
    let mut bytes = [0u8; 4];
    let day = NaiveDate::from_ymd_opt(2026, 7, 13).unwrap();
    let doy = day.ordinal() as u16;
    let packed = doy;
    bytes[..2].copy_from_slice(&packed.to_le_bytes());
    bytes[2..].copy_from_slice(&2026u16.to_le_bytes());
    assert_eq!(decode_date(&bytes, 0), Some(day));
}

#[test]
fn date_rejects_bad_years_and_days() {
    let mut bytes = [0u8; 4];
    bytes[..2].copy_from_slice(&1u16.to_le_bytes());
    bytes[2..].copy_from_slice(&1899u16.to_le_bytes());
    assert_eq!(decode_date(&bytes, 0), None);
    bytes[2..].copy_from_slice(&2026u16.to_le_bytes());
    bytes[..2].copy_from_slice(&400u16.to_le_bytes());
    assert_eq!(decode_date(&bytes, 0), None);
}

#[test]
fn staff_job_names_cover_the_board_and_medical_codes() {
    assert_eq!(staff_job_name(24), Some("General Manager"));
    assert_eq!(staff_job_name(44), Some("Chief Scout"));
    assert_eq!(staff_job_name(46), Some("Doctor"));
    assert_eq!(staff_job_name(66), Some("Owner"));
    assert_eq!(staff_job_name(99), None);
}

#[test]
fn attributes_use_the_verified_scale() {
    assert_eq!(scale_attribute(0), 1);
    assert_eq!(scale_attribute(1), 1);
    assert_eq!(scale_attribute(50), 10);
    assert_eq!(scale_attribute(75), 15);
    assert_eq!(scale_attribute(100), 20);
}

#[test]
fn attribute_indices_skip_the_two_foot_bytes() {
    assert_eq!(stored_attribute_index("crossing"), Some(0));
    assert_eq!(stored_attribute_index("technique"), Some(23));
    assert_eq!(stored_attribute_index("flair"), Some(26));
    assert_eq!(stored_attribute_index("consistency"), Some(44));
    assert_eq!(stored_attribute_index("important_matches"), Some(47));
    assert_eq!(stored_attribute_index("concentration"), Some(53));
    assert_eq!(stored_attribute_index("not_an_attribute"), None);
}

#[test]
fn player_traits_cover_every_used_bit() {
    for bit in 0..64u8 {
        if bit == 53 {
            assert_eq!(trait_name(bit), None);
            continue;
        }
        assert!(trait_name(bit).is_some(), "trait bit {bit} has no name");
    }
    assert_eq!(trait_name(10), Some("Places Shots"));
    assert_eq!(trait_name(51), Some("Runs With Ball Often"));
}

#[test]
fn name_pool_signature_encodes_the_verified_words() {
    let words: Vec<u32> = NAME_POOL_SIGNATURE
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("four-byte word")))
        .collect();
    assert_eq!(words, [46421, 0, 1024, 256, 2048, 0]);
    assert_eq!(&NAME_POOL_PREFIX[..], &NAME_POOL_SIGNATURE[..5]);
}
