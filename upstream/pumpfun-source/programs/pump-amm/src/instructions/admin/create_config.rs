use crate::*;

const FIXED_ADMIN: Pubkey = anchor_lang::solana_program::pubkey!("8LWu7QM2dGR1G8nKDHthckea57bkCzXyBTAKPJUBDHo8");

#[event_cpi]
#[derive(Accounts)]
pub struct CreateConfig<'info> {
    /// Static-only reconstruction; fixed-admin positive execution is unvalidated.
    #[account(mut, address = FIXED_ADMIN @ PumpAmmError::InvalidAdmin)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 949, seeds = [b"global_config"], bump)]
    pub global_config: Account<'info, GlobalConfig>,
    pub system_program: Program<'info, System>,
}

#[event]
#[derive(Clone)]
pub struct CreateConfigEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub coin_creator_fee_basis_points: u64,
    pub admin_set_coin_creator_authority: Pubkey,
}

pub fn create_config(ctx: Context<CreateConfig>, lp_fee_basis_points: u64,
    protocol_fee_basis_points: u64, protocol_fee_recipients: [Pubkey; 8],
    coin_creator_fee_basis_points: u64, admin_set_coin_creator_authority: Pubkey) -> Result<()> {
    let admin = ctx.accounts.admin.key();
    let global_config = &mut ctx.accounts.global_config;
    global_config.admin = admin;
    global_config.set_lp_fee_basis_points(lp_fee_basis_points)?;
    global_config.set_protocol_fee_basis_points(protocol_fee_basis_points)?;
    global_config.set_protocol_fee_recipients(protocol_fee_recipients)?;
    global_config.set_coin_creator_fee_basis_points(coin_creator_fee_basis_points)?;
    global_config.disable_flags = 0;
    global_config.admin_set_coin_creator_authority = admin_set_coin_creator_authority;
    let event = CreateConfigEvent { timestamp: Clock::get()?.unix_timestamp,
        admin, lp_fee_basis_points, protocol_fee_basis_points,
        protocol_fee_recipients, coin_creator_fee_basis_points,
        admin_set_coin_creator_authority };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
