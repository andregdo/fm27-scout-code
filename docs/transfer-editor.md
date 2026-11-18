# Player transfer editor

Status: **experimental, CLI and desktop, mechanically verified and accepted in game on
`27.0.0+2486695` and `27.0.1+2503222`.** Re-parse and roster verification also pass on
`27.1.0+2505117`; in-game loading on that build has not been tested.

The transfer editor moves a contracted player to another club's senior team in a new save
copy. It never modifies the source save. It uses the same `SaveFile::write_with_replacements`
path as the other editors, then re-parses the result and verifies every field it wrote.

## Scope

Supported:

- a player whose registered team is a club's senior team;
- a single current permanent contract whose team and club match the player's registration;
- a destination club with a stored senior-team player list;
- the same gender at both clubs.

Refused:

- free agents (no contract);
- loans and second-team registrations (contract team differs from the registered team);
- players with more than one contract chain record (offers, earlier contracts or a future
  transfer);
- destination clubs without a stored senior-team player list;
- the player's current club.

Not written:

- transfer fee, transfer history (`transfer_man`), club balance or transfer budget;
- a new squad number (the old number is cleared; the new club assigns one after loading);
- saved match selections or tactics;
- staff or player-coach roles.

## Save structures involved

The player record carries the team id at `+16` and the club join date at `+98`. The
in-effect contract record carries the team id at `+9`, the start date at `-19`, the squad
number at `-12`, the end date in the tail at `+28`, the squad status at `+36` and the
transfer/loan listing bits at `+39`. See [fm27-layout.md](fm27-layout.md).

Each team also has a stored player list: a counted array of `pindex + 1` values with a
46-byte header and a 38-byte trailer. The full layout is in
[fm27-layout.md](fm27-layout.md#team-player-lists). A transfer removes the player from the
old team's array, appends him to the new team's array, updates both counts, and clears a
captain or vice-captain reference to him in the old trailer. The two records change length
by four bytes in opposite directions, so `game_db` keeps its length.

The edit also clears the current contract's squad-number byte (`tag - 12`) to `255` when one
was assigned, matching the game's own completed transfer. The new club assigns a number
after the save loads.

## Evidence

The layout was read from two saves of the same career, one week apart
(`AP King - Milan (v02)` at 2029-02-09 and `(v03)` at 2029-02-02, build `27.0.1+2503222`).
In that week 70 players changed team or club and 26 changed their contract team, giving a
corpus of game-made moves. Bastien Meupiyou's move from Alverca to Sporting on 2029-02-06
showed the complete write set:

- the player record changed `team_id` from 1834 to 1185 and the club join date to
  2029-02-06;
- the contract chain went from an Alverca record plus a future Sporting record to a single
  Sporting record starting 2029-02-06;
- the player index moved from Alverca's senior-team list to Sporting's;
- the contract's squad-number byte went from 51 at Alverca to 255 (none) at Sporting;
- the `transfer_man` section kept a record with the player index, both teams, the agreed
  date (2029-01-30), the completion date (2029-02-06) and an amount that looks like the
  fee. The transfer editor does not edit that section yet.

The same team-list format was then checked on `27.0.0+2486695` (`BB KING - Parma Calcio`),
where Parma's senior list matched its players and the editor completed a Parma-to-Juventus
test transfer.

The two test saves then loaded normally in FM27 and each edited player appeared at the new
club:

- `FM27 Scout Transfer Test (Milan).fm` (`27.0.1+2503222`, 2029-02-09): Moncef Zekri moved
  from Sporting to Alverca.
- `FM27 Scout Transfer Test (Parma).fm` (`27.0.0+2486695`, 2026-12-11): Simone Ghidotti
  moved from Parma to Juventus.

This confirms save-level acceptance for a simple permanent move. It was not a per-screen
audit of squad numbers, saved selections or transfer history. The batch path that stages one
player edit and two transfers into the same destination is mechanically verified by
re-parsing the output, but that multi-move save has not been separately loaded in game.

## CLI

```bash
./target/release/fm27scout transfer "/path/to/save.fm" \
  --uid 2000449256 --to-club 2391 \
  --output transferred.fm --save-name "FM27 Scout Transfer Test"
```

`--dry-run` prints the planned field changes and both roster counts without writing.
`--force` allows replacing an existing output; the source path is always refused.

## Desktop app

In the player profile, **Transfer** opens a dialog that:

1. searches clubs by name and shows only clubs of the player's gender;
2. previews the move automatically when a destination is selected, using the in-memory
   editor and writing nothing;
3. shows both squad counts, the contract restart date, and whether listing, captaincy or
   squad number was cleared;
4. stages the move through **Add to pending changes** instead of writing a save.

Staged transfers appear in **Pending changes** together with staged player edits. **Save
all to a copy…** writes one new save containing every staged edit and transfer. The batch
applies all fixed-width player edits first, then each transfer in turn; because a team-list
splice moves later bytes, the editor rebuilds its player and person indexes after every
move. The written save is re-parsed once and every edit and transfer is verified before the
output is accepted.

The preview uses the `preview_transfer` Tauri command; the batch writer uses
`save_pending_changes`. The dialog works in browser mock mode with synthetic data. A staged
player edit for the same player is called out in the dialog because both are written
together.

## Verification

The command checks the new save by:

1. re-parsing it and finding the player;
2. checking the player's club uid, team id, contract team id and contract start date;
3. opening the new save's editor and reading both team player lists;
4. checking the player index is absent from the old list and present in the new one;
5. checking the squad number is clear (`0` or `255`).

It also asserts that `game_db` keeps its length. Removal of the output follows any failed
check. A batch save verifies every transfer against the same re-parsed result and a single
opened editor, then removes the output if any edit or move fails verification. The ignored
real-save desktop test stages one PA edit and two transfers into the same destination and
checks all three after re-parsing the output.

This proves the save is internally consistent for the fields the editor models, and the
in-game test above now confirms that the game accepts a simple permanent move. The general
in-game test remains:

1. copy the original save and keep it untouched;
2. write a transfer copy with a distinctive save name;
3. close FM before replacing the save file, then load the copy;
4. advance several days and check the player's club, the squad screen, the next match and
   the transfer window;
5. if anything is wrong, discard the copy and report which screen disagreed.

The first test player should be a fringe player who is not a captain, is not in the saved
match selection and is not transfer-listed, so a failure is easy to isolate. The editor
clears the captain/vice references, listing bits and squad number, but it cannot change a
saved lineup.

## Open questions

- **Saved tactics and selections.** The save has tactical data, but the transfer editor
  does not touch it. FMLibreEditor empties selection slots at runtime; the save
  counterpart is unmapped.
- **Transfer history and fee.** `transfer_man` holds a record for game-made transfers. The
  editor does not insert one, so the transfer may not appear in the player's history and no
  fee is shown or paid.
- **Free agents.** 1,225 of 2,273 free agents in the reference save have no contract record
  at all. Signing them needs a contract record the editor cannot yet create.
- **Loans.** Creating one needs a loan registration record and loan terms; the editor only
  supports permanent moves.
- **Player-coaches.** The save does not expose a simple flag in the parsed player row. A
  player who also holds a staff job may need an extra guard before a move.

## Next steps

1. Map the saved-selection fields, then clear them in the same edit.
2. Decode `transfer_man` enough to write a history record and a fee, and reuse the existing
   club finance editor for the money.
3. Consider free-agent signings only after contract-record creation is understood.
4. Re-test a move that involves a captain, a transfer-listed player and a player in the
   saved match selection, and check the new club's squad-number screen, because the current
   in-game acceptance used a simple move.
