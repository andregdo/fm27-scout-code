//! Real-save release regression test. Fixtures are intentionally not committed.
//!
//! Set `FM27_TEST_SAVE` to a verified-build career save and run:
//!
//! ```text
//! FM27_TEST_SAVE="/path/to/save.fm" cargo test -p fm27-core --release \
//!   --test release_real_save -- --ignored
//! ```

use fm27_core::container::SaveFile;
use fm27_core::editor::PlayerEditor;
use fm27_core::metadata::read_metadata;
use fm27_core::parse_save;

#[test]
#[ignore = "requires a real FM27 save and FM27_TEST_SAVE"]
fn releases_a_simple_player_and_verifies_the_free_agent_state() {
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
                && contract.chain.iter().all(|entry| {
                    entry
                        .start
                        .as_deref()
                        .is_none_or(|start| start <= game_date.as_str())
                })
                && editor
                    .team_roster(team_id, club_uid)
                    .is_ok_and(|roster| roster.len() >= 2)
        })
        .expect("a senior player with a current contract and a team-mate");
    let from_club_uid = player.club_uid.expect("the player has a club");
    let from_team_id = player.team_id.expect("the player has a team");
    let contract_before = player.contract.clone().expect("the player has a contract");

    let change = editor
        .release(player.uid)
        .expect("plan and apply the release");
    editor.validate().expect("validate edited player");
    assert_eq!(change.from_club_uid, from_club_uid);
    assert_eq!(change.from_team_id, from_team_id);
    assert_eq!(change.players_after, change.players_before - 1);
    assert!(!change.lists.is_empty());
    assert!(
        change.lists.iter().any(|list| list.team_id == from_team_id),
        "the registered team's list must be rewritten"
    );

    let directory = tempfile::tempdir().expect("temporary directory");
    let output = directory.path().join("released.fm");
    editor
        .write_as(&save, &output, 3, Some("FM27 Scout release test"))
        .expect("write released save");

    let output_save = SaveFile::open(&output).expect("open released save");
    let output_game_db = output_save
        .read_section("game_db")
        .expect("read released game_db");
    assert_eq!(
        output_game_db.len(),
        before_game_db.len() - 4 * change.lists.len(),
        "the release must remove four bytes from each rewritten list"
    );

    let reparsed = parse_save(&output).expect("re-parse released save");
    let released = reparsed
        .players
        .iter()
        .find(|candidate| candidate.uid == player.uid)
        .expect("the released player is still in the save");
    assert_eq!(released.club_uid, None);
    assert_eq!(released.team_id, None);
    let contract = released
        .contract
        .as_ref()
        .expect("the release keeps the contract record");
    assert_eq!(contract.club_uid, contract_before.club_uid);
    assert_eq!(contract.team_id, contract_before.team_id);
    assert_eq!(contract.wage, contract_before.wage);
    assert_eq!(contract.start, contract_before.start);
    assert_eq!(contract.end, contract_before.end);
    assert_eq!(contract.squad_status, contract_before.squad_status);
    assert_eq!(contract.transfer_listed, contract_before.transfer_listed);
    assert_eq!(contract.loan_listed, contract_before.loan_listed);

    let verify_editor = PlayerEditor::open(
        &output_save,
        &read_metadata(&output_save).expect("released metadata"),
    )
    .expect("open released editor");
    for list in &change.lists {
        let roster = verify_editor
            .team_roster(list.team_id, list.club_uid)
            .expect("read the released team roster");
        assert!(
            !roster.contains(&change.player_index),
            "team {} still lists the released player",
            list.team_id
        );
    }
    change
        .verify_save(&output)
        .expect("verify the written release through the shared helper");

    println!(
        "released {} (uid {}) from club {} team {}; lists {:?}; squad {} -> {}",
        player.name.as_deref().unwrap_or("<unnamed>"),
        player.uid,
        from_club_uid,
        from_team_id,
        change
            .lists
            .iter()
            .map(|list| (list.team_id, list.players_before, list.players_after))
            .collect::<Vec<_>>(),
        change.players_before,
        change.players_after
    );
}

#[test]
#[ignore = "requires a real FM27 save and FM27_TEST_SAVE"]
fn refuses_to_release_a_free_agent() {
    let path = std::env::var("FM27_TEST_SAVE").expect("set FM27_TEST_SAVE");
    let save = SaveFile::open(&path).expect("open source save");
    let metadata = read_metadata(&save).expect("read source metadata");
    let parsed = parse_save(&path).expect("parse source save");
    let mut editor = PlayerEditor::open(&save, &metadata).expect("open player editor");
    let free_agent = parsed
        .players
        .iter()
        .find(|player| player.team_id.is_none())
        .expect("the save has a free agent");
    let error = editor
        .release(free_agent.uid)
        .expect_err("a free agent must be refused");
    assert!(error.to_string().contains("free agent"), "{error}");
}
