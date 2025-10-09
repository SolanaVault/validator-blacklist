use anchor_lang::prelude::*;
use crate::authority_checks;
use crate::stake_pool_helpers::{deserialize_stake_pool_with_checks, validate_stake_pool_config};
use crate::state::{Blacklist, Delegation, VoteRemoveFromBlacklist, Config, MAX_REASON_LENGTH};
use crate::error::ValidatorBlacklistError;

/// Vote to remove a validator from the blacklist
pub fn vote_remove(
    ctx: Context<VoteRemove>,
    validator_identity_address: Pubkey,
    reason: String,
) -> Result<()> {

    require!(
        reason.len() <= MAX_REASON_LENGTH,
        ValidatorBlacklistError::ReasonTooLong
    );
        
    let blacklist = &mut ctx.accounts.blacklist;
    let vote_remove = &mut ctx.accounts.vote_remove;
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

    // Create the vote record
    vote_remove.stake_pool = ctx.accounts.stake_pool.key();
    vote_remove.validator_identity_address = validator_identity_address;
    vote_remove.reason = reason;
    vote_remove.timestamp = clock.unix_timestamp;
    vote_remove.slot = clock.slot;

    // Update the tally
    blacklist.tally_remove = blacklist.tally_remove.checked_add(1)
        .ok_or(ValidatorBlacklistError::MathOverflow)?;

    msg!("Vote to remove validator {} from blacklist cast by stake pool {}", 
         validator_identity_address, ctx.accounts.stake_pool.key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(validator_identity_address: Pubkey, reason: String)]
pub struct VoteRemove<'info> {
    /// Global configuration account
    #[account()]
    pub config: Account<'info, Config>,

    /// The stake pool account to validate the authority
    /// CHECK: We manually validate this is a valid stake pool in the instruction logic
    #[account()]
    pub stake_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"blacklist", config.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub blacklist: Account<'info, Blacklist>,

    #[account(
        init,
        payer = authority,
        space = VoteRemoveFromBlacklist::LEN,
        seeds = [b"vote_remove", config.key().as_ref(), stake_pool.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub vote_remove: Account<'info, VoteRemoveFromBlacklist>,

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
