use anchor_lang::prelude::*;

#[error_code]
pub enum ValidatorBlacklistError {
    #[msg("The signer is not the manager of the stake pool")]
    InvalidManager,
    #[msg("Invalid stake pool for this delegation")]
    InvalidStakePool,
    #[msg("Invalid delegate for this delegation")]
    InvalidDelegate,
    #[msg("Unauthorized signer - must be manager or valid delegate")]
    UnauthorizedSigner,
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("Math underflow occurred")]
    MathUnderflow,
    #[msg("The reason field exceeds the maximum allowed length")]
    ReasonTooLong,
}
