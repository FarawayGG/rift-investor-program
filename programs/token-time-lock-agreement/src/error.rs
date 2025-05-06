use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid hold duration")]
    InvalidHoldDuration,
    #[msg("Invalid remaining accounts")]
    InvalidRemainingAccounts,
    #[msg("Invalid investor account")]
    InvalidInvestorAccount,
    #[msg("Investor already exists")]
    InvestorAlreadyExists,
    #[msg("Invalid agreement account")]
    AlreadyDeposited,
    #[msg("Deposit exceeds expected payment")]
    DepositExceedsExpectedPayment,
    #[msg("Seller already deposited")]
    SellerDeposited,
    #[msg("Total allocations exceed target")]
    TotalAllocationsExceedTarget,
    #[msg("Total token allocations exceed target")]
    TotalTokenAllocationsExceedTarget,
    #[msg("Seller must deposit first")]
    SellerMustDepositFirst,
    #[msg("Seller cannot deposit stablecoins")]
    SellerCannotDepositStablecoins,
    #[msg("Incorrect deposit amount")]
    IncorrectDepositAmount,
    #[msg("Invalid destination")]
    InvalidDestination,
    #[msg("Only seller allowed")]
    OnlySellerAllowed,
    #[msg("Agreement already cancelled")]
    AgreementAlreadyCancelled,
    #[msg("Tokens already deposited")]
    TokensAlreadyDeposited,
    #[msg("Full amount not collected")]
    FullAmountNotCollected,
    #[msg("Fund commission already collected")]
    FundCommissionAlreadyCollected,
    #[msg("Tokens not deposited")]
    TokensNotDeposited,
    #[msg("Hold duration not started")]
    HoldDurationNotStarted,
    #[msg("Hold duration period not expired")]
    HoldDurationPeriodNotExpired,
    #[msg("Tokens already withdrawn")]
    TokensAlreadyWithdrawn,
    #[msg("Not an investor")]
    NotAnInvestor,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("No first deposit")]
    NoFirstDeposit,
    #[msg("Cancellation timeout not reached")]
    CancellationTimeoutNotReached,
    #[msg("Agreement not cancelled")]
    AgreementNotCancelled,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("No funds to withdraw")]
    NoFundsToWithdraw,
    #[msg("Invalid token account")]
    InsufficientTokenBalance,
    #[msg("Duplicate investor")]
    DuplicateInvestor,
    #[msg("Invalid investor account owner")]
    InvalidInvestorAccountOwner,
}
