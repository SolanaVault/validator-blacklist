use anchor_lang::prelude::*;
use crate::authority_checks;
use crate::stake_pool_helpers::deserialize_stake_pool_with_checks;
use crate::state::{Blacklist, Delegation, VoteRemoveFromBlacklist, MAX_REASON_LENGTH};
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
    /// The stake pool account to validate the authority
    /// CHECK: We manually validate this is a valid stake pool in the instruction logic
    #[account()]
    pub stake_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"blacklist", validator_identity_address.as_ref()],
        bump
    )]
    pub blacklist: Account<'info, Blacklist>,

    #[account(
        init,
        payer = authority,
        space = VoteRemoveFromBlacklist::LEN,
        seeds = [b"vote_remove", stake_pool.key().as_ref(), validator_identity_address.as_ref()],
        bump
    )]
    pub vote_remove: Account<'info, VoteRemoveFromBlacklist>,

    /// Optional delegation account - if present, authority must be the delegate
    pub delegation: Option<Account<'info, Delegation>>,

    /// The authority (either manager or delegated authority)
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
