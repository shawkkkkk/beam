use anchor_lang::prelude::*;
use crate::{ExtendAccount, GlobalParams, MayhemError};
use super::fee_recipients::FeeRecipient;























#[event(discriminator = [0x61,0x61,0xd7,0x90,0x5d,0x92,0x16,0x7c])]
pub struct ExtendAccountEvent {
    pub account: Pubkey,
    pub user: Pubkey,
    pub old_size: u64,
    pub new_size: u64,
    pub timestamp: i64,
}

pub fn handler(ctx: Context<ExtendAccount>) -> Result<()> {
    let old_size = ctx.accounts.account.data_len();
    let new_size = account_size(ctx.accounts.account.as_ref())?;

    if old_size == new_size {
        msg!("Account already has the correct size");
        return Ok(());
    }
    if old_size > new_size {
        msg!("Account already has more than {} bytes: {}", new_size, old_size);
        return Ok(());
    }
    let rent = Rent::get()?;
    let required = rent.minimum_balance(new_size).checked_sub(ctx.accounts.account.lamports()).unwrap_or(0);
    if required > 0 {
        fund_account(&ctx, required)?;
    }
    ctx.accounts.account.resize(new_size)?;
    let event = ExtendAccountEvent {
        account: ctx.accounts.account.key(), user: ctx.accounts.user.key(),
        old_size: old_size as u64, new_size: new_size as u64,
        timestamp: Clock::get()?.unix_timestamp,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}




















#[inline]
fn account_size(account: &AccountInfo) -> Result<usize> {




    require!(account.data_len() >= 8, MayhemError::AccountTypeNotSupported);
    let data = account.try_borrow_data()?;



    let discriminator =                            &data[..8];
    let size = match discriminator {
        d if d == GlobalParams::DISCRIMINATOR => 318,
        d if d == FeeRecipient::DISCRIMINATOR => 74,
        d if d == [0xb1,0xfd,0xbf,0x7d,0xcb,0x16,0x86,0x6b] => 170,









        _ => return err!(MayhemError::AccountTypeNotSupported),
    };
    Ok(size)
}





























#[inline(always)]
fn clone_info<'a>(info:&AccountInfo<'a>)->AccountInfo<'a> {
    AccountInfo { key:info.key, lamports:std::rc::Rc::clone(&info.lamports), data:std::rc::Rc::clone(&info.data), owner:info.owner, _unused:info._unused, is_signer:info.is_signer, is_writable:info.is_writable, executable:info.executable }
}

#[inline]
fn fund_account<'info>(ctx: &Context<ExtendAccount<'info>>, required: u64) -> Result<()> {
    anchor_lang::system_program::transfer(
        CpiContext::new(ctx.accounts.system_program.key(),
            anchor_lang::system_program::Transfer {
                from: clone_info(ctx.accounts.user.as_ref()), to: clone_info(ctx.accounts.account.as_ref())
            }), required)
}
