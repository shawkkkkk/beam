use crate::*;
#[event_cpi]
#[derive(Accounts)]
pub struct ExtendAccount<'info> {
    /// CHECK: this instruction handles raw supported account discriminators.
    #[account(mut,owner=crate::ID)]
    pub account: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
pub fn execute(ctx: Context<ExtendAccount>) -> Result<()> {
    let account = &ctx.accounts.account;
    let current_size = account.data_len();
    let new_size = account_size(account)?;
    if current_size == new_size {
        msg!("Account already has the correct size");
        return Ok(());
    }
    if current_size > new_size {
        msg!("Account already has more than {} bytes: {}", new_size, current_size);
        return Ok(());
    }
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let current_minimum_balance = rent.minimum_balance(current_size);
    if new_minimum_balance > current_minimum_balance {
        let lamports = new_minimum_balance.saturating_sub(current_minimum_balance);
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: account.to_account_info(),
                },
            ),
            lamports,
        )?;
    }
    account.realloc(new_size, false)?;
    let event = ExtendAccountEvent {
        timestamp: Clock::get()?.unix_timestamp,
        account: account.key(),
        user: ctx.accounts.user.key(),
        current_size: current_size as u64,
        new_size: new_size as u64,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
fn account_size(account: &AccountInfo) -> Result<usize> {
    require!(account.data_len() >= 8, PumpAmmError::AccountTypeNotSupported);
    //
    //
    //
    //
    let type_discriminator = &account.try_borrow_data()?[..8];
    let size = match type_discriminator {
        d if d.starts_with(&GlobalConfig::DISCRIMINATOR) => 949,
        d if d.starts_with(&Pool::DISCRIMINATOR) => 301,
        d if d.starts_with(&UserVolumeAccumulator::DISCRIMINATOR) => 137,
        d if d.starts_with(&[202, 42, 246, 43, 142, 190, 30, 255]) => 600,
        _ => {
            //
            //
            //
            //
            //
            //
            //
            //
            //
            //
            //
            return err!(PumpAmmError::AccountTypeNotSupported);
        }
    };
    Ok(size)
}
#[event]
#[derive(Clone)]
pub struct ExtendAccountEvent {
    pub timestamp: i64,
    pub account: Pubkey,
    pub user: Pubkey,
    pub current_size: u64,
    pub new_size: u64,
}
