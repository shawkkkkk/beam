use anchor_lang::prelude::*;
use crate::{FeeError, ID};
pub use crate::contexts::ExtendFeeConfig;
use crate::types::ExtendFeeConfigEvent;

































pub fn execute(ctx: Context<ExtendFeeConfig>) -> Result<()> {
    let expected; let info = &ctx.accounts.fee_config;
    require_keys_eq!(*info.owner, ID, FeeError::InvalidFeeConfigAccount);




    let data = info.try_borrow_data()?;

    require_gte!(data.len(), 9usize, FeeError::InvalidFeeConfigAccount);




    require!(data.starts_with(<crate::types::FeeConfig as anchor_lang::Discriminator>::DISCRIMINATOR), FeeError::InvalidFeeConfigAccount);


    let bump_value = data[8];
    




    expected = Pubkey::create_program_address(&[b"fee_config", ctx.accounts.config_program_id.key().as_ref(), &[bump_value]], ctx.program_id)
        .map_err(|_| error!(FeeError::InvalidFeeConfigAccount))?;
    require_keys_eq!(expected, info.key(), FeeError::InvalidFeeConfigAccount);
    drop(data);
    let current_size = info.data_len(); let new_size = 4097usize;
    if current_size == new_size { msg!("Fee config account already has the correct size"); return Ok(()); } if current_size > new_size { msg!("Fee config account already has more than {} bytes: {}", new_size, current_size); return Ok(()); }
    let rent = Rent::get()?;
    let new_minimum = rent.minimum_balance(new_size);
    let current_minimum = rent.minimum_balance(current_size);
    if new_minimum > current_minimum {
    anchor_lang::system_program::transfer(CpiContext::new(
        ctx.accounts.system_program.to_account_info(), anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(), to: info.clone(),
        }), new_minimum.saturating_sub(current_minimum))?;
    }
    info.realloc(new_size, true)?;
    let event = ExtendFeeConfigEvent { fee_config: info.key(), user: ctx.accounts.user.key(),
        current_size: current_size as u64, new_size: new_size as u64, timestamp: Clock::get()?.unix_timestamp };
    emit!(event); emit_cpi!(event); Ok(())
}
