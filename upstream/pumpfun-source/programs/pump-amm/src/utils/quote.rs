use anchor_lang::prelude::*;

const USDC: Pubkey = anchor_lang::solana_program::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub fn is_native_or_usdc(mint: &Pubkey) -> bool {
    *mint == anchor_spl::token::spl_token::native_mint::ID
        || *mint == anchor_spl::token_2022::spl_token_2022::native_mint::ID
        || *mint == USDC
}
