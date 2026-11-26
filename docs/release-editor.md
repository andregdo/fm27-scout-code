# Player release editor

Status: **experimental, CLI and desktop, mechanically verified by re-parsing on
`27.0.0+2486695`, `27.0.0+2493426`, `27.1.0+2505117` and `27.1.1+2511875`.** In-game loading
of a released save has not been tested yet. The write set follows FMLibreEditor's
live-memory release, which was checked in game on the macOS Steam build (`release
--restore`: the club lost one player and its captain, and both came back).

The release editor turns a contracted player into a free agent in a new save copy. It never
modifies the source save. It uses the same `SaveFile::write_with_replacements` path as the
other editors, then re-parses the result and verifies every field it wrote.

## Scope

Supported:

- a player registered with a club team (senior, reserve or youth) whose team id resolves to
  a club;
- a current permanent contract with terms whose club matches the player's registered club;
- a stored player list for the player's registered team;
- any number of the club's team lists that also hold the player.

Refused:

- free agents (no registered team);
- loans (the contract club differs from the player's club);
- expired contracts, contracts without terms and players whose contract chain cannot be
  decoded;
- future contract records or offers (a chain record starting after the in-game date);
- players whose registered team has no stored player list, or whose registered team does
  not list them;
- players listed more than once in a team list.

Not written:

- the contract record: team, wage, start, end, squad status, clauses, squad number and
  transfer/loan listing bits are all left as they were;
- the club join date and the player's last-club fields;
- transfer history (`transfer_man`), club finances or the club's wage bill;
- saved match selections or tactics;
- staff or player-coach roles.

## Save structures involved

The player record carries the team id at `+16`; the game's normal free agents store
`FFFFFFFF` there (`MISSING_U32`). In the reference save, 2,058 of 2,072 club-less players
use that marker; the other 14 store 0 and carry no contract or join date. Each team has a
stored player list: a counted array of `pindex + 1` values with a 46-byte header and a
38-byte trailer. See [fm27-layout.md](fm27-layout.md#team-player-lists).

A release:

1. sets the player record's team id to `FFFFFFFF`;
2. removes the player's `pindex + 1` from every list of the club's teams that holds him;
3. clears a captain or vice-captain reference to him in those lists.

The contract record is deliberately left in place. The game's own free agents often keep a
contract record with nothing pointing at it: in the reference 2030 save, 835 of 2,072 free
agents still decode a contract, 529 keep a chain record, and 499 name a club the parser can
resolve.
No free agent in that save has an expired chain record, which matches the game dropping a
contract when it expires but leaving the record of a released player.

Unlike a transfer, the two roster records do not trade bytes, so `game_db` shrinks by four
bytes for each list the player leaves. `write_with_replacements` updates the frame and
directory sizes; the editor does not require the section length to stay constant.

## Evidence

FMLibreEditor's live-memory recon (`docs/recon/fm27-transfers.md` in that project) records
what the game does when a player leaves:

- the game's free agents have a null team pointer and a null contract pointer;
- a release takes the player out of every list of his club's teams that holds him, empties a
  captaincy he holds, and then clears his team and contract pointers;
- the contract object is left as it was, because only the player pointed at it;
- the game also records the last club and the day he left; the save editor leaves those
  fields alone, as FMLibreEditor's release does.

FMLibreEditor's roadmap records that moving a contract's end date to the game date is
**not** a release: on the Windows Steam build the game showed the new end and ended none of
four such contracts in six days. A release must clear the team registration and the roster
lists.

The save-side end state was checked against the reference career save: free agents store
`FFFFFFFF` as their team id, keep their last club join date, and may keep a contract chain
record naming the last club. A spot check of eight free agents with a chain record found
none of them in any stored team list of their last club. The release editor reproduces that
state.

## CLI

```bash
./target/release/fm27scout release "/path/to/save.fm" \
  --uid 200515 --output released.fm --save-name "FM27 Scout Release Test"
```

`--dry-run` prints the planned field changes, the squad counts and the lists the player
leaves without writing. `--force` allows replacing an existing output; the source path is
always refused.

## Desktop app

In the player profile, **Release** opens a dialog that:

1. previews the release automatically, using the in-memory editor and writing nothing;
2. shows the current club, the registered team's squad count before and after, any extra
   reserve/youth lists the player leaves, and whether a captaincy is cleared;
3. states that the contract, wage, squad number and listing bits are kept;
4. stages the release through **Add to pending changes** instead of writing a save.

A staged release appears in **Pending changes** with staged player edits and transfers.
**Save all to a copy…** writes one new save containing every staged change. The batch applies
all fixed-width player edits first, then each transfer, then each release; every roster edit
rebuilds the player and person indexes before the next one. A player staged for both a
transfer and a release is refused, and the dialogs point at the conflicting pending change.
The written save is re-parsed once and every edit, transfer and release is verified before
the output is accepted.

The preview uses the `preview_release` Tauri command; the batch writer uses
`save_pending_changes`. The dialog works in browser mock mode with synthetic data.

## Verification

The command checks the new save by:

1. re-parsing it and finding the player;
2. checking the player's club and team id are both empty;
3. checking the contract still names the same club, team, wage, start and end dates;
4. opening the new save's editor and reading every list the release touched;
5. checking the player index is absent from each of them.

It also checks that `game_db` shrank by exactly four bytes per rewritten list. Removal of the
output follows any failed check. A batch save verifies every release against the same
re-parsed result and a single opened editor, then removes the output if any edit, move or
release fails verification. The ignored real-save tests `release_real_save` and
`previews_and_saves_a_release_in_one_batch` cover the single and batch paths.

This proves the save is internally consistent for the fields the editor models. The in-game
test remains:

1. copy the original save and keep it untouched;
2. write a release copy with a distinctive save name;
3. close FM before replacing the save file, then load the copy;
4. advance several days and check the player is a free agent, no longer in the squad, and
   can be signed by another club;
5. if anything is wrong, discard the copy and report which screen disagreed.

The first test player should be a fringe player who is not a captain, is not in the saved
match selection and is not transfer-listed, so a failure is easy to isolate. The editor
clears captain/vice references but cannot change a saved lineup.

## Open questions

- **Saved tactics and selections.** The save has tactical data, but the release editor does
  not touch it. FMLibreEditor empties selection slots at runtime; the save counterpart is
  unmapped.
- **Future offers and transfers.** A chain record that starts after the game date is refused
  rather than cancelled, because `transfer_man` offer states are not written.
- **Player-coaches.** A player who also holds a staff job may need an extra guard; the
  transfer editor has the same open question.
- **Last-club history.** The game sets the last club and leaving date on its own release.
  The save editor leaves the join date and history records untouched, so the player's
  profile may still name the club as his last one.
