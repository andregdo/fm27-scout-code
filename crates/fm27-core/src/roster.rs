//! Per-team player lists in `game_db`.
//!
//! FM27 stores each team's players as a counted array of player indexes. The
//! layout below was confirmed on `27.0.0+2486695` and `27.0.1+2503222` saves by
//! comparing career saves before and after the game completed a transfer, and by
//! checking the decoded lists against the players' `team_id` values:
//!
//! ```text
//! +0                     u32 team id - 1
//! +4                     u32 club uid - 1
//! +8                     u32 club uid - 1 (copy of +4)
//! +46                    u16 player count
//! +48                    u32 player index + 1, `count` times
//! +48 + 4 * count        first player reference (captain) or FFFFFFFF
//! +52 + 4 * count        second player reference (vice-captain) or FFFFFFFF
//! +48 + 4 * count + 38   next record
//! ```
//!
//! A record is `86 + 4 * count` bytes long and records are laid out one after
//! another. The rest of the 38-byte trailer is not mapped; it is copied unchanged.
//! The first two trailer words are player indexes plus one, always among the
//! record's own players when set, and are treated as the captain and vice-captain.
//! That identification is not proven against the game UI yet, so the transfer
//! editor only clears them, as FMLibreEditor does with a captaincy.
//!
//! This module never guesses: a candidate record must carry the doubled club uid,
//! a sane count, player indexes that resolve to known player records, and trailer
//! references that resolve to players in the same list.

use std::collections::HashSet;

use memchr::memmem;

use crate::error::{Error, Result};
use crate::layout::read_u32;

/// Bytes before the player-count word in a team record.
pub const TEAM_HEADER_BYTES: usize = 46;
/// Offset of the `u16` player count from the record start.
pub const PLAYER_COUNT_OFFSET: usize = 46;
/// Offset of the first player index from the record start.
pub const PLAYERS_OFFSET: usize = 48;
/// Bytes after the player array before the next record.
pub const TRAILER_BYTES: usize = 38;
/// Bytes in a record with no players (`46 + 2 + 38`).
pub const EMPTY_RECORD_BYTES: usize = TEAM_HEADER_BYTES + 2 + TRAILER_BYTES;
/// The largest player count accepted from a record. No FM squad comes near it.
pub const MAX_PLAYERS: usize = 500;
/// The largest team id accepted while validating neighbouring records.
const TEAM_ID_LIMIT: u32 = 3_000_000;

/// One team's decoded player list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamListRecord {
    /// Offset of the record in `game_db`.
    pub start: usize,
    pub team_id: u32,
    pub club_uid: u32,
    /// Player indexes plus one, in stored order.
    pub players: Vec<u32>,
    /// First trailer player reference, if the record carries one.
    pub captain: Option<u32>,
    /// Second trailer player reference, if the record carries one.
    pub vice_captain: Option<u32>,
}

impl TeamListRecord {
    /// Bytes this record occupies in `game_db`.
    pub fn len(&self) -> usize {
        EMPTY_RECORD_BYTES + 4 * self.players.len()
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

/// What a roster edit changed, for reporting and verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RosterTransfer {
    /// Player index plus one.
    pub player_index: u32,
    pub from_players_before: usize,
    pub from_players_after: usize,
    pub to_players_before: usize,
    pub to_players_after: usize,
    /// Whether the old team listed the player as captain or vice-captain.
    pub captaincy_cleared: bool,
}

/// Find the player list for `team_id` in `game_db`.
///
/// `expected_club_uid` is the club the caller believes fields the team. Senior
/// teams store their club uid minus one in the record; affiliate or reserve teams
/// may be stored under a separate club object, so callers should pass `None` when
/// they do not know the stored club. `known_pindexes` must contain every player
/// record's `pindex`.
pub fn find_team_list(
    game_db: &[u8],
    team_id: u32,
    expected_club_uid: Option<u32>,
    known_pindexes: &HashSet<u32>,
) -> Result<TeamListRecord> {
    if team_id == 0 {
        return Err(Error::Unsupported(
            "team id 0 does not name a team player list".to_owned(),
        ));
    }
    let mut needle = team_id.wrapping_sub(1).to_le_bytes().to_vec();
    if let Some(expected) = expected_club_uid {
        let club_raw = expected.wrapping_sub(1).to_le_bytes();
        needle.extend_from_slice(&club_raw);
        needle.extend_from_slice(&club_raw);
    }
    let mut matches = Vec::new();
    for at in memmem::find_iter(game_db, &needle) {
        let Some(record) = decode_record_at(game_db, at, known_pindexes) else {
            continue;
        };
        if record.team_id != team_id {
            continue;
        }
        if let Some(expected) = expected_club_uid {
            if record.club_uid != expected {
                continue;
            }
        }
        if !plausible_chain(game_db, &record, known_pindexes) {
            continue;
        }
        matches.push(record);
    }
    match matches.len() {
        0 => Err(Error::Unsupported(format!(
            "the save stores no player list for team {team_id}"
        ))),
        1 => Ok(matches.pop().expect("one match")),
        count => Err(Error::parse(format!(
            "team {team_id} has {count} candidate player lists in game_db"
        ))),
    }
}

/// Decode a team record starting at `at`, or return `None` when the bytes do not
/// carry the observed team-record shape.
fn decode_record_at(
    game_db: &[u8],
    at: usize,
    known_pindexes: &HashSet<u32>,
) -> Option<TeamListRecord> {
    let team_raw = read_u32(game_db, at)?;
    let club_raw = read_u32(game_db, at + 4)?;
    let club_copy = read_u32(game_db, at + 8)?;
    if club_raw != club_copy {
        return None;
    }
    let team_id = team_raw.checked_add(1)?;
    let club_uid = club_raw.checked_add(1)?;
    if !(1..TEAM_ID_LIMIT).contains(&team_id) {
        return None;
    }
    let count = usize::from(u16::from_le_bytes(
        game_db
            .get(at + PLAYER_COUNT_OFFSET..at + PLAYER_COUNT_OFFSET + 2)?
            .try_into()
            .ok()?,
    ));
    if count > MAX_PLAYERS {
        return None;
    }
    let players_start = at + PLAYERS_OFFSET;
    let players_end = players_start.checked_add(count.checked_mul(4)?)?;
    let trailer_end = players_end.checked_add(TRAILER_BYTES)?;
    if trailer_end > game_db.len() {
        return None;
    }
    let mut players = Vec::with_capacity(count);
    for index in 0..count {
        let value = read_u32(game_db, players_start + index * 4)?;
        if value == 0 || value == u32::MAX {
            players.push(value);
            continue;
        }
        if !known_pindexes.contains(&(value - 1)) {
            return None;
        }
        players.push(value);
    }
    let first = read_u32(game_db, players_end)?;
    let second = read_u32(game_db, players_end + 4)?;
    let reference = |value: u32| -> Option<Option<u32>> {
        match value {
            0 | u32::MAX => Some(None),
            value if players.contains(&value) => Some(Some(value)),
            _ => None,
        }
    };
    let captain = reference(first)?;
    let vice_captain = reference(second)?;
    Some(TeamListRecord {
        start: at,
        team_id,
        club_uid,
        players,
        captain,
        vice_captain,
    })
}

/// Whether the candidate record is next to another well-shaped team record.
///
/// This rejects a random byte pattern that happens to carry a team id and club id.
/// A record with players is already strongly constrained by resolving every player
/// index; an empty record relies on its neighbours.
fn plausible_chain(game_db: &[u8], record: &TeamListRecord, known_pindexes: &HashSet<u32>) -> bool {
    if !record.players.is_empty() {
        return true;
    }
    let next = record.start + record.len();
    if next + 10 <= game_db.len() {
        let next_team = read_u32(game_db, next)
            .and_then(|value| value.checked_add(1))
            .unwrap_or(0);
        let next_club = read_u32(game_db, next + 4);
        let next_club_copy = read_u32(game_db, next + 8);
        if next_club.is_some()
            && next_club == next_club_copy
            && next_team > record.team_id
            && next_team < TEAM_ID_LIMIT
        {
            return true;
        }
    }
    let lowest = record
        .start
        .saturating_sub(EMPTY_RECORD_BYTES + 4 * MAX_PLAYERS);
    for candidate in lowest..record.start {
        let Some(previous) = decode_record_at(game_db, candidate, known_pindexes) else {
            continue;
        };
        if previous.start + previous.len() == record.start {
            return true;
        }
    }
    false
}

/// Move `player_index` (a `pindex + 1`) from `from` to `to`, rewriting both team
/// records in `game_db`.
///
/// The caller must have found both records in `game_db`. The edit removes the
/// player from the old list, clears a captain or vice-captain reference to him,
/// and appends him to the new list. The game's own transfer appended the player,
/// so the stored order is preserved rather than sorted. The total `game_db` length
/// is unchanged: one record loses four bytes and the other gains four.
pub fn validate_transfer(
    game_db: &[u8],
    player_index: u32,
    from: &TeamListRecord,
    to: &TeamListRecord,
) -> Result<()> {
    if from.start == to.start {
        return Err(Error::Unsupported(
            "the player's current and destination teams are the same record".to_owned(),
        ));
    }
    if player_index == 0 || player_index == u32::MAX {
        return Err(Error::parse("the player index is a missing-player marker"));
    }
    let from_hits = from
        .players
        .iter()
        .filter(|&&value| value == player_index)
        .count();
    if from_hits != 1 {
        return Err(Error::Unsupported(format!(
            "the old team's player list holds the player {from_hits} times; a transfer needs exactly once"
        )));
    }
    if to.players.contains(&player_index) {
        return Err(Error::Unsupported(
            "the destination team's player list already holds the player".to_owned(),
        ));
    }
    if from.start + from.len() > game_db.len() || to.start + to.len() > game_db.len() {
        return Err(Error::parse("a team record runs past the end of game_db"));
    }
    let (first, second) = if from.start < to.start {
        (from, to)
    } else {
        (to, from)
    };
    if first.start + first.len() > second.start {
        return Err(Error::parse("the two team records overlap in game_db"));
    }
    Ok(())
}

pub fn apply_transfer(
    game_db: &mut Vec<u8>,
    player_index: u32,
    from: &TeamListRecord,
    to: &TeamListRecord,
) -> Result<RosterTransfer> {
    validate_transfer(game_db, player_index, from, to)?;

    let mut from_players = from.players.clone();
    from_players.retain(|&value| value != player_index);
    let mut to_players = to.players.clone();
    to_players.push(player_index);

    let captaincy_cleared =
        from.captain == Some(player_index) || from.vice_captain == Some(player_index);
    let from_new = encode_record(game_db, from, &from_players, true, player_index);
    let to_new = encode_record(game_db, to, &to_players, false, player_index);

    let (first, first_new, second, second_new) = if from.start < to.start {
        (from, from_new, to, to_new)
    } else {
        (to, to_new, from, from_new)
    };
    let first_end = first.start + first.len();
    let second_end = second.start + second.len();
    if first_end > second.start {
        return Err(Error::parse("the two team records overlap in game_db"));
    }
    let mut out = Vec::with_capacity(game_db.len());
    out.extend_from_slice(&game_db[..first.start]);
    out.extend_from_slice(&first_new);
    out.extend_from_slice(&game_db[first_end..second.start]);
    out.extend_from_slice(&second_new);
    out.extend_from_slice(&game_db[second_end..]);
    if out.len() != game_db.len() {
        return Err(Error::parse(format!(
            "the roster edit changed game_db length from {} to {}",
            game_db.len(),
            out.len()
        )));
    }
    *game_db = out;
    Ok(RosterTransfer {
        player_index,
        from_players_before: from.players.len(),
        from_players_after: from_players.len(),
        to_players_before: to.players.len(),
        to_players_after: to_players.len(),
        captaincy_cleared,
    })
}

/// Encode one team record with `players`, copying the record's header and trailer.
fn encode_record(
    game_db: &[u8],
    record: &TeamListRecord,
    players: &[u32],
    clear_captaincy: bool,
    player_index: u32,
) -> Vec<u8> {
    let trailer_start = record.start + PLAYERS_OFFSET + record.players.len() * 4;
    let mut out = Vec::with_capacity(EMPTY_RECORD_BYTES + players.len() * 4);
    out.extend_from_slice(&game_db[record.start..record.start + TEAM_HEADER_BYTES]);
    out.extend_from_slice(&(players.len() as u16).to_le_bytes());
    for player in players {
        out.extend_from_slice(&player.to_le_bytes());
    }
    let trailer = &game_db[trailer_start..trailer_start + TRAILER_BYTES];
    out.extend_from_slice(trailer);
    if clear_captaincy {
        let first = TEAM_HEADER_BYTES + 2 + players.len() * 4;
        for offset in [first, first + 4] {
            let value = u32::from_le_bytes(out[offset..offset + 4].try_into().expect("four bytes"));
            if value == player_index {
                out[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(
        team_id: u32,
        club_uid: u32,
        players: &[u32],
        captain: Option<u32>,
        vice: Option<u32>,
    ) -> Vec<u8> {
        let mut out = vec![0u8; EMPTY_RECORD_BYTES + players.len() * 4];
        out[0..4].copy_from_slice(&(team_id - 1).to_le_bytes());
        out[4..8].copy_from_slice(&(club_uid - 1).to_le_bytes());
        out[8..12].copy_from_slice(&(club_uid - 1).to_le_bytes());
        out[PLAYER_COUNT_OFFSET..PLAYER_COUNT_OFFSET + 2]
            .copy_from_slice(&(players.len() as u16).to_le_bytes());
        for (index, player) in players.iter().enumerate() {
            let at = PLAYERS_OFFSET + index * 4;
            out[at..at + 4].copy_from_slice(&player.to_le_bytes());
        }
        let trailer = PLAYERS_OFFSET + players.len() * 4;
        out[trailer..trailer + 4].copy_from_slice(&captain.unwrap_or(u32::MAX).to_le_bytes());
        out[trailer + 4..trailer + 8].copy_from_slice(&vice.unwrap_or(u32::MAX).to_le_bytes());
        out
    }

    fn append_record(game_db: &mut Vec<u8>, bytes: &[u8]) {
        game_db.extend_from_slice(bytes);
    }

    fn known(players: &[u32]) -> HashSet<u32> {
        players.iter().map(|value| value - 1).collect()
    }

    #[test]
    fn finds_and_decodes_a_team_record() {
        let mut game_db = Vec::new();
        append_record(
            &mut game_db,
            &record(10, 20, &[101, 102, 103], Some(101), None),
        );
        let found = find_team_list(&game_db, 10, Some(20), &known(&[101, 102, 103])).unwrap();
        assert_eq!(found.team_id, 10);
        assert_eq!(found.club_uid, 20);
        assert_eq!(found.players, vec![101, 102, 103]);
        assert_eq!(found.captain, Some(101));
        assert_eq!(found.vice_captain, None);
    }

    #[test]
    fn moves_a_player_from_an_earlier_record_to_a_later_one() {
        let mut game_db = Vec::new();
        append_record(
            &mut game_db,
            &record(10, 20, &[101, 102, 103], Some(102), None),
        );
        append_record(&mut game_db, &record(11, 21, &[201, 202], None, None));
        let before = game_db.len();
        let from =
            find_team_list(&game_db, 10, Some(20), &known(&[101, 102, 103, 201, 202])).unwrap();
        let to =
            find_team_list(&game_db, 11, Some(21), &known(&[101, 102, 103, 201, 202])).unwrap();
        let change = apply_transfer(&mut game_db, 102, &from, &to).unwrap();
        assert_eq!(game_db.len(), before);
        assert_eq!(change.from_players_before, 3);
        assert_eq!(change.from_players_after, 2);
        assert_eq!(change.to_players_before, 2);
        assert_eq!(change.to_players_after, 3);
        assert!(change.captaincy_cleared);
        let from_after =
            find_team_list(&game_db, 10, Some(20), &known(&[101, 102, 103, 201, 202])).unwrap();
        let to_after =
            find_team_list(&game_db, 11, Some(21), &known(&[101, 102, 103, 201, 202])).unwrap();
        assert_eq!(from_after.players, vec![101, 103]);
        assert_eq!(from_after.captain, None);
        assert_eq!(to_after.players, vec![201, 202, 102]);
    }

    #[test]
    fn moves_a_player_from_a_later_record_to_an_earlier_one() {
        let mut game_db = Vec::new();
        append_record(&mut game_db, &record(10, 20, &[101, 102], None, None));
        append_record(
            &mut game_db,
            &record(11, 21, &[201, 202, 203], Some(203), None),
        );
        let before = game_db.len();
        let from =
            find_team_list(&game_db, 11, Some(21), &known(&[101, 102, 201, 202, 203])).unwrap();
        let to =
            find_team_list(&game_db, 10, Some(20), &known(&[101, 102, 201, 202, 203])).unwrap();
        let change = apply_transfer(&mut game_db, 203, &from, &to).unwrap();
        assert_eq!(game_db.len(), before);
        assert!(change.captaincy_cleared);
        let from_after =
            find_team_list(&game_db, 11, Some(21), &known(&[101, 102, 201, 202, 203])).unwrap();
        let to_after =
            find_team_list(&game_db, 10, Some(20), &known(&[101, 102, 201, 202, 203])).unwrap();
        assert_eq!(from_after.players, vec![201, 202]);
        assert_eq!(to_after.players, vec![101, 102, 203]);
    }

    #[test]
    fn refuses_a_player_who_is_not_in_the_old_list() {
        let mut game_db = Vec::new();
        append_record(&mut game_db, &record(10, 20, &[101], None, None));
        append_record(&mut game_db, &record(11, 21, &[201], None, None));
        let from = find_team_list(&game_db, 10, Some(20), &known(&[101, 201])).unwrap();
        let to = find_team_list(&game_db, 11, Some(21), &known(&[101, 201])).unwrap();
        assert!(apply_transfer(&mut game_db, 101, &from, &from).is_err());
        assert!(apply_transfer(&mut game_db, 999, &from, &to).is_err());
    }

    #[test]
    fn rejects_a_record_whose_players_are_not_known() {
        let mut game_db = Vec::new();
        append_record(&mut game_db, &record(10, 20, &[101, 102], None, None));
        let error = find_team_list(&game_db, 10, Some(20), &known(&[101])).unwrap_err();
        assert!(matches!(error, Error::Unsupported(_)));
    }

    #[test]
    fn refuses_an_already_present_destination_player() {
        let mut game_db = Vec::new();
        append_record(&mut game_db, &record(10, 20, &[101], None, None));
        append_record(&mut game_db, &record(11, 21, &[101], None, None));
        let from = find_team_list(&game_db, 10, Some(20), &known(&[101])).unwrap();
        let to = find_team_list(&game_db, 11, Some(21), &known(&[101])).unwrap();
        assert!(apply_transfer(&mut game_db, 101, &from, &to).is_err());
    }
}
