use crate::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

#[event_cpi]
#[derive(Accounts)]
pub struct AdminUpdateTokenIncentives<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(seeds = [b"global_config"], bump, has_one = admin @ PumpAmmError::InvalidAdmin)]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut, seeds = [b"global_volume_accumulator"], bump)]
    pub global_volume_accumulator: Account<'info, GlobalVolumeAccumulator>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = global_volume_accumulator,
        associated_token::token_program = token_program,
    )]
    pub global_incentive_token_account: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(
    ctx: Context<AdminUpdateTokenIncentives>,
    start_time: i64,
    end_time: i64,
    seconds_in_a_day: i64,
    day_number: u64,
    token_supply_per_day: u64,
) -> Result<()> {
    let mint = ctx.accounts.mint.key();
    let timestamp = Clock::get()?.unix_timestamp;

    ctx.accounts.global_volume_accumulator.update_token_incentives(
        start_time,
        end_time,
        seconds_in_a_day,
        &mint.key(),
        day_number,
        token_supply_per_day,
        timestamp,
    )?;

    let event = AdminUpdateTokenIncentivesEvent {
        start_time,
        end_time,
        day_number,
        token_supply_per_day,
        mint,
        seconds_in_a_day,
        timestamp,
    };
    emit!(event.clone());
    emit_cpi!(event);

    msg!("GlobalVolumeAccumulator={:?}", ctx.accounts.global_volume_accumulator);

    Ok(())
}
