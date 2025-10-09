use anchor_lang::prelude::*;

pub const MAX_REASON_LENGTH: usize = 1024;
const MAX_ALLOWED_PROGRAMS: usize = 10;

/// Global configuration for the validator blacklist program
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,                     // 32 bytes - admin who can update config
    pub min_tvl: u64,                      // 8 bytes - minimum total value locked required
    #[max_len(MAX_ALLOWED_PROGRAMS)]
    pub allowed_programs: Vec<Pubkey>,     // 4 + (32 * 10) bytes - allowed stake pool programs
}

impl Config {
    pub const LEN: usize = 8 + 32 + 8 + 4 + (32 * MAX_ALLOWED_PROGRAMS); // discriminator + admin + min_tvl + vec len + allowed_programs
}

/// State account representing a validator that has votes for blacklisting
#[account]
pub struct Blacklist {
    pub validator_identity_address: Pubkey,    // 32 bytes
    pub tally_add: u64,              // 8 bytes - votes to add to blacklist
    pub tally_remove: u64,           // 8 bytes - votes to remove from blacklist
}

impl Blacklist {
    pub const LEN: usize = 8 + 32 + 8 + 8; // discriminator + validator_identity_address + tally_add + tally_remove
}

/// State account representing delegation from a stake pool manager to another authority
#[account]
pub struct Delegation {
    pub stake_pool: Pubkey,          // 32 bytes - the stake pool address
    pub manager: Pubkey,             // 32 bytes - the manager of the stake pool
    pub delegate: Pubkey,            // 32 bytes - the delegated authority
    pub timestamp: i64,              // 8 bytes - when delegation was created
}

impl Delegation {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8; // discriminator + stake_pool + manager + delegate + timestamp
}

/// Vote record for adding a validator to the blacklist
#[account]
#[derive(InitSpace)]
pub struct VoteAddToBlacklist {
    pub stake_pool : Pubkey,         // 32 bytes - the stake pool who cast this vote
    pub validator_identity_address: Pubkey,   // 32 bytes - the validator being voted on
    #[max_len(MAX_REASON_LENGTH)]
    pub reason: String,              // 4 + up to MAX_REASON_LENGTH bytes - reason for the vote
    pub timestamp: i64,              // 8 bytes - UTC timestamp
    pub slot: u64,                   // 8 bytes - slot when vote was cast
}

impl VoteAddToBlacklist {
    pub const LEN: usize = 8 + 32 + 32 + 4 + MAX_REASON_LENGTH + 8 + 8; // discriminator + operator + validator_identity_address + string len + reason + timestamp + slot
}

/// Vote record for removing a validator from the blacklist
#[account]
#[derive(InitSpace)]
pub struct VoteRemoveFromBlacklist {
    pub stake_pool: Pubkey,          // 32 bytes - the stake pool who cast this vote
    pub validator_identity_address: Pubkey,   // 32 bytes - the validator being voted on
    #[max_len(MAX_REASON_LENGTH)]
    pub reason: String,              // 4 + up to MAX_REASON_LENGTH bytes - reason for the vote
    pub timestamp: i64,              // 8 bytes - UTC timestamp
    pub slot: u64,                   // 8 bytes - slot when vote was cast
}

impl VoteRemoveFromBlacklist {
    pub const LEN: usize = 8 + 32 + 32 + 4 + MAX_REASON_LENGTH + 8 + 8; // discriminator + operator + validator_identity_address + string len + reason + timestamp + slot
}
