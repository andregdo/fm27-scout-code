//! FM27 staff objects and their profile attributes, role suitability and job
//! codes.

/// FM27 staff objects. The header is a person id followed by the doubled uid, the
/// object-kind byte and the entry count; the ability block sits after the entries and
/// three reputation words. Profile attributes are split between the preference area
/// and the 26-byte block at the end of the ability block.
#[derive(Debug, Clone, Copy)]
pub struct StaffLayout;

impl StaffLayout {
    pub const KIND_OFFSET: usize = 12;
    pub const STAFF_KIND: u8 = 1;
    pub const HUMAN_KIND: u8 = 9;
    pub const UID_OFFSET: usize = 4;
    pub const UID_COPY_OFFSET: usize = 8;
    pub const ENTRY_COUNT_OFFSET: usize = 13;
    pub const ENTRY_BYTES: usize = 7;
    pub const ABILITY_BASE_OFFSET: usize = 21;
    pub const CURRENT_ABILITY_OFFSET: usize = 0;
    pub const POTENTIAL_ABILITY_OFFSET: usize = 2;
    /// The ability block's r4 byte. It is not the displayed job.
    pub const ABILITY_R4_OFFSET: usize = 4;
    pub const SENTINEL_OFFSET: usize = 13;
    pub const SENTINEL_VALUE: u8 = 12;
    pub const PREFERENCES_OFFSET: usize = 14;
    pub const PREFERENCE_COUNT: usize = 26;
    pub const PREFERENCE_AUTHORITY_OFFSET: usize = 4;
    pub const PREFERENCE_YOUNGSTERS_OFFSET: usize = 12;
    pub const PREFERENCE_DETERMINATION_OFFSET: usize = 13;
    pub const BLOCK_40_OFFSET: usize = 40;
    pub const BLOCK_40_COUNT: usize = 26;
    /// (attribute key index, block-40 slot) for the attributes outside the
    /// preference area. Determination, authority and working with youngsters are
    /// read from the preference area instead.
    pub const BLOCK_40_ATTRIBUTE_SLOTS: [(usize, usize); 18] = [
        (3, 1),
        (4, 2),
        (5, 3),
        (6, 4),
        (7, 5),
        (8, 6),
        (9, 7),
        (10, 8),
        (11, 9),
        (12, 10),
        (13, 11),
        (14, 12),
        (15, 13),
        (16, 18),
        (17, 21),
        (18, 23),
        (19, 24),
        (20, 25),
    ];
    pub const ABILITY_RANGE: (u16, u16) = (1, 200);
    pub const BLOCK_40_RANGE: (u8, u8) = (1, 100);
    pub const ALLOCATED_UID_MINIMUM: u32 = 2_000_000_000;
    /// FM27 reads the job code this many bytes past a contract record's selector.
    pub const JOB_OFFSET_FROM_SELECTOR: usize = 16;
    /// A contract offer's second tag follows the selector by this many bytes.
    pub const OFFER_MARKER_OFFSET: usize = 19;
    pub const CONTRACT_SEARCH_BYTES: usize = 32_768;
    /// A contracted person's name block starts this many bytes past his latest tag.
    pub const PERSON_BLOCK_OFFSET_AFTER_TAG: usize = 60;
    pub const PERSON_SEARCH_BYTES: usize = 4_096;
    pub const LIST_PERSON_SEARCH_BYTES: usize = 20_000;
    /// A person who holds a club role with no contract has a tag, a 05 05 lead,
    /// his selector, his team id and then his job code.
    pub const NO_CONTRACT_SELECTOR_OFFSET: usize = 16;
    pub const NO_CONTRACT_LEAD: [u8; 2] = [0x05, 0x05];
    pub const NO_CONTRACT_TEAM_OFFSET: usize = 20;
    pub const NO_CONTRACT_JOB_OFFSET: usize = 32;
    /// Fifteen role-suitability bytes follow a 0x01 byte and precede four zero bytes.
    pub const ROLE_VECTOR_BYTES: usize = 15;
    pub const ROLE_VECTOR_PRECEDING: u8 = 1;
    pub const ROLE_SCORE_MIN: u8 = 1;
    pub const ROLE_SCORE_MAX: u8 = 20;
    pub const ROLE_GREEN_THRESHOLD: u8 = 10;
}

pub const STAFF_ATTRIBUTE_KEYS: [&str; 21] = [
    "authority",
    "working_with_youngsters",
    "determination",
    "goalkeeping",
    "judging_player_ability",
    "judging_player_potential",
    "people_management",
    "motivating",
    "physiotherapy",
    "tactical_knowledge",
    "attacking",
    "defending",
    "fitness",
    "possession",
    "technical",
    "tactical",
    "data_analysis",
    "sports_science",
    "negotiating",
    "judging_staff_ability",
    "set_pieces",
];

pub const STAFF_ROLE_KEYS: [&str; 15] = [
    "manager",
    "assistant_manager",
    "coach",
    "physio",
    "scout",
    "gk_coach",
    "fitness_coach",
    "chairperson",
    "director_of_football",
    "head_of_youth_development",
    "loan_manager",
    "technical_director",
    "data_analyst",
    "sports_scientist",
    "set_piece_coach",
];

pub const STAFF_ROLE_NAMES: [&str; 15] = [
    "Manager",
    "Assistant Manager",
    "Coach",
    "Physio",
    "Scout",
    "Goalkeeping Coach",
    "Fitness Coach",
    "Chairperson",
    "Director of Football",
    "Head of Youth Development",
    "Loan Manager",
    "Technical Director",
    "Data Analyst",
    "Sports Scientist",
    "Set Piece Coach",
];

/// The job codes FM27 labels in the staff screen. Code 24 is the General Manager
/// chairman-title variant, distinct from 8 Managing Director. Unconfirmed codes
/// return `None` instead of inventing a title.
pub fn staff_job_name(code: u8) -> Option<&'static str> {
    Some(match code {
        2 => "Coach",
        4 => "Chairperson",
        6 => "Director",
        8 => "Managing Director",
        10 => "Director of Football",
        12 => "Physio",
        14 => "Scout",
        16 => "Manager",
        20 => "Assistant Manager",
        22 => "Set Piece Coach",
        24 => "General Manager",
        26 => "Fitness Coach",
        34 => "Goalkeeping Coach",
        38 => "Chief Doctor",
        40 => "Head of Sports Science",
        44 => "Chief Scout",
        46 => "Doctor",
        48 => "Sports Scientist",
        50 => "Head Physio",
        58 => "Data Analyst",
        60 => "Performance Analyst",
        62 => "Data Analyst",
        64 => "Head of Youth Development",
        66 => "Owner",
        70 => "President",
        86 => "Loan Manager",
        88 => "Technical Director",
        _ => return None,
    })
}

pub fn staff_role_name(index: u8) -> Option<&'static str> {
    STAFF_ROLE_NAMES.get(usize::from(index)).copied()
}
