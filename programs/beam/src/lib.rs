//! Experimental Beam launch program. The fee balances remain in each launch
//! account until a separately reviewed rewards/treasury distribution path is
//! implemented. No migration instruction is exposed yet.
// Anchor 0.31's generated code still emits these on Rust 1.99.
#![allow(unexpected_cfgs, deprecated)]

use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer as NativeTransfer};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, SetAuthority, Token, TokenAccount, TransferChecked};
use beam_core::{BondingCurve, FeePolicy, LaunchConfig, Trade};

declare_id!("5ZMhw2LcGtMCy7NKhmL8SKLw5Sshgj4gRjn1H9ta9kYf");

#[program]
pub mod beam_launch {
    use super::*;

    /// Mint a fixed-supply SPL token and place its entire supply in the curve
    /// vault. The unsold pool reserve stays there until a future migration.
    pub fn create(ctx: Context<Create>, args: CreateArgs) -> Result<()> {
        let curve = BondingCurve::new(LaunchConfig {
            total_supply: args.total_supply,
            initial_real_token_reserves: args.initial_real_token_reserves,
            initial_virtual_token_reserves: args.initial_virtual_token_reserves,
            initial_virtual_quote_reserves: args.initial_virtual_quote_reserves,
            migration_fee: 0,
        })
        .map_err(map_core_error)?;

        let launch = &mut ctx.accounts.launch;
        launch.version = 1;
        launch.bump = ctx.bumps.launch;
        launch.creator = ctx.accounts.creator.key();
        launch.mint = ctx.accounts.mint.key();
        launch.set_curve(curve);

        let mint_key = ctx.accounts.mint.key();
        let bump = [ctx.bumps.launch];
        let signer_seeds: &[&[u8]] = &[b"launch", mint_key.as_ref(), &bump];
        let signers = &[signer_seeds];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: launch.to_account_info(),
                },
                signers,
            ),
            args.total_supply,
        )?;
        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SetAuthority {
                    current_authority: launch.to_account_info(),
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                },
                signers,
            ),
            anchor_spl::token::spl_token::instruction::AuthorityType::MintTokens,
            None,
        )?;

        emit!(LaunchCreated {
            mint: mint_key,
            creator: launch.creator,
            total_supply: args.total_supply,
            real_token_reserves: curve.real_token_reserves,
            reserved_pool_tokens: curve.reserved_pool_tokens,
        });
        Ok(())
    }

    /// Buy an exact quantity of tokens. The account receives the gross quote
    /// and all fee allocations; external fee claims are intentionally absent.
    pub fn buy(ctx: Context<Buy>, tokens: u64, max_lamports: u64) -> Result<()> {
        let launch = &mut ctx.accounts.launch;
        let mut curve = launch.curve();
        let expected_vault_balance = curve
            .real_token_reserves
            .checked_add(curve.reserved_pool_tokens)
            .ok_or(BeamError::Overflow)?;
        require_gte!(
            ctx.accounts.vault.amount,
            expected_vault_balance,
            BeamError::VaultBalance
        );
        let trade = curve
            .buy(tokens, max_lamports, FeePolicy::default())
            .map_err(map_core_error)?;
        launch.check_accrual(&trade)?;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                NativeTransfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: launch.to_account_info(),
                },
            ),
            trade.trader_quote,
        )?;

        let mint_key = ctx.accounts.mint.key();
        let bump = [launch.bump];
        let signer_seeds: &[&[u8]] = &[b"launch", mint_key.as_ref(), &bump];
        token::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.buyer_tokens.to_account_info(),
                    authority: launch.to_account_info(),
                },
                &[signer_seeds],
            ),
            tokens,
            ctx.accounts.mint.decimals,
        )?;

        launch.accrue(&trade)?;
        launch.set_curve(curve);
        emit_trade(
            ctx.accounts.buyer.key(),
            mint_key,
            true,
            trade,
            curve.complete,
        );
        Ok(())
    }

    /// Sell tokens back while the curve is active. Only the net quote leaves
    /// the launch account; the fee stays in its separately tracked balance.
    pub fn sell(ctx: Context<Sell>, tokens: u64, min_lamports: u64) -> Result<()> {
        let launch = &mut ctx.accounts.launch;
        let mut curve = launch.curve();
        let trade = curve
            .sell(tokens, min_lamports, FeePolicy::default())
            .map_err(map_core_error)?;
        launch.check_accrual(&trade)?;
        let rent_floor = Rent::get()?.minimum_balance(8 + Launch::INIT_SPACE);
        require_gte!(
            launch.to_account_info().lamports(),
            rent_floor
                .checked_add(trade.trader_quote)
                .ok_or(BeamError::Overflow)?,
            BeamError::InsufficientEscrow
        );

        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.seller_tokens.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            tokens,
            ctx.accounts.mint.decimals,
        )?;

        launch.to_account_info().sub_lamports(trade.trader_quote)?;
        ctx.accounts
            .seller
            .to_account_info()
            .add_lamports(trade.trader_quote)?;
        launch.accrue(&trade)?;
        launch.set_curve(curve);
        emit_trade(
            ctx.accounts.seller.key(),
            ctx.accounts.mint.key(),
            false,
            trade,
            false,
        );
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct CreateArgs {
    /// Raw token units. The created mint has six decimals.
    pub total_supply: u64,
    pub initial_real_token_reserves: u64,
    pub initial_virtual_token_reserves: u64,
    /// Lamports.
    pub initial_virtual_quote_reserves: u64,
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        space = 8 + Launch::INIT_SPACE,
        seeds = [b"launch", mint.key().as_ref()],
        bump
    )]
    pub launch: Account<'info, Launch>,
    #[account(init, payer = creator, mint::decimals = 6, mint::authority = launch)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = creator, associated_token::mint = mint, associated_token::authority = launch)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [b"launch", mint.key().as_ref()], bump = launch.bump, has_one = mint)]
    pub launch: Account<'info, Launch>,
    pub mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = launch)]
    pub vault: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = buyer, associated_token::mint = mint, associated_token::authority = buyer)]
    pub buyer_tokens: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut, seeds = [b"launch", mint.key().as_ref()], bump = launch.bump, has_one = mint)]
    pub launch: Account<'info, Launch>,
    pub mint: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = launch)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = seller)]
    pub seller_tokens: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct Launch {
    pub version: u8,
    pub bump: u8,
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub initial_real_token_reserves: u64,
    pub virtual_token_reserves: u64,
    pub virtual_quote_reserves: u64,
    pub real_token_reserves: u64,
    pub real_quote_reserves: u64,
    pub reserved_pool_tokens: u64,
    pub rewards_accrued: u64,
    pub treasury_accrued: u64,
    pub founder_accrued: u64,
    pub complete: bool,
}

impl Launch {
    fn curve(&self) -> BondingCurve {
        BondingCurve {
            initial_real_token_reserves: self.initial_real_token_reserves,
            virtual_token_reserves: self.virtual_token_reserves,
            virtual_quote_reserves: self.virtual_quote_reserves,
            real_token_reserves: self.real_token_reserves,
            real_quote_reserves: self.real_quote_reserves,
            reserved_pool_tokens: self.reserved_pool_tokens,
            complete: self.complete,
        }
    }

    fn set_curve(&mut self, curve: BondingCurve) {
        self.initial_real_token_reserves = curve.initial_real_token_reserves;
        self.virtual_token_reserves = curve.virtual_token_reserves;
        self.virtual_quote_reserves = curve.virtual_quote_reserves;
        self.real_token_reserves = curve.real_token_reserves;
        self.real_quote_reserves = curve.real_quote_reserves;
        self.reserved_pool_tokens = curve.reserved_pool_tokens;
        self.complete = curve.complete;
    }

    fn check_accrual(&self, trade: &Trade) -> Result<()> {
        self.rewards_accrued
            .checked_add(trade.fees.rewards)
            .ok_or(BeamError::Overflow)?;
        self.treasury_accrued
            .checked_add(trade.fees.treasury)
            .ok_or(BeamError::Overflow)?;
        self.founder_accrued
            .checked_add(trade.fees.founder)
            .ok_or(BeamError::Overflow)?;
        Ok(())
    }

    fn accrue(&mut self, trade: &Trade) -> Result<()> {
        self.rewards_accrued = self
            .rewards_accrued
            .checked_add(trade.fees.rewards)
            .ok_or(BeamError::Overflow)?;
        self.treasury_accrued = self
            .treasury_accrued
            .checked_add(trade.fees.treasury)
            .ok_or(BeamError::Overflow)?;
        self.founder_accrued = self
            .founder_accrued
            .checked_add(trade.fees.founder)
            .ok_or(BeamError::Overflow)?;
        Ok(())
    }
}

#[event]
pub struct LaunchCreated {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub total_supply: u64,
    pub real_token_reserves: u64,
    pub reserved_pool_tokens: u64,
}

#[event]
pub struct CurveTrade {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub buy: bool,
    pub tokens: u64,
    pub gross_quote: u64,
    pub trader_quote: u64,
    pub rewards_fee: u64,
    pub treasury_fee: u64,
    pub founder_fee: u64,
    pub curve_complete: bool,
}

fn emit_trade(user: Pubkey, mint: Pubkey, buy: bool, trade: Trade, curve_complete: bool) {
    emit!(CurveTrade {
        user,
        mint,
        buy,
        tokens: trade.tokens,
        gross_quote: trade.quote,
        trader_quote: trade.trader_quote,
        rewards_fee: trade.fees.rewards,
        treasury_fee: trade.fees.treasury,
        founder_fee: trade.fees.founder,
        curve_complete,
    });
}

fn map_core_error(error: beam_core::Error) -> anchor_lang::error::Error {
    match error {
        beam_core::Error::InvalidConfig => error!(BeamError::InvalidConfig),
        beam_core::Error::InvalidAmount => error!(BeamError::InvalidAmount),
        beam_core::Error::Overflow => error!(BeamError::Overflow),
        beam_core::Error::InsufficientTokens => error!(BeamError::InsufficientTokens),
        beam_core::Error::InsufficientQuote => error!(BeamError::InsufficientQuote),
        beam_core::Error::Slippage => error!(BeamError::Slippage),
        beam_core::Error::CurveComplete => error!(BeamError::CurveComplete),
        beam_core::Error::CurveIncomplete => error!(BeamError::CurveIncomplete),
    }
}

#[error_code]
pub enum BeamError {
    #[msg("Invalid launch parameters")]
    InvalidConfig,
    #[msg("Trade quantity must be positive")]
    InvalidAmount,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Not enough tokens are available")]
    InsufficientTokens,
    #[msg("Not enough quote is available")]
    InsufficientQuote,
    #[msg("Trade exceeds the slippage limit")]
    Slippage,
    #[msg("The bonding curve is complete")]
    CurveComplete,
    #[msg("The bonding curve is not complete")]
    CurveIncomplete,
    #[msg("Token vault balance is below the curve accounting balance")]
    VaultBalance,
    #[msg("Launch account cannot pay without breaking rent exemption")]
    InsufficientEscrow,
}
