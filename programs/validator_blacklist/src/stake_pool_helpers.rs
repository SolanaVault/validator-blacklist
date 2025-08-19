use anchor_lang::{prelude::*, solana_program::borsh1};
use spl_stake_pool::state::StakePool;

use crate::error::ValidatorBlacklistError;

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