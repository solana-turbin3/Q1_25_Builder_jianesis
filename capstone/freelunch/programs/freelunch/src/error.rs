use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient unlockable funds.")]
    InsufficientFunds,

    #[msg("Invalid merchant account.")]
    InvalidMerchant,

    #[msg("Unauthorized operation.")]
    Unauthorized,

    #[msg("Insufficient staked amount.")]
    InsufficientStake,

    #[msg("Payment already completed.")]
    PaymentAlreadyCompleted,

    #[msg("Invalid proof of future payment.")]
    InvalidProofOfPayment,

    #[msg("Invalid Purchase Amount.")]
    InvalidPurchaseAmount,

    #[msg("Invalid APY.")]
    InvalidAPY,
}
