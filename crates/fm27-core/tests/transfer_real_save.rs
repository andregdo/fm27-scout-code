//! Real-save transfer regression test. Fixtures are intentionally not committed.
//!
//! Set `FM27_TEST_SAVE` to a verified-build career save and run:
//!
//! ```text
//! FM27_TEST_SAVE="/path/to/save.fm" cargo test -p fm27-core --release \
//!   --test transfer_real_save -- --ignored
//! ```

use fm27_core::container::SaveFile;
use fm27_core::editor::PlayerEditor;
use fm27_core::metadata::read_metadata;
use fm27_core::parse_save;

#[test]
#[ignore = "requires a real FM27 save and FM27_TEST_SAVE"]
fn transfers_a_simple_player_and_verifies_the_roster() {
    let path = std::env::var("FM27_TEST_SAVE").expect("set FM27_TEST_SAVE");
    let save = SaveFile::open(&path).expect("open source save");
    let metadata = read_metadata(&save).expect("read source metadata");
    let parsed = parse_save(&path).expect("parse source save");
    let mut editor = PlayerEditor::open(&save, &metadata).expect("open player editor");
    let before_game_db = save.read_section("game_db").expect("read source game_db");
    let game_date = metadata
        .game_date
        .clone()
        .expect("the save has an in-game date");

    let player = parsed
        .players
        .iter()
        .find(|player| {
            let Some(contract) = player.contract.as_ref() else {
                return false;
            };
            let (Some(club_uid), Some(team_id)) = (player.club_uid, player.team_id) else {
                return false;
            };
            player.team_slot == Some(0)
                && player.on_loan != Some(true)
                && contract.team_id == player.team_id
                && contract.chain.len() == 1
                && contract.chain[0].has_terms
                && editor
                    .team_roster(team_id, club_uid)
                    .is_ok_and(|roster| roster.len() >= 2)
                && editor
                    .squad_number(player.uid)
                    .is_ok_and(|number| number.is_some())
        })
        .expect("a player with a simple current contract, a team-mate and a squad number");
    let from_club_uid = player.club_uid.expect("the player has a club");
    let from_team_id = player.team_id.expect("the player has a team");
    let from_gender = player.gender;

    let destination = parsed
        .clubs
        .iter()
        .filter(|club| club.uid != from_club_uid)
        .filter(|club| from_gender.is_none() || club.gender == from_gender)
        .find_map(|club| {
            let team_id = *club.teams.first()?;
            editor
                .team_roster(team_id, club.uid)
                .ok()
                .map(|_| (club.uid, team_id))
        })
        .expect("a destination club with a stored player list");

    let squad_number_before = editor
        .squad_number(player.uid)
        .expect("read the squad number");
    assert!(
        squad_number_before.is_some(),
        "the selected player must wear a squad number"
    );
    let change = editor
        .transfer(player.uid, destination.0)
        .expect("plan and apply the transfer");
    editor.validate().expect("validate edited player");
    assert_eq!(change.to_team_id, destination.1);
    assert_eq!(change.squad_number_before, squad_number_before);

    let directory = tempfile::tempdir().expect("temporary directory");
    let output = directory.path().join("transferred.fm");
    editor
        .write_as(&save, &output, 3, Some("FM27 Scout transfer test"))
        .expect("write transferred save");

    let output_save = SaveFile::open(&output).expect("open transferred save");
    let output_metadata = read_metadata(&output_save).expect("read transferred metadata");
    assert_eq!(
        before_game_db.len(),
        output_save
            .read_section("game_db")
            .expect("read transferred game_db")
            .len(),
        "the transfer must keep game_db the same length"
    );
    let reparsed = parse_save(&output).expect("re-parse transferred save");
    let transferred = reparsed
        .players
        .iter()
        .find(|candidate| candidate.uid == player.uid)
        .expect("the transferred player is still in the save");
    assert_eq!(transferred.club_uid, Some(destination.0));
    assert_eq!(transferred.team_id, Some(destination.1));
    let contract = transferred
        .contract
        .as_ref()
        .expect("the player keeps a contract");
    assert_eq!(contract.team_id, Some(destination.1));
    assert_eq!(contract.start.as_deref(), Some(game_date.as_str()));

    let verify_editor =
        PlayerEditor::open(&output_save, &output_metadata).expect("open transferred editor");
    let old_roster = verify_editor
        .team_roster(from_team_id, from_club_uid)
        .expect("read the old team roster");
    let new_roster = verify_editor
        .team_roster(destination.1, destination.0)
        .expect("read the new team roster");
    let player_index = change.player_index;
    assert!(!old_roster.contains(&player_index));
    assert!(new_roster.contains(&player_index));
    assert_eq!(
        verify_editor
            .squad_number(player.uid)
            .expect("read the new squad number"),
        None
    );
    change
        .verify_save(&output)
        .expect("verify the written transfer through the shared helper");

    println!(
        "transferred {} (uid {}) from club {} team {} to club {} team {}; squad number {:?} cleared",
        player.name.as_deref().unwrap_or("<unnamed>"),
        player.uid,
        from_club_uid,
        from_team_id,
        destination.0,
        destination.1,
        squad_number_before
    );
}
