pub mod deposit;
pub mod withdraw;

use crate::*;
use anchor_spl::{token::Token, token_2022::Token2022, token_interface::{Mint, TokenAccount}};

#[event_cpi]
#[derive(Accounts)]
pub struct Liquidity<'info> {
    #[account(mut, has_one = base_mint @ PumpAmmError::InvalidBaseMint, has_one = quote_mint @ PumpAmmError::InvalidQuoteMint, has_one = lp_mint @ PumpAmmError::InvalidLpMint, has_one = pool_base_token_account @ PumpAmmError::InvalidPoolBaseTokenAccount, has_one = pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount)]
    pub pool: Box<Account<'info, Pool>>,
    pub global_config: Box<Account<'info, GlobalConfig>>,
    pub user: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub user_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::authority = user)]
    pub user_pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub pool_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}
