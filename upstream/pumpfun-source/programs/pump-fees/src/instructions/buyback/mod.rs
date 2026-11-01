use anchor_lang::prelude::*;
use crate::FeeError;
use crate::buyback::BuybackVault;








#[derive(Accounts)]
#[instruction(index:u8)]
pub struct InitializeBuyback<'info>{
 #[account(mut)]pub payer:Signer<'info>,
 #[account(init,payer=payer,space=208,seeds=[b"buyback-vault".as_ref(),&[index]],bump)]pub buyback_vault:Box<Account<'info,BuybackVault>>,
 #[account(init_if_needed,payer=payer,associated_token::mint=mint,associated_token::authority=buyback_vault,associated_token::token_program=token_program)]pub buyback_vault_ata:Box<InterfaceAccount<'info,anchor_spl::token_interface::TokenAccount>>,
 pub system_program:Program<'info,System>,pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: event target.
 pub program:AccountInfo<'info>,
}





































































 pub fn initialize_buyback(_ctx:Context<InitializeBuyback>,index:u8)->Result<()>{require_gt!(8u8,index,FeeError::InvalidBuybackIndex);Ok(())}













pub fn sweep_buyback(ctx:Context<crate::buyback::SweepBuyback>,index:u8)->Result<()>{ let bump_value = ctx.bumps.buyback_vault; let vault = &mut ctx.accounts.buyback_vault; let destination = &mut ctx.accounts.destination;
    require_gt!(8u8,index,FeeError::InvalidBuybackIndex);
    

    vault.claim()?;
    let sol_amount = vault.get_lamports().saturating_sub(Rent::get()?.minimum_balance(vault.to_account_info().data_len()));
    let token_amount = ctx.accounts.buyback_vault_ata.amount;
    
    anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),anchor_spl::token_interface::TransferChecked{from:ctx.accounts.buyback_vault_ata.to_account_info(),to:ctx.accounts.destination_ata.to_account_info(),authority:vault.to_account_info(),mint:ctx.accounts.mint.to_account_info()},&[&[b"buyback-vault",&[index],&[bump_value]]]),token_amount,ctx.accounts.mint.decimals)?;
    vault.sub_lamports(sol_amount)?;
    destination.add_lamports(sol_amount)?;
    vault.add_total_claimed(sol_amount)?;
    if ctx.accounts.mint.key()==anchor_spl::token::spl_token::native_mint::ID{vault.add_total_claimed_token1(token_amount)?;}else{vault.add_total_claimed_token2(token_amount)?;}
    let event=crate::buyback::SweepBuybackEvent{index,sol_amount,token_amount,destination:ctx.accounts.destination.key(),buyback_vault:vault.key(),mint:ctx.accounts.mint.key()};emit!(event);emit_cpi!(event);Ok(())
}
