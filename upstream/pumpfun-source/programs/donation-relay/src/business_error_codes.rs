//! Program error catalog reconstructed from captured names/messages.
//! Use the authenticated Anchor proc-macro, not a handwritten expansion.
#[anchor_lang::error_code]
pub enum ErrorCode {
    #[msg("Invalid admin")]
    InvalidAdmin = 0,
    #[msg("Invalid owner")]
    InvalidOwner = 1,
    #[msg("Invalid config id")]
    InvalidConfigId = 2,
    #[msg("Invalid bump")]
    InvalidBump = 3,
    #[msg("Invalid debouncer PDA address")]
    InvalidDebouncerPda = 4,
    #[msg("Invalid PDA address")]
    InvalidPda = 5,
    #[msg("Invalid mint")]
    InvalidMint = 6,
    #[msg("Invalid state")]
    InvalidState = 7,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow = 8,
    #[msg("Invalid message length")]
    InvalidMessageLength = 9,
    #[msg("The account is already initialized")]
    AccountInitialized = 10,
    #[msg("Epoch already closed")]
    EpochAlreadyClosed = 11,
    #[msg("Invalid balance")]
    InvalidBalance = 12,
    #[msg("Invalid token account state")]
    InvalidTokenAccountState = 13,
    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner = 14,
    #[msg("Signer is not the relayer")]
    SignerIsNotRelayer = 15,
    #[msg("Transfer fee extension not supported")]
    TransferFeeExtensionNotSupported = 16,
    #[msg("Fair distribution error")]
    FairDistributionError = 17,
    #[msg("Invalid bps")]
    InvalidBps = 18,
    #[msg("Invalid instruction data")]
    InvalidInstructionData = 19,
    #[msg("Invalid instruction data length")]
    InvalidInstructionDataLength = 20,
    #[msg("Input amount mismatch")]
    InputAmountMismatch = 21,
    #[msg("Bps sum does not equal ten thousand")]
    BpsSumDoesNotEqualTenThousand = 22,
    #[msg("Output amount must be greater than zero")]
    OutputAmountMustBeGreaterThanZero = 23,
    #[msg("Invalid token account mint")]
    InvalidTokenAccountMint = 24,
    #[msg("Invalid token account program")]
    InvalidTokenAccountProgram = 25,
    #[msg("Debouncer not ready to distribute")]
    DebouncerNotReadyToDistribute = 26,
    #[msg("Invalid epoch")]
    InvalidEpoch = 27,
    #[msg("Epoch not closed - cannot distribute")]
    EpochNotClosed = 28,
    #[msg("Invalid token program")]
    InvalidTokenProgram = 29,
    #[msg("Insufficient remaining accounts")]
    InsufficientRemainingAccounts = 30,
    #[msg("Mint not safe for donation")]
    MintNotSafeForDonation = 31,
    #[msg("Mint whitelist account invalid")]
    InvalidMintWhitelist = 32,
    #[msg("Swap receipt does not match this distribution")]
    SwapReceiptMismatch = 33,
}
