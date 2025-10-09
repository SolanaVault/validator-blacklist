use anchor_lang::prelude::*;
use crate::authority_checks;
use crate::stake_pool_helpers::{deserialize_stake_pool_with_checks, validate_stake_pool_config};
use crate::state::{Blacklist, Delegation, VoteAddToBlacklist, Config, MAX_REASON_LENGTH};
use crate::error::ValidatorBlacklistError;

/// Vote to add a validator to the blacklist
pub fn vote_add(
    ctx: Context<VoteAdd>,
    validator_identity_address: Pubkey,
    reason: String,
) -> Result<()> {

    require!(
        reason.len() <= MAX_REASON_LENGTH,
        ValidatorBlacklistError::ReasonTooLong
    );

    let blacklist = &mut ctx.accounts.blacklist;
    let vote_add = &mut ctx.accounts.vote_add;
    let clock = Clock::get()?;

    let stake_pool = deserialize_stake_pool_with_checks(&ctx.accounts.stake_pool.try_borrow_data()?)?;

    // Validate stake pool meets config requirements
    validate_stake_pool_config(
        &stake_pool,
        &ctx.accounts.stake_pool.owner,
        &ctx.accounts.config,
    )?;

    // Validate the authority
    authority_checks::check_authority(
        ctx.accounts.delegation.as_deref(), 
        &ctx.accounts.stake_pool.key(), 
        &stake_pool,
        &ctx.accounts.authority.key())?;
    

    // Initialize blacklist if this is the first vote for this validator
    if blacklist.validator_identity_address == Pubkey::default() {
        blacklist.validator_identity_address = validator_identity_address;
        blacklist.tally_add = 0;
        blacklist.tally_remove = 0;
    }

    // Create the vote record
    vote_add.stake_pool = ctx.accounts.stake_pool.key();
    vote_add.validator_identity_address = validator_identity_address;
    vote_add.reason = reason;
    vote_add.timestamp = clock.unix_timestamp;
    vote_add.slot = clock.slot;

    // Update the tally
    blacklist.tally_add = blacklist.tally_add.checked_add(1)
        .ok_or(ValidatorBlacklistError::MathOverflow)?;

    msg!("Vote to add validator {} to blacklist cast by stake pool {}", 
         validator_identity_address, ctx.accounts.stake_pool.key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(validator_identity_address: Pubkey, reason: String)]
pub struct VoteAdd<'info> {
    /// Global configuration account
    #[account()]
    pub config: Account<'info, Config>,

    /// The stake pool account for stake pool that is casting the vote
    /// CHECK: We manually validate this is a valid stake pool in the instruction logic
    #[account()]
    pub stake_pool: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        space = Blacklist::LEN,
        seeds = [b"blacklist", config.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub blacklist: Account<'info, Blacklist>,

    #[account(
        init,
        payer = authority,
        space = VoteAddToBlacklist::LEN,
        seeds = [b"vote_add", config.key().as_ref(), stake_pool.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub vote_add: Account<'info, VoteAddToBlacklist>,

    /// Optional delegation account - if present, authority must be the delegate
    #[account(
        seeds = [b"delegation", config.key().as_ref(), stake_pool.key().as_ref()],
        bump
    )]
    pub delegation: Option<Account<'info, Delegation>>,

    /// The authority (either manager or delegated authority)
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
