//! Real-save regression coverage for stadium names.
//!
//! Set `FM27_TEST_SAVE` to a verified-build career save and run:
//!
//! ```text
//! FM27_TEST_SAVE="/path/to/save.fm" cargo test --release -- --ignored
//! ```

use fm27_core::parse_save;

#[test]
#[ignore = "requires a real FM27 save and FM27_TEST_SAVE"]
fn resolves_stadium_names_from_the_table_and_save_references() {
    let path = std::env::var("FM27_TEST_SAVE").expect("set FM27_TEST_SAVE");
    let parsed = parse_save(&path).expect("parse save");
    assert!(parsed.stadiums.len() > 20_000, "stadium count");

    let named = parsed
        .stadiums
        .iter()
        .filter(|stadium| stadium.name.is_some())
        .count();
    assert!(named > 10_000, "only {named} stadiums carry a name");

    for (uid, expected) in [
        (680, "Old Trafford"),
        (676, "Anfield"),
        (5_114_336, "Emirates Stadium"),
        (152_588, "San Siro"),
        (43_156_148, "Centro Sportivo Milanello"),
        (43_156_152, "Centro Sportivo P. Vismara"),
        (8_700_532, "Allianz Arena"),
    ] {
        let stadium = parsed
            .stadiums
            .iter()
            .find(|stadium| stadium.uid == uid)
            .unwrap_or_else(|| panic!("stadium uid {uid} not found"));
        assert_eq!(stadium.name.as_deref(), Some(expected), "stadium uid {uid}");
    }
}
