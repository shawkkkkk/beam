use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{FeeError, withdrawal::{Withdraw, WithdrawEvent}};
























































pub fn execute(ctx: Context<Withdraw>) -> Result<()> {
    let user = ctx.accounts.user.key();
    let signer_seeds: &[&[&[u8]]] = &[&[b"referral-vault", user.as_ref(), &[ctx.bumps.referral_vault]]];
    let (mint, amount) = match ctx.accounts.quote_mint.as_ref() {
        None => (Pubkey::default(), withdraw_native(ctx.accounts, signer_seeds)?),
        Some(quote) => (quote.key(), withdraw_token(ctx.accounts, quote, signer_seeds)?),
    };
    let event = WithdrawEvent { timestamp: Clock::get()?.unix_timestamp, user, quote_mint: mint, amount };
    emit!(event); emit_cpi!(event); Ok(())
}















fn withdraw_native<'info>(accounts: &Withdraw<'info>, signer_seeds: &[&[&[u8]]]) -> Result<u64> { let vault = &accounts.referral_vault; let user = &accounts.user; let system = &accounts.system_program;

    let amount = vault.lamports().saturating_sub(Rent::get()?.minimum_balance(0));



    require!(amount > 0, FeeError::NothingToWithdraw);

    anchor_lang::system_program::transfer(CpiContext::new_with_signer(system.to_account_info(), anchor_lang::system_program::Transfer { from: vault.to_account_info(), to: user.to_account_info() }, signer_seeds), amount)?;
    Ok(amount)
}



















fn withdraw_token<'info>(accounts: &Withdraw<'info>, quote: &InterfaceAccount<'info, Mint>, signer_seeds: &[&[&[u8]]]) -> Result<u64> { let authority = &accounts.referral_vault;

    let vault = accounts.vault_ata.as_ref().ok_or(FeeError::IncompleteTokenAccounts)?;
    let destination = accounts.user_ata.as_ref().ok_or(FeeError::IncompleteTokenAccounts)?;
    let token = accounts.token_program.as_ref().ok_or(FeeError::IncompleteTokenAccounts)?;

    let amount = vault.amount;



    require!(amount > 0, FeeError::NothingToWithdraw);

    anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(token.to_account_info(), anchor_spl::token_interface::TransferChecked { from: vault.to_account_info(), mint: quote.to_account_info(), to: destination.to_account_info(), authority: authority.to_account_info() }, signer_seeds), amount, quote.decimals)?;
    Ok(amount)
}
