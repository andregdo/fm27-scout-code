//! Confirmed future transfers in the `transfer_man` section.
//!
//! The game's transfer manager serialises each offer as a record starting with a
//! kind byte, the marker `14 00`, the player's index + 1, two team ids and the
//! offer's state. State 19 means the clubs and the player have agreed the move;
//! states 0 and 8 are offers still waiting on a club or the player, and state 20
//! is a move the game has already made. Only state 19 confirms a future transfer.
//!
//! The section has no documented count or table header, so the reader walks it
//! for the marker and accepts a record only when the player index, both team ids
//! and the offer kind all resolve. A record that fails any check is skipped
//! rather than guessed at.

use std::collections::{HashMap, HashSet};

use crate::clubs::ClubIndex;
use crate::layout::{MISSING_U32, read_u32};
use crate::player_scan::PlayerRecords;

/// Both clubs and the player have accepted the offer.
const AGREED_STATE: u8 = 19;
/// The marker that follows the offer kind in a transfer-manager record.
const OFFER_MARKER: [u8; 2] = [0x14, 0x00];
/// Offer kinds seen in the supported FM27 builds. Kind 2 is a transfer with a
/// fee, 8 a free move at the end of a contract and 3 most loans; the others are
/// observed variants, kept so a confirmed move is not dropped.
const OFFER_KINDS: &[u8] = &[1, 2, 3, 4, 7, 8, 10];

/// Destination club uids of the agreed future moves, keyed by player index.
///
/// A player with no agreed offer is absent from the map. Both team ids in the
/// offer resolve to a club and are kept because a move to a reserve or youth
/// team names that side in one of them; matching happens at club level.
pub fn read_agreed_future_clubs(
    section: &[u8],
    records: &PlayerRecords,
    clubs: &ClubIndex,
) -> HashMap<u32, HashSet<u32>> {
    let mut agreed: HashMap<u32, HashSet<u32>> = HashMap::new();
    if section.len() < 32 {
        return agreed;
    }
    // The marker starts at offset 1, so the final record still has 30 bytes of
    // state and terms after it.
    for offset in 1..section.len() - 30 {
        if section[offset] != OFFER_MARKER[0] || section[offset + 1] != OFFER_MARKER[1] {
            continue;
        }
        if !OFFER_KINDS.contains(&section[offset - 1]) {
            continue;
        }
        let Some(selector) = read_u32(section, offset + 2) else {
            continue;
        };
        let Some(pindex) = selector.checked_sub(1) else {
            continue;
        };
        if !records.by_pindex.contains_key(&pindex) {
            continue;
        }
        if section[offset + 6] != 0 || section[offset + 11] != 0 {
            continue;
        }
        let Some(team_one) = read_u32(section, offset + 7) else {
            continue;
        };
        let Some(team_two) = read_u32(section, offset + 12) else {
            continue;
        };
        if team_one == MISSING_U32 || team_two == MISSING_U32 {
            continue;
        }
        let Some((club_one, _)) = clubs.fielding_team_to_club.get(&team_one) else {
            continue;
        };
        let Some((club_two, _)) = clubs.fielding_team_to_club.get(&team_two) else {
            continue;
        };
        if section[offset + 28] != AGREED_STATE {
            continue;
        }
        let destinations = agreed.entry(pindex).or_default();
        destinations.insert(*club_one);
        destinations.insert(*club_two);
    }
    agreed
}

#[cfg(test)]
mod tests {
    use super::*;

    const PINDEX: u32 = 41;
    const CLUB_UID: u32 = 99;
    const TEAM_ID: u32 = 7;

    fn records_with_player() -> PlayerRecords {
        PlayerRecords {
            by_pindex: HashMap::from([(PINDEX, 0)]),
            ..PlayerRecords::default()
        }
    }

    fn clubs_with_team() -> ClubIndex {
        ClubIndex {
            fielding_team_to_club: HashMap::from([(TEAM_ID, (CLUB_UID, 0))]),
            ..ClubIndex::default()
        }
    }

    fn offer(state: u8) -> Vec<u8> {
        let mut bytes = vec![0u8; 128];
        let marker = 16;
        bytes[marker - 1] = 2;
        bytes[marker] = OFFER_MARKER[0];
        bytes[marker + 1] = OFFER_MARKER[1];
        bytes[marker + 2..marker + 6].copy_from_slice(&(PINDEX + 1).to_le_bytes());
        bytes[marker + 7..marker + 11].copy_from_slice(&TEAM_ID.to_le_bytes());
        bytes[marker + 12..marker + 16].copy_from_slice(&TEAM_ID.to_le_bytes());
        bytes[marker + 28] = state;
        bytes
    }

    #[test]
    fn an_agreed_offer_marks_the_destination_club() {
        let agreed = read_agreed_future_clubs(
            &offer(AGREED_STATE),
            &records_with_player(),
            &clubs_with_team(),
        );
        assert_eq!(agreed.get(&PINDEX), Some(&HashSet::from([CLUB_UID])));
    }

    #[test]
    fn pending_and_made_offers_are_not_confirmed_moves() {
        for state in [0, 8, 20] {
            let agreed =
                read_agreed_future_clubs(&offer(state), &records_with_player(), &clubs_with_team());
            assert!(agreed.is_empty(), "state {state} must not confirm a move");
        }
    }

    #[test]
    fn a_record_with_an_unknown_team_is_skipped() {
        let mut bytes = offer(AGREED_STATE);
        bytes[23..27].copy_from_slice(&MISSING_U32.to_le_bytes());
        let agreed = read_agreed_future_clubs(&bytes, &records_with_player(), &clubs_with_team());
        assert!(agreed.is_empty());
    }
}
