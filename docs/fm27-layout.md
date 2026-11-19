# FM27 layout quick reference

Builds `27.0.0+2486695`, `27.0.0+2493426`, `27.0.1+2494687`, `27.0.1+2503222`
and `27.1.0+2505117`, `game_db` schema 4000. All five builds share the layout
below except for the club full-finance closing block described under
[Clubs](#clubs). Offsets are decimal bytes unless noted.

## Save container

```text
header size            26
magic                  02 01 66 6d 66 2e
trailer pointer u64 @  9
trailer offset         9 + pointer
trailer header         9 bytes
zstd frame             1..n, then directory frame
section header         03 01 + reversed extension, schema u16 @ +6
```

## Player record

Marker `01 00 6c 07` is at record +102. Record extent 122 bytes.

```text
pindex              -19
uid                 -15
uid copy            -11
object kind         -7
home reputation     -6
current reputation  -4
world reputation    -2
CA                  +0   u16
PA                  +2   i16
reputation bucket   +8   u8
team id             +16  u32
positions           +24  15 × u8
attributes          +39  54 × u8 (feet at 24/25)
transfer value      +93  u32
club join date      +98  game date
match sharpness     +106 u16
condition           +110 u16
height cm           +121 u8
identity flags      closing header -7  u8 (bit 0x10 = woman)
```

The `pindex + 1, uid, uid` closing header follows the record. Its identity flags byte sits
seven bytes before the header; bit `0x10` is set for women and clear for men.

Display attributes: `max(1, (raw + 2) / 5)`.

## Person block (FM27 shift)

```text
window start        record + 100
personality offset  birth + 22  (FM26: 21)
zero region         birth + 15, 6 bytes (FM26: 14, 7)
nation id           birth + 14
relation present    birth + 38
relation count      birth + 39
relation entries    birth + 40, 16 bytes
legal name length   birth - 4 (u32)
legal name text     block_start + 19
trait bits          block_start - 8
first name id       block_start + 0
surname id          block_start + 5
common name id      block_start + 10
```

FM27 requires exactly one valid candidate.

## Clubs

Counted table rows; FM27 changes:

```text
full_finance[0].lead_bytes = 66   (FM26: 70)
after_finance[5..7].count_bytes = 2 (FM26: 1)
```

The full-finance chain closes with a fixed block. It is 67 bytes through
`27.0.1+2503222` and 72 bytes on `27.1.0+2505117`:

```text
+0..21   unchanged block fields
+22      status byte (0..=6 observed; meaning not yet decoded)
+23      game date (4 bytes; 1900-01-01 / null on the tested save)
+27..    the previous +22.. block tail, shifted by five bytes
```

All 319 club records that carry a full-finance chain use the 72-byte block on
the tested 27.1.0 save. The parser accepts both sizes and keeps the one whose
following `after_finance` walk lands on the row's own footer, so the footer
ordinal is the structural check.

Initial club uid is 0 and has no anchor; it comes from the table's first row.
Anchor scan fallback: `FF FF FF FF` at record +17.

### Club finance chain

A club that keeps a monthly series stores a counted run of 49-byte rows inside
its own record. The count is a u32 at `row - 4`; the full chain count range is
3–1000, with a balanced 1–2 row short chain as a fallback.

```text
+0   tag                 01
+1   balance             i32
+5   transfer allocated  i32
+9   transfer remaining  i32
+13  wage budget         u32
+17  wage payroll        u32
+21  income excl. transfers i32
+25  net transfers       i32
+29  wage bill           i32
+33  net                 i32
+37  expenditure excl. transfers i32
+41  total income        i32
+45  total expenditure   i32
```

The corporate-facilities byte sits 50 bytes past the end of the chain, range
1–20. Only clubs in the tracked league nations keep a chain.

### Club sponsor contracts

A club's primary sponsor list is stored inside its own record. It usually follows
the monthly finance chain, but clubs without a full finance series also carry
one, so the parser scans the whole club record for the list. The list is
structurally owned: a missing or invalid prefix yields no data, later
sponsor-shaped bytes are never a fallback, and two valid lists in one record are
treated as ambiguous and rejected.

Prefix, with offsets from the start of the list:

```text
+0    marker                 01 00 6c 07
+25   first prefix date      game date or the null-date marker
+29   second prefix date     game date or the null-date marker
+71   prefix entry count     u8 (0..=255)
+72   prefix entries         19 bytes each, reserved bytes zero at +8..+14
```

The counted entries are followed by a fixed suffix. The suffix is 68 bytes
through `27.0.1+2503222` and 73 bytes on `27.1.0+2505117`. The extra five bytes
are the same status byte and game date inserted at suffix offset 22 that the
full-finance closing block gained; the later fixed bytes, dates and the primary
sponsor count shift by five. The primary sponsor count is the last suffix byte.
That many 25-byte rows follow:

```text
+0    tag                  02
+1    kind                 u8 (raw type code)
+2    start                game date
+6    end                  game date
+10   flag                 0 or 1
+11   total value          u32
+15   reputation at signing u16
+17   unknown              u16
+19   percentage           u16, 0..100
+21   annual value         u32, 0 means not stored
```

The parser validates the prefix marker, both prefix dates (or null markers),
the reserved bytes of every 19-byte entry, the suffix fixed bytes and dates,
the primary count, and every row: tag, flag, `total <= 2,000,000,000`,
`annual <= total`, `percentage <= 100`, both dates valid, and
`end - start >= -1 day`. FM27 can store a row that ends one day before it
starts. A positive count with any failing row rejects the whole list; an
explicit zero count is a decoded empty list. Both suffix sizes are tried, and
two valid shapes are treated as ambiguous and rejected.

The annual field is zero for many deals, including some active ones. The parser
reports that as unknown (`null`) rather than zero income; a term is derived from
the dates the way the game screen counts it. The percentage falls on relegation,
or rises on promotion for kind 22 (kit manufacturer), and the reputation is the
club's value when the deal was signed.

Coverage now follows the club record rather than the finance series: 167 clubs
and 1,094 rows on the `27.0.0` reference save, 318 clubs and 1,627 rows on
`27.0.1`, and 318 clubs and 1,625 rows on `27.1.0`. One list on each of the
27.0.1/27.1.0 saves is rejected because a row stores sentinel-like values near
`u32::MAX`.

The rows store contract terms and a raw type code, not a sponsor name. The
game's sponsorship screen confirmed codes 1 Main Kit, 3 and 23 Stadium, 4
General, 5 Individual TV Deal, 7 Club Membership, 8 Away Kit, 9 Additional Kit
Sponsor, 11 Back of Shirt, 12 Shorts, 13 Training Kit, 14 Youth Team, 15
Training Ground, 16 Continental Competition, 19 Shirt Sleeve and 22 Kit
Manufacturer on builds 27.0.0 and 27.0.1. Codes 2, 6, 17 and 18 remain unnamed,
and 27.1.0 keeps raw codes until its screen is checked.

### Club facility rows

Training, youth, junior-coaching and youth-recruitment ratings live in a club-info
row outside the club record. A row starts with the stored club index plus one
(u32) and the training-facilities byte at `+4`. The row header is followed by a
variable-length list, then the supporter-share block:

```text
supporter shares       six bytes, sum 100
supporter traits       +15, six bytes (1–20)
supporter field        +21, six bytes
tag                    +27  02
youth facilities       +63  u8 (1–20)
junior coaching        +64  u8 (1–20)
youth recruitment      +65  u8 (1–20)
```

The supporter field at `+21` is six zero bytes on 27.0.0 saves. On
27.0.1+2503222 it holds a second supporter-share block that also sums to 100.
Both shapes are accepted; requiring zeros rejects almost every row on the newer
build.

Rows are not contiguous and only the tracked clubs (those with a finance chain)
have one. A club's key can also occur inside another record, so locate every
plausible row by its club-index key and share block, then keep the longest
sequence ordered by club index: the real facility table is sorted by that key.
Do not simply take the first byte match or walk the rows sequentially.

English Youth Level is not stored in this row or in the save. It comes from the
game's `eng_youth_grades.edt` file, which maps club Unique IDs to EPPP categories
1–4 for the current season. The parser reads that file from the installed game
and leaves the field empty when the file is unavailable.

### Stadium table

Rows are 181 bytes, unless bit `0x10` of the flags byte at +156 is set: then a
u32 name length at +158 and that many UTF-8 bytes at +162 follow, and the row is
`181 + 4 + name_length` bytes.

```text
+0   ordinal             u32
+4   uid                 u32 (stored uid = uid - 1)
+8   uid copy            u32
+12  zero byte           u8
+13  all-seater capacity u32
+21  expansion capacity  u32
+29  owner club index    u32 (FFFFFFFF = none)
+34  capacity            u32
+38  pitch length        u16 (decimetres)
+40  pitch width         u16
+50  built date          game date
+54  rebuilt date        game date
+67.. pitch min/max      four u16
+156 flags               u8
```

The table is located by twelve consecutive rows carrying ordinals 1, 2, 3 …,
the uid twice and a zero byte. Ownership is stored; home-ground use is not.

## Contract

```text
selector            pindex + 1
chain tag offset    selector offset - 5
tag                 01 00 6c 07
team id             +9
wage                +17
start date          -19
squad number        -12  (1..=99; 0 and 255 mean none)
tail base            -66
tail step            29
tail event count     +42
tail end date        +28
tail squad status    +36
tail list flags      +39  bit 0 transfer, bit 1 loan
tail status code     +40  often 5, not a listing indicator
clause step          8
clause entries       base - 4
head gate            base - 35 (value 5)
head type            base - 33
```

The squad number is the byte at `tag - 12` (tail +54). Across both reference saves it
holds only `0`, `1..=99` or `255`; assigned numbers are unique within a team, and the
distribution falls off with the number, as shirt numbers do. `0` and `255` both appear
for players without a number. The game's own completed transfer changed the departing
player's number to `255`; the new club assigns one after the move. Rust implementation:
`PlayerEditor::squad_number`; a transfer clears the byte.

Loans: tail-shaped block before a no-terms chain record, marker `FFFFFFFF` at +24,
end date at +28.

## Team player lists

FM27 stores each team's players as a counted array of player indexes, one record per
team. The record starts at the team-id word (there is no record tag):

```text
team id - 1         +0   u32
club uid - 1        +4   u32
club uid - 1        +8   u32 (copy of +4)
unmapped header     +12 .. +45
player count        +46  u16
players             +48  u32 player index + 1, count times
captain reference   +48 + 4 * count     u32 player index + 1, or FFFFFFFF
vice-captain ref    +52 + 4 * count     u32 player index + 1, or FFFFFFFF
unmapped trailer    ...  FFFFFFFF and other bytes, ending 38 bytes after the players
```

A record is `86 + 4 * count` bytes. The next team record starts immediately after it.
The player values are **`pindex + 1`**, the same selector space as the contract chain; a
value of zero or `FFFFFFFF` names no player. The first two trailer words are always one
of the record's own players when set; they are treated as the captain and vice-captain,
but that identification has not been confirmed against the game UI. The rest of the
trailer is copied unchanged.

The records cover the teams whose player lists the save maintains, including reserve and
youth teams. A reserve team can be stored under its own club object, so the header's
club uid is not always the parent club's uid. They are not one contiguous walk: segments
of team records are interleaved with other records. Locate a specific team by scanning
for `team_id - 1`, requiring the doubled club uid, a sane count, every player value to
resolve to a known player record, and every set trailer reference to be one of the
record's players.

Observed on `27.0.0+2486695` and `27.0.1+2503222` saves: a plain save stores the senior
team's list exactly matching the players whose `team_id` is that team, and a transfer the
game completed removed the player from the old list and appended him to the new one. The
total `game_db` length is unchanged by such a move: one record loses four bytes and the
other gains four.

## Competitions

The stage table sits in the tail of `game_db`. Rows are 33 bytes:

```text
previous stage id   +0   u32
stage id            +4   u32
stage id copy       +8   u32
zero byte           +12  u8
competition id      +13  u32
```

A row decodes when the copy equals the stage id, the zero byte is zero, the previous id is
`id - 1` or `FFFFFFFF`, and the id is strictly between 0 and 200000. The table is found by a 200-row chain
and walked with byte-level resynchronisation.

Editor database ids come from `FF × 16 + 01` markers. The record starts 30 bytes after the
marker:

```text
entity id           +0   u32 (1..199999)
database id         +4   u32
database id copy    +8   u32 (must equal +4)
constants           -8 7, -6 0, -4 7, -1 FF, +12 0, +15 1,
                    +22..+25 0, +26 1, +32 2, +40 3, +55 6, +60 7
```

The database id is the key for `shared/competition-names.csv`; the save stores no names.

## Season statistics frame

The season-statistics section is not listed in the directory. It is one frame in the
unlisted gap after `non_pl_hist_ls`:

```text
region            unlisted_after_non_pl_hist_ls
section header    03 01 74 61 64 2e 0f 00
```

Record layout:

```text
record marker     01
player key        +1   u32 (pindex + 1)
header            +5   16 bytes
own slots         +21  8 slots
block             +?   20 bytes
other team count  u32 (max 16)
other team row    lead 1 byte, team id u32, 6 slots
rating item count u32 (max 256)
rating items      count × 8 bytes
form entries      count byte (max 16)
form entries      count × 3 bytes
```

A slot is `00` when absent or a 155-byte line starting with `01 07`. The list ends with
`00` followed by exactly 16 footer bytes.

Line fields use the FM26 offsets, with `line_bytes = 155` for FM27. Goalkeepers and outfield
players share offsets 35, 37, 69/71, 83 and 95; natural position GK selects the goalkeeper
set. `distance_km == 0` marks the whole advanced-stat block missing (key passes, progressive
passes, interceptions, headers, clearances, blocks, sprints, possession, saves, xGP).

Rust implementation: `crates/fm27-core/src/season_stats.rs`; container region support:
`crates/fm27-core/src/container.rs`.

## Fixture calendar

The career's fixture calendar is another unlisted run in
`unlisted_after_non_pl_hist_ls`, separate from the season-statistics frame. FM27
uses the FM26 layout with a two-byte shorter record:

```text
record bytes       66
marker             -12  1F
stage id           -11  u32
home team id        +0  u32
sentinel            +4  FF
sentinel           +11  FF
away team id        +7  u32
kick-off date      +13  two u16 words
kick-off year      +15  u16 (high word of the date)
season start year  +22  u16
```

The locator origin is the home team id, so the marker and stage id are before
it. The kick-off year must be within -7/+8 years of the save clock. Team ids are
below 3,000,000. The calendar is the largest run of accepted records in a
frame, with runs split by gaps over 1 MiB; the scan reads 16 KiB frame heads,
then decompresses only the dense frame and a physically adjacent continuation
when it holds at least 10% of the main run. The season filter uses the 1 July
boundary. Rust implementation: `crates/fm27-core/src/fixtures.rs`.

## Staff objects

Staff objects use the person-header shape with kind byte `1` at header +12. Verified on
`27.0.0+2486695`, `27.0.0+2493426`, `27.0.1+2494687`, `27.0.1+2503222` and
`27.1.0+2505117`:

```text
header             pindex u32
uid                +4   u32 (previous person's Unique ID)
uid copy           +8   u32
kind               +12  01
entry count        +13  u8 (7-byte entries follow)
ability block      header + 21 + 7 * entry_count
  CA               +0   u16 (1..200)
  PA               +2   i16 (1..200)
  r4               +4   u8 (not the displayed job; never used as one)
  sentinel         +13  12
  preferences      +14  26 bytes (1..100)
  block 40         +40  26 bytes (1..100)
```

The closing person header has `pindex + 1` and a doubled uid; its uid is this person's
Unique ID. Job codes are read from the current contract record at `tag + selector + 16`,
or from a no-contract role record at `tag + 32`:

```text
contract tag      01 00 6c 07
selector          tag + 5   u32 (pindex + 1)
team              tag + 9   u32
wage              tag + 17  u32
job               selector + 16
offer marker      selector + 19 (a second tag marks an offer, not a held job)

no-contract tag   01 00 6c 07
lead              tag + 14  05 05
selector          tag + 16  u32 (pindex + 1)
team              tag + 20  u32
job               tag + 32  u8
```

The job byte uses the game's job codes. `staff_job_name` in `layout.rs` names the verified
ones: 44 is Chief Scout, 46 Doctor and 66 Owner; 24 is General Manager, the chairman-title
variant kept apart from 8 Managing Director and 70 President.

Staff profile attributes, displayed on the 1–20 scale:

```text
preferences +4    authority               1..20
preferences +12   working with youngsters  1..20
preferences +13   determination            1..100 (divide by 5 for display)
block 40 index    1 goalkeeping            2 judging player ability
                  3 judging player potential 4 people management
                  5 motivating             6 physiotherapy
                  7 tactical knowledge     8 attacking
                  9 defending              10 fitness
                  11 possession            12 technical
                  13 tactical              18 data analysis
                  21 sports science        23 negotiating
                  24 judging staff ability 25 set pieces
```

Role suitability is a 15-byte vector found inside the person's object, within 4 KiB of its
header: each byte is 1..20, preceded by `01`, followed by four zero bytes, and at least one
value is 10 or higher. The order is **Manager, Assistant Manager, Coach, Physio, Scout,
Goalkeeping Coach, Fitness Coach, Chairperson, Director of Football, Head of Youth
Development, Loan Manager, Technical Director, Data Analyst, Sports Scientist, Set Piece
Coach**.

Rust implementation: `crates/fm27-core/src/staff.rs`.

## Managed club

`humans` schema 24 names the human managers:

```text
+8   u16 human-manager count
+10  u32 first manager's selector (person id + 1)
```

The selector appears five bytes after a contract record tag in `game_db`. The
`save_game_summary` strings are the independent check: a club short name
followed by its club uid; the adjacent preceding string is the manager name.
Details and conflict rules are in [parser-notes.md](parser-notes.md).

Rust implementation: `crates/fm27-core/src/managed.rs`.

## Section schemas observed on the reference save

`game_db` 4000, `game_info` 47, `save_game_summary` 31, `contract_man` 14,
`person_record_manager` 30, `person_db_changes` 3754, `player_stats_hist_dt` 7,
`regeneration_manager` 47, `training_man` 32, `news` 2957, `humans` 24.
Other sections are not required by the scouting MVP.
