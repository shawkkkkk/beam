use crate::*;
use anchor_spl::{associated_token::{self,AssociatedToken},token_interface::TokenInterface};
#[event_cpi]
#[derive(Accounts)]
pub struct TransferCreatorFeesToPump<'info> {
 #[account(constraint=wsol_mint.key()==anchor_spl::token::spl_token::native_mint::ID @ PumpAmmError::UnsupportedQuoteMint)]
 pub wsol_mint: InterfaceAccount<'info,Mint>,
 pub token_program: Interface<'info,TokenInterface>,
 pub system_program: Program<'info,System>,
 pub associated_token_program: Program<'info,AssociatedToken>,
 /// CHECK: only its key derives the receiving vault.
 pub coin_creator: AccountInfo<'info>,
 /// CHECK: PDA signer constrained by seeds.
 #[account(mut,seeds=[b"creator_vault",coin_creator.key().as_ref()],bump)]
 pub coin_creator_vault_authority: AccountInfo<'info>,
 #[account(mut,associated_token::mint=wsol_mint,associated_token::authority=coin_creator_vault_authority,associated_token::token_program=token_program)]
 pub coin_creator_vault_ata: InterfaceAccount<'info,TokenAccount>,
 /// CHECK: destination is the canonical Pump creator vault PDA.
 #[account(mut,seeds=[b"creator-vault",coin_creator.key().as_ref()],bump,seeds::program=crate::trade::PUMP)]
 pub pump_creator_vault: AccountInfo<'info>,
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
pub fn legacy(ctx:Context<TransferCreatorFeesToPump>)->Result<()> {
 let a=&*ctx.accounts;

 let deficit = Rent::get()?.minimum_balance(a.pump_creator_vault.data_len())
     .saturating_sub(a.pump_creator_vault.to_account_info().lamports());
 let min_balance = Rent::get()?.minimum_balance(165);
 let minimum = min_balance.checked_add(deficit).ok_or(error!(PumpAmmError::Overflow))?;
 let available = a.coin_creator_vault_ata.to_account_info().lamports();
 if available < minimum {
     msg!("Available fees ({}) below minimum transfer amount ({}):", available, minimum);
     return Ok(());
 }
 
 token_interface::close_account(CpiContext::new_with_signer(
     a.token_program.to_account_info(),
     token_interface::CloseAccount {
         account:a.coin_creator_vault_ata.to_account_info(),
         destination:a.coin_creator_vault_authority.to_account_info(),
         authority:a.coin_creator_vault_authority.to_account_info(),
     }, &[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]],
 ))?;
 associated_token::create(CpiContext::new_with_signer(
     a.associated_token_program.to_account_info(),
     associated_token::Create {
         payer:a.coin_creator_vault_authority.to_account_info(),
         associated_token:a.coin_creator_vault_ata.to_account_info(),
         authority:a.coin_creator_vault_authority.to_account_info(),
         mint:a.wsol_mint.to_account_info(),
         system_program:a.system_program.to_account_info(),
         token_program:a.token_program.to_account_info(),
     }, &[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]],
 ))?;
 anchor_lang::system_program::transfer(CpiContext::new_with_signer(
     a.system_program.to_account_info(),
     anchor_lang::system_program::Transfer {
         from:a.coin_creator_vault_authority.to_account_info(),
         to:a.pump_creator_vault.to_account_info(),
     }, &[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]],
 ),a.coin_creator_vault_authority.lamports())?;
 Ok(())

}
#[event_cpi]
#[derive(Accounts)]
pub struct TransferCreatorFeesToPumpV2<'info> {
 #[account(mut)] pub payer: Signer<'info>,
 pub quote_mint: InterfaceAccount<'info,Mint>,
 pub token_program: Interface<'info,TokenInterface>,
 pub system_program: Program<'info,System>,
 pub associated_token_program: Program<'info,AssociatedToken>,
 /// CHECK: only its key derives the receiving vault.
 pub coin_creator: AccountInfo<'info>,
 /// CHECK: PDA signer constrained by seeds.
 #[account(mut,seeds=[b"creator_vault",coin_creator.key().as_ref()],bump)]
 pub coin_creator_vault_authority: AccountInfo<'info>,
 #[account(mut,associated_token::mint=quote_mint,associated_token::authority=coin_creator_vault_authority,associated_token::token_program=token_program)]
 pub coin_creator_vault_ata: InterfaceAccount<'info,TokenAccount>,
 /// CHECK: destination is the canonical Pump creator vault PDA.
 #[account(mut,seeds=[b"creator-vault",coin_creator.key().as_ref()],bump,seeds::program=crate::trade::PUMP)]
 pub pump_creator_vault: AccountInfo<'info>,
 /// CHECK: address constrained; handler creates it for non-native quotes.
 #[account(mut,seeds=[pump_creator_vault.key().as_ref(),token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]
 pub pump_creator_vault_ata: UncheckedAccount<'info>,
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
pub fn v2(ctx:Context<TransferCreatorFeesToPumpV2>)->Result<()> {
 let a=&*ctx.accounts;
 if a.quote_mint.key()==anchor_spl::token::spl_token::native_mint::ID {
 let deficit = Rent::get()?.minimum_balance(a.pump_creator_vault.data_len())
     .saturating_sub(a.pump_creator_vault.to_account_info().lamports());
 let min_balance = Rent::get()?.minimum_balance(165);
 let minimum = min_balance.checked_add(deficit).ok_or(error!(PumpAmmError::Overflow))?;
 let available = a.coin_creator_vault_ata.to_account_info().lamports();
 if available < minimum {
     msg!("Available fees ({}) below minimum transfer amount ({}):", available, minimum);
     return Ok(());
 }
 
 token_interface::close_account(CpiContext::new_with_signer(
     a.token_program.to_account_info(),
     token_interface::CloseAccount {
         account:a.coin_creator_vault_ata.to_account_info(),
         destination:a.coin_creator_vault_authority.to_account_info(),
         authority:a.coin_creator_vault_authority.to_account_info(),
     }, &[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]],
 ))?;
 associated_token::create(CpiContext::new_with_signer(
     a.associated_token_program.to_account_info(),
     associated_token::Create {
         payer:a.coin_creator_vault_authority.to_account_info(),
         associated_token:a.coin_creator_vault_ata.to_account_info(),
         authority:a.coin_creator_vault_authority.to_account_info(),
         mint:a.quote_mint.to_account_info(),
         system_program:a.system_program.to_account_info(),
         token_program:a.token_program.to_account_info(),
     }, &[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]],
 ))?;
 anchor_lang::system_program::transfer(CpiContext::new_with_signer(
     a.system_program.to_account_info(),
     anchor_lang::system_program::Transfer {
         from:a.coin_creator_vault_authority.to_account_info(),
         to:a.pump_creator_vault.to_account_info(),
     }, &[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]],
 ),a.coin_creator_vault_authority.lamports())?;
 return Ok(())
}
 
 if a.pump_creator_vault_ata.data_is_empty(){associated_token::create(CpiContext::new(a.associated_token_program.to_account_info(),associated_token::Create{payer:a.payer.to_account_info(),associated_token:a.pump_creator_vault_ata.to_account_info(),authority:a.pump_creator_vault.to_account_info(),mint:a.quote_mint.to_account_info(),system_program:a.system_program.to_account_info(),token_program:a.token_program.to_account_info()}))?;}
 token_interface::transfer_checked(CpiContext::new_with_signer(a.token_program.to_account_info(),token_interface::TransferChecked{from:a.coin_creator_vault_ata.to_account_info(),mint:a.quote_mint.to_account_info(),to:a.pump_creator_vault_ata.to_account_info(),authority:a.coin_creator_vault_authority.to_account_info()},&[&[b"creator_vault", a.coin_creator.key().as_ref(), &[ctx.bumps.coin_creator_vault_authority]]]),a.coin_creator_vault_ata.amount,a.quote_mint.decimals)?;
 Ok(())
}
