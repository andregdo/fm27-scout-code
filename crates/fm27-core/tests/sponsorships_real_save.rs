//! Real-save regression test for club sponsorship contracts.
//!
//! Set `FM27_TEST_SAVE` to a verified-build career save and run:
//!
//! ```text
//! FM27_TEST_SAVE="/path/to/save.fm" cargo test --release -p fm27-core \
//!   --test sponsorships_real_save -- --ignored
//! ```

use fm27_core::parse_save;
use fm27_core::store::Store;

#[test]
#[ignore = "requires a real FM27 save and FM27_TEST_SAVE"]
fn reads_and_caches_club_sponsorship_contracts() {
    let path = std::env::var("FM27_TEST_SAVE").expect("set FM27_TEST_SAVE");
    let parsed = parse_save(&path).expect("parse save");

    assert!(
        parsed.report.sponsor_lists > 0,
        "expected at least one structurally valid primary sponsor list"
    );
    assert!(
        parsed.report.clubs_with_sponsorships > 0,
        "expected at least one club with sponsor rows"
    );
    assert!(parsed.report.sponsorships > 0);
    assert_eq!(parsed.report.sponsorships, parsed.sponsorships.len());
    // A structurally owned list can still carry a row whose values fail the
    // monetary bounds. Those lists are skipped and reported, never guessed
    // around; the reference save used for development has one such list.

    for row in &parsed.sponsorships {
        assert!(row.start.is_some() && row.end.is_some());
        if let Some(annual) = row.annual_value {
            assert!(annual <= row.total_value);
        }
        assert!(row.years.is_some_and(|years| years >= 1));
        assert!(!row.club_name.is_empty());
    }

    let first = &parsed.sponsorships[0];
    let expected: Vec<_> = parsed
        .sponsorships
        .iter()
        .filter(|row| row.club_uid == first.club_uid)
        .collect();
    let mut store = Store::open_in_memory().expect("open in-memory store");
    let save_id = store.insert_parsed(&parsed).expect("insert parsed save");
    let detail = store
        .get_club(save_id, first.club_uid)
        .expect("club query")
        .expect("club detail");
    assert_eq!(detail.sponsorships.len(), expected.len());
    assert_eq!(
        detail.sponsorships[0].start.as_deref(),
        first.start.as_deref()
    );
    assert_eq!(detail.sponsorships[0].annual_value, first.annual_value);
}
