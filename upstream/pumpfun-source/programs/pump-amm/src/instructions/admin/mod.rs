pub mod admin_cto_pool;
pub mod admin_update_token_incentives;
pub mod create_config;
pub mod disable;
pub mod set_boost_authority;
pub mod set_reserved_fee_recipients;
pub mod toggle_cashback_enabled;
pub mod toggle_boost;
pub mod toggle_mayhem_mode;
pub mod update_admin;
pub mod update_buyback_config;
pub mod update_creator_fee_config;
pub mod update_fee_config;
use crate::*;

pub fn extend_global_config<'info>(
    global_config: &Account<'info, GlobalConfig>,
    system_program: &Program<'info, System>,
    payer: &Signer<'info>,
) -> Result<()> {
    let account = global_config.to_account_info();
    if account.data_len() < 949 {
        let required = Rent::get()?.minimum_balance(949);
        let top_up = required.saturating_sub(account.lamports());
        if top_up > 0 {
            anchor_lang::system_program::transfer(CpiContext::new(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(), to: account.clone(),
                }), top_up)?;
        }
        account.realloc(949, false)?;
    }
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct AdminGlobal<'info> {
    pub admin: Signer<'info>,
    #[account(mut, has_one = admin @ PumpAmmError::InvalidAdmin)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    pub admin: Signer<'info>,
    #[account(mut, has_one = admin @ PumpAmmError::InvalidAdmin)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    /// CHECK: Only this account's public key is stored as the new admin.
    pub new_admin: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AdminGlobalNoEvent<'info> {
    pub admin: Signer<'info>,
    #[account(mut, has_one = admin @ PumpAmmError::InvalidAdmin)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct SetBoostAuthority<'info> {
    pub admin: Signer<'info>,
    #[account(mut, has_one = admin @ PumpAmmError::InvalidAdmin)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    /// CHECK: Only its public key is stored as authority.
    pub boost_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateCreatorFeeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, has_one = admin @ PumpAmmError::InvalidAdmin)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    pub system_program: Program<'info, System>,
}
