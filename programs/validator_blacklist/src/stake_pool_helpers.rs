use anchor_lang::{prelude::*, solana_program::borsh1};
use spl_stake_pool::state::StakePool;

use crate::error::ValidatorBlacklistError;
use crate::state::Config;

pub fn deserialize_stake_pool_with_checks(stake_pool_data: &[u8]) -> Result<StakePool> {
    
    require_eq!(
        stake_pool_data.len(),
        borsh1::get_packed_len::<StakePool>(),
        ValidatorBlacklistError::InvalidStakePool
    );

    let stake_pool: StakePool = borsh1::try_from_slice_unchecked(&stake_pool_data)
        .map_err(|_| ValidatorBlacklistError::InvalidStakePool)?;
    
    Ok(stake_pool)
}

pub fn validate_stake_pool_config(
    stake_pool: &StakePool,
    stake_pool_owner: &Pubkey,
    config: &Config,
) -> Result<()> {
    // Check minimum TVL requirement
    require!(
        stake_pool.total_lamports >= config.min_tvl,
        ValidatorBlacklistError::InsufficientTvl
    );

    // Check if the stake pool owner is in the allowed programs list
    require!(
        config.allowed_programs.contains(stake_pool_owner),
        ValidatorBlacklistError::UnauthorizedStakePoolProgram
    );

    Ok(())
}
