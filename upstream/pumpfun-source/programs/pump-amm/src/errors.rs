use anchor_lang::prelude::*;

#[error_code]
pub enum PumpAmmError {
    FeeBasisPointsExceedsMaximum = 0,
    ZeroBaseAmount = 1,
    ZeroQuoteAmount = 2,
    TooLittlePoolTokenLiquidity = 3,
    ExceededSlippage = 4,
    InvalidAdmin = 5,
    UnsupportedBaseMint = 6,
    UnsupportedQuoteMint = 7,
    InvalidBaseMint = 8,
    InvalidQuoteMint = 9,
    InvalidLpMint = 10,
    AllProtocolFeeRecipientsShouldBeNonZero = 11,
    UnsortedNotUniqueProtocolFeeRecipients = 12,
    InvalidProtocolFeeRecipient = 13,
    InvalidPoolBaseTokenAccount = 14,
    InvalidPoolQuoteTokenAccount = 15,
    BuyMoreBaseAmountThanPoolReserves = 16,
    DisabledCreatePool = 17,
    DisabledDeposit = 18,
    DisabledWithdraw = 19,
    DisabledBuy = 20,
    DisabledSell = 21,
    SameMint = 22,
    Overflow = 23,
    Truncation = 24,
    DivisionByZero = 25,
    NewSizeLessThanCurrentSize = 26,
    AccountTypeNotSupported = 27,
    OnlyCanonicalPumpPoolsCanHaveCoinCreator = 28,
    InvalidAdminSetCoinCreatorAuthority = 29,
    StartTimeInThePast = 30,
    EndTimeInThePast = 31,
    EndTimeBeforeStartTime = 32,
    TimeRangeTooLarge = 33,
    EndTimeBeforeCurrentDay = 34,
    SupplyUpdateForFinishedRange = 35,
    DayIndexAfterEndIndex = 36,
    DayInActiveRange = 37,
    InvalidIncentiveMint = 38,
    #[msg("buy: Not enough quote tokens to cover for fees.")]
    BuyNotEnoughQuoteTokensToCoverFees = 39,
    #[msg("buy: slippage - would buy less tokens than expected min_base_amount_out")]
    BuySlippageBelowMinBaseAmountOut = 40,
    MayhemModeDisabled = 41,
    OnlyPumpPoolsMayhemMode = 42,
    MayhemModeInDesiredState = 43,
    NotEnoughRemainingAccounts = 44,
    InvalidSharingConfigBaseMint = 45,
    InvalidSharingConfigCoinCreator = 46,
    #[msg("coin creator has been migrated to sharing config")]
    CoinCreatorMigratedToSharingConfig = 47,
    #[msg("creator_vault has been migrated to sharing config, use pump:distribute_creator_fees instead")]
    CreatorVaultMigratedToSharingConfig = 48,
    #[msg("Cashback is disabled")]
    CashbackNotEnabled = 49,
    OnlyPumpPoolsCashback = 50,
    CashbackNotInDesiredState = 51,
    TokensInVaultLessThanCashbackEarned = 52,
    #[msg("Buyback fee recipient not authorized")]
    BuybackFeeRecipientNotAuthorized = 53,
    AllBuybackFeeRecipientsShouldBeNonZero = 54,
    NotUniqueBuybackFeeRecipients = 55,
    #[msg("buyback_basis_points must be <= 10_000")]
    BuybackBasisPointsOutOfRange = 56,
    #[msg("buyback fee recipients require exactly 8 remaining accounts (or none)")]
    WrongBuybackFeeRecipientsCount = 57,
    BuybackFeeRecipientMissing = 58,
    #[msg("Cashback trade is missing the required remaining accounts")]
    MissingCashbackAccounts = 59,
    #[msg("Cashback user_volume_accumulator account is invalid")]
    InvalidCashbackAccumulator = 60,
    #[msg("Cashback user_volume_accumulator ATA is missing or invalid")]
    InvalidCashbackAccumulatorAta = 61,
    #[msg("pool_v2 remaining account is missing or invalid")]
    InvalidPoolV2 = 62,
    #[msg("BOOST: sell output exceeds the real quote vault. effective = real + virtual is pricing-only; payout is capped at real_vault, so quote min(out, real_vault)")]
    InsufficientRealQuoteReserves = 63,
    #[msg("BOOST: deposit/withdraw don't apply to boost pools")]
    BoostPoolLiquidityUnsupported = 64,
    #[msg("BOOST: pool cannot be boosted (no virtual reserves)")]
    PoolCannotBoost = 65,
    #[msg("BOOST: boost is disabled")]
    BoostDisabled = 66,
    #[msg("BOOST: lp_supply must never drop below the circulating LP mint supply")]
    SeedLockViolation = 67,
    #[msg("Configurable creator fee is disabled")]
    CreatorFeeNotConfigurable = 68,
    #[msg("Creator fee basis points must be between 1 and the configured maximum")]
    CreatorFeeBpsOutOfRange = 69,
    #[msg("Creator fee is not editable for this pool")]
    CreatorFeeNotEditable = 70,
    #[msg("Cashback coins cannot have a creator fee")]
    CreatorFeeNotAllowedForCashbackCoin = 71,
    #[msg("Sharing config is not active")]
    SharingConfigNotActive = 72,
    #[msg("Not authorized")]
    NotAuthorized = 73,
    #[msg("The coin creator of a holder-reward pool cannot be changed")]
    HolderRewardCreatorImmutable = 74,
    #[msg("CTO is not allowed on a mayhem-mode pool")]
    CtoNotAllowedForMayhemPool = 75,
    #[msg("A holder-reward pool's coin creator must be the holder-rewards PDA")]
    InvalidHolderRewardCoinCreator = 76,
    #[msg("Creator fee is not configurable on a SOL or USDC quote; the fee schedule applies")]
    CreatorFeeNotConfigurableForQuote = 77,
    #[msg("Only canonical pump pools are supported by this instruction")]
    OnlyPumpPools = 78,
    #[msg("v2 trades do not support cashback coins; use the v1 instructions")]
    CashbackCoinNotSupported = 79,
    #[msg("v2 trades do not support mayhem-mode pools; use the v1 instructions")]
    MayhemPoolNotSupported = 80,
    #[msg("creator_fees must be swept before the coin creator changes")]
    CreatorFeesNotSwept = 81,
    #[msg("The fee config tier table is empty")]
    FeeTiersEmpty = 82,
    #[msg("The fee config account is shorter than its initialize size")]
    FeeConfigTooShort = 83,
    #[msg("multi-hop: a hop's input mint is neither side of its pool or curve, or the route does not end in user_out_token_account's mint")]
    MultiHopDiscontinuousPath = 84,
    #[msg("multi-hop: the balances around the bonding-curve hops do not match the amounts pump reported")]
    MultiHopCurveRunMismatch = 85,
    #[msg("multi-hop: every hop must trade in the same direction, all buys or all sells")]
    MultiHopMixedDirection = 86,
    #[msg("A transfer's source is its destination and the signer does not own it")]
    SelfTransferNotAllowed = 87,
    #[msg("quote_mint must be writable for a quote with a Token-2022 transfer fee: its withheld fees are harvested to the mint")]
    QuoteMintNotWritable = 88,
}
