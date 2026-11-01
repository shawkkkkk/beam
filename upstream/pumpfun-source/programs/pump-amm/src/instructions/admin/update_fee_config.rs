use crate::*;

pub fn update_fee_config(ctx: Context<AdminGlobal>, lp_fee_basis_points: u64,
    protocol_fee_basis_points: u64, protocol_fee_recipients: [Pubkey; 8],
    coin_creator_fee_basis_points: u64, admin_set_coin_creator_authority: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.global_config;
    config.set_lp_fee_basis_points(lp_fee_basis_points)?;
    config.set_protocol_fee_basis_points(protocol_fee_basis_points)?;
    config.set_protocol_fee_recipients(protocol_fee_recipients)?;
    config.set_coin_creator_fee_basis_points(coin_creator_fee_basis_points)?;
    config.admin_set_coin_creator_authority = admin_set_coin_creator_authority;
    let event = UpdateFeeConfigEvent { timestamp: Clock::get()?.unix_timestamp,
        admin: ctx.accounts.admin.key(), lp_fee_basis_points, protocol_fee_basis_points,
        protocol_fee_recipients, coin_creator_fee_basis_points, admin_set_coin_creator_authority };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

#[event]
#[derive(Clone)]
pub struct UpdateFeeConfigEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub coin_creator_fee_basis_points: u64,
    pub admin_set_coin_creator_authority: Pubkey,
}
