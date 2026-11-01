//! External AMM codecs recovered from original Mayhem decoding routines.
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
pub const AMM_ID:Pubkey=pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
// 0x17d8 computes current serialized size 287 from the default struct, allocates a
// ZERO-filled 287-byte buffer for every shorter input, copies ALL available input
// bytes, then calls 0x8eb78. No version-boundary truncation is used in Mayhem.
pub type SwapPool=curve_accounts::AccountWrapper<curve_accounts::Pool>;






































// 0x744a8 checks discriminator; 0x74890 consumes exactly the old418-byte prefix.
// Unlike Pool this has no padding and no optional historical layout selector.
pub use crate::pump_amm::accounts::GlobalConfig as SwapGlobalConfig;





























// Keep the published module path while using the shared error namespace.
pub use crate::MayhemError as PumpSwapError;
