//! Player and person record layouts, plus the position, attribute, personality
//! and trait tables that name their stored values.

#[derive(Debug, Clone, Copy)]
pub struct PlayerLayout;

impl PlayerLayout {
    pub const MARKER: &'static [u8] = &[0x01, 0x00, 0x6c, 0x07];
    pub const MARKER_OFFSET: usize = 102;
    pub const PINDEX_OFFSET: i32 = -19;
    pub const UID_OFFSET: i32 = -15;
    pub const UID_COPY_OFFSET: i32 = -11;
    pub const OBJECT_KIND_OFFSET: i32 = -7;
    pub const HOME_REPUTATION_OFFSET: i32 = -6;
    pub const CURRENT_REPUTATION_OFFSET: i32 = -4;
    pub const WORLD_REPUTATION_OFFSET: i32 = -2;
    pub const CA_OFFSET: usize = 0;
    pub const PA_OFFSET: usize = 2;
    pub const REPUTATION_BUCKET_OFFSET: usize = 8;
    pub const TEAM_ID_OFFSET: usize = 16;
    pub const RATINGS_OFFSET: usize = 24;
    pub const RATINGS_COUNT: usize = 15;
    pub const ATTRIBUTES_OFFSET: usize = 39;
    pub const ATTRIBUTE_COUNT: usize = 54;
    pub const LEFT_FOOT_INDEX: usize = 24;
    pub const RIGHT_FOOT_INDEX: usize = 25;
    pub const TRANSFER_VALUE_OFFSET: usize = 93;
    pub const TRANSFER_VALUE_PLACEHOLDER: u32 = 300_000_000;
    pub const CLUB_JOIN_DATE_OFFSET: usize = 98;
    pub const MATCH_SHARPNESS_OFFSET: usize = 106;
    pub const CONDITION_OFFSET: usize = 110;
    pub const HEIGHT_OFFSET: usize = 121;
    pub const DECODE_EXTENT: usize = 122;
    pub const ALLOCATED_UID_MINIMUM: u32 = 2_000_000_000;
    /// Flags byte in the person's identity block, seven bytes before the
    /// `pindex + 1, uid, uid` closing header. This is the save-file counterpart
    /// of the live-memory identity block (`identity + 0x19`, per FMLibreEditor's
    /// FM27 recon). Bit [`Self::IDENTITY_WOMAN_FLAG`] is set for women.
    pub const IDENTITY_FLAGS_OFFSET_FROM_HEADER: i32 = -7;
    pub const IDENTITY_WOMAN_FLAG: u8 = 0x10;
}

#[derive(Debug, Clone, Copy)]
pub struct PersonLayout;

impl PersonLayout {
    pub const WINDOW_START_OFFSET: usize = 100;
    /// FM27 moved the person block one byte later than FM26.
    pub const PERSONALITY_OFFSET_FROM_BIRTH: usize = 22;
    pub const PERSONALITY_COUNT: usize = 8;
    pub const DATE_ZERO_BYTES_OFFSET_FROM_BIRTH: usize = 15;
    pub const DATE_ZERO_BYTES_COUNT: usize = 6;
    pub const NATION_ID_OFFSET_FROM_BIRTH: usize = 14;
    pub const RELATION_PRESENT_OFFSET_FROM_BIRTH: usize = 38;
    pub const RELATION_COUNT_OFFSET_FROM_BIRTH: usize = 39;
    pub const RELATION_ENTRIES_OFFSET_FROM_BIRTH: usize = 40;
    pub const RELATION_ENTRY_BYTES: usize = 16;
    pub const LEGAL_NAME_LENGTH_MIN: usize = 2;
    pub const LEGAL_NAME_LENGTH_MAX: usize = 80;
    pub const LEGAL_NAME_OFFSET_FROM_BLOCK_START: usize = 19;
    pub const TRAIT_BITS_OFFSET_FROM_BLOCK_START: i32 = -8;
    pub const NAME_ID_LIMIT: u32 = 1 << 23;
    pub const SECOND_NATION_PAIR: (u8, u8) = (8, 9);
    pub const HOME_GROWN_NATION_PAIR: (u8, u8) = (8, 70);
    pub const HOME_GROWN_CLUB_PAIR: (u8, u8) = (1, 72);
    pub const SECOND_NATION_QUALIFIERS: [u8; 2] = [15, 100];
    pub const RELATION_SENTINEL_VALUE: u8 = 0xFF;
}

pub const POSITION_NAMES: [&str; 15] = [
    "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "STC", "WBL", "WBR",
];

pub const POSITION_KEYS: [&str; 15] = [
    "gk", "sw", "dl", "dc", "dr", "dm", "ml", "mc", "mr", "aml", "amc", "amr", "stc", "wbl", "wbr",
];

pub const ATTRIBUTE_KEYS: [&str; 52] = [
    "crossing",
    "dribbling",
    "finishing",
    "heading",
    "long_shots",
    "marking",
    "off_the_ball",
    "passing",
    "penalty_taking",
    "tackling",
    "vision",
    "handling",
    "aerial_reach",
    "command_of_area",
    "communication",
    "kicking",
    "throwing",
    "anticipation",
    "decisions",
    "one_on_ones",
    "positioning",
    "reflexes",
    "first_touch",
    "technique",
    "flair",
    "corners",
    "teamwork",
    "work_rate",
    "long_throws",
    "eccentricity",
    "rushing_out",
    "punching",
    "acceleration",
    "free_kick_taking",
    "strength",
    "stamina",
    "pace",
    "jumping_reach",
    "leadership",
    "dirtiness",
    "balance",
    "bravery",
    "consistency",
    "aggression",
    "agility",
    "important_matches",
    "injury_proneness",
    "versatility",
    "natural_fitness",
    "determination",
    "composure",
    "concentration",
];

pub const HIDDEN_ATTRIBUTE_KEYS: [&str; 5] = [
    "dirtiness",
    "consistency",
    "important_matches",
    "injury_proneness",
    "versatility",
];

pub fn is_hidden_attribute_key(key: &str) -> bool {
    HIDDEN_ATTRIBUTE_KEYS.contains(&key)
}

/// Attributes that do not consume the CA budget. Determination, natural fitness,
/// flair and aggression stayed unchanged in controlled CA rebalancing tests;
/// hidden attributes are exempt too.
pub fn is_ca_exempt_attribute(key: &str) -> bool {
    is_hidden_attribute_key(key)
        || matches!(
            key,
            "determination" | "natural_fitness" | "flair" | "aggression"
        )
}

pub const PERSONALITY_KEYS: [&str; 8] = [
    "adaptability",
    "ambition",
    "loyalty",
    "pressure",
    "professionalism",
    "sportsmanship",
    "temperament",
    "controversy",
];

/// Bit index -> the player trait the game stores in that bit.
///
/// The first 48 entries follow the game's own trait order. The remaining entries
/// were matched against the save's player data (positions and attributes) and
/// the FM27 localisation table. Bit 53 currently has no trait name.
pub const TRAIT_NAMES: [&str; 64] = [
    "Runs With Ball Down Left",
    "Runs With Ball Down Right",
    "Runs With Ball Through Centre",
    "Gets Into Opposition Area",
    "Moves Into Channels",
    "Gets Forward Whenever Possible",
    "Plays Short Simple Passes",
    "Tries Killer Balls Often",
    "Shoots From Distance",
    "Shoots With Power",
    "Places Shots",
    "Curls Ball",
    "Likes To Round Keeper",
    "Likes To Try To Beat Offside Trap",
    "Uses Outside Of Foot",
    "Marks Opponent Tightly",
    "Winds Up Opponents",
    "Argues With Officials",
    "Plays With Back To Goal",
    "Comes Deep To Get Ball",
    "Plays One-Twos",
    "Likes To Lob Keeper",
    "Dictates Tempo",
    "Attempts Overhead Kicks",
    "Looks For Pass Rather Than Attempting To Score",
    "Plays No Through Balls",
    "Stops Play",
    "Knocks Ball Past Opponent",
    "Moves Ball To Right Foot Before Dribble Attempt",
    "Moves Ball To Left Foot Before Dribble Attempt",
    "Dwells On Ball",
    "Arrives Late In Opponents' Area",
    "Tries To Play Way Out Of Trouble",
    "Stays Back At All Times",
    "Avoids Using Weaker Foot",
    "Tries Tricks",
    "Tries Long Range Free Kicks",
    "Dives Into Tackles",
    "Does Not Dive Into Tackles",
    "Cuts Inside From Both Wings",
    "Hugs Line",
    "Gets Crowd Going",
    "Tries First Time Shots",
    "Tries Long Range Passes",
    "Likes Ball Played Into Feet",
    "Hits Free Kicks With Power",
    "Likes To Beat Opponent Repeatedly",
    "Likes To Switch Ball To Wide Areas",
    "Uses Long Throw To Start Counter Attacks",
    "Play Football For as Long as Possible",
    "Possesses Long Flat Throw",
    "Runs With Ball Often",
    "Runs With Ball Rarely",
    "",
    "Does not Move Into Channels",
    "Plays Ball With Feet",
    "Refrains From Taking Long Shots",
    "Cuts Inside From Left Wing",
    "Cuts Inside From Right Wing",
    "Crosses Early",
    "Brings Ball Out of Defence",
    "Demands to Take Free Kicks",
    "Demands to Take Penalties",
    "Will Retire at Top",
];

pub fn trait_name(bit: u8) -> Option<&'static str> {
    TRAIT_NAMES
        .get(usize::from(bit))
        .copied()
        .filter(|name| !name.is_empty())
}

pub fn scale_attribute(raw: u8) -> u8 {
    ((u16::from(raw) + 2) / 5).max(1) as u8
}

/// Map an attribute key to its byte index inside the stored 54-byte player slice.
/// The two foot bytes sit at indices 24 and 25, between `technique` and `flair`.
pub fn stored_attribute_index(key: &str) -> Option<usize> {
    ATTRIBUTE_KEYS
        .iter()
        .position(|name| *name == key)
        .map(|index| {
            if index < PlayerLayout::LEFT_FOOT_INDEX {
                index
            } else {
                index + 2
            }
        })
}
