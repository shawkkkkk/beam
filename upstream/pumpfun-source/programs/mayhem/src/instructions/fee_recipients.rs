use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint,TokenAccount,TokenInterface}}; use crate::GlobalParams;










#[event(discriminator = [218,130,134,253,101,109,149,152])]
pub struct SweepEvent { pub amount: u64, pub timestamp: i64 }





#[account]
pub struct FeeRecipient {
 pub bump:u8,pub index:u8,
 pub field_10:u64,pub field_18:u64,pub field_26:u64,pub field_34:u64,
}




















impl FeeRecipient {
    pub fn validate_sweep(&mut self, global_params:&GlobalParams, native:bool)->Result<()> {
        let current_time = Clock::get().unwrap().unix_timestamp;




        let last = if native {self.field_10}else{self.field_34} as i64;
        let flag = &native;


        if last != 0 && global_params.field_10 as i64 != 0 {
            let interval = global_params.field_10 as i64;
            let next_time =    last + interval;


            require!(current_time >= next_time, crate::MayhemError::SweepFeeRateLimit);
        }
        if *flag {self.field_10=current_time as u64;}else{self.field_34=current_time as u64;}
        Ok(())
    }
}






#[derive(Accounts)]
#[instruction(index: u8)]
pub struct InitializeFeeRecipient<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub wsol_mint: InterfaceAccount<'info, Mint>,
    #[account(init, payer = admin, space = 42, seeds = [b"fee-recipient".as_ref(), &[index]], bump)]
    pub fee_recipient: Account<'info, FeeRecipient>,
    #[account(init_if_needed, payer = admin, associated_token::mint = wsol_mint, associated_token::authority = fee_recipient, associated_token::token_program = token_program)]
    pub fee_recipient_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, seeds = [b"global-params"], bump)]
    pub global_params: Account<'info, GlobalParams>,
    pub system_program: Program<'info,System>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}






























































































pub fn initialize(ctx: Context<InitializeFeeRecipient>, index:u8)->Result<()> {
    require_gt!(8,index,crate::MayhemError::MaxFeeRecipientsExceeded);
    ctx.accounts.fee_recipient.bump=ctx.bumps.fee_recipient;
    ctx.accounts.fee_recipient.index=index;
    ctx.accounts.fee_recipient.field_10=0;
    ctx.accounts.fee_recipient.field_18=0;
    ctx.accounts.fee_recipient.field_26=0;
    ctx.accounts.fee_recipient.field_34=0;
    Ok(())
}










fn sweepable(total: u64, amount: u64, limit: u64) -> u64 {
    if total == limit { return 0; }
    if total + amount >= limit {
        return limit - total;
    }
    amount
}




















































impl<'info> crate::SweepFees<'info> {
    fn sweep(&self, remaining_accounts: &'info [AccountInfo<'info>], native: bool, global_params: &Account<'info, GlobalParams>) -> Result<u64> {

        let limit = if native { global_params.max_sol_per_sweep } else { global_params.max_stable_per_sweep };
        let mut total = 0u64; let trader: &AccountInfo<'info> = self.trader.as_ref(); let trader_quote_ata: &AccountInfo<'info> = self.trader_quote_ata.as_ref(); let token_program: &AccountInfo<'info> = self.token_program.as_ref();
        for pair in remaining_accounts.chunks_exact(2) {
            let mut recipient = Account::<FeeRecipient>::try_from(&pair[0])?;
            let ata = InterfaceAccount::<TokenAccount>::try_from(&pair[1])?;
            recipient.validate_sweep(global_params, native)?;
            let index = &[recipient.index]; let seeds: &[&[u8]] = &[b"fee-recipient", index, &[recipient.bump]];
            let mut amount = sweepable(total, ata.amount, limit);
            if amount > 0 { anchor_spl::token::transfer(CpiContext::new_with_signer(token_program.key(), anchor_spl::token::Transfer { from: ata.to_account_info(), to: trader_quote_ata.to_account_info(), authority: recipient.to_account_info() }, &[seeds]), amount)?;
                total = total.checked_add(amount)                                 .unwrap();
            }
            let mut lamports = 0;
            if native {

                lamports = recipient.get_lamports();
                let rent = Rent::get()?;
                let space = pair[0].data_len();
                lamports -= rent.minimum_balance(space);
                lamports = sweepable(total, lamports, limit);
            }


            if lamports > 0 {
                **recipient.to_account_info().try_borrow_mut_lamports()? -= lamports;
                **trader.to_account_info().try_borrow_mut_lamports()? += lamports;


                amount = amount.checked_add(lamports)
                    .unwrap();
                total = total.checked_add(lamports)                                 .unwrap();
            }




            if native {
                recipient.field_18 += amount;
            } else {

                recipient.field_26 += amount;
            }
            recipient.exit(&crate::ID)?;
        }
        Ok(total)
    }
}



























impl<'info> crate::SweepFeesAdmin<'info> {
    fn sweep(&self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<u64> {
        let native = self.quote_mint.key() == anchor_spl::token::spl_token::native_mint::ID;
        let mut total = 0u64; let admin: &AccountInfo<'info> = self.admin.as_ref(); let admin_quote_ata: &AccountInfo<'info> = self.admin_quote_ata.as_ref(); let token_program: &AccountInfo<'info> = self.token_program.as_ref();
        for pair in remaining_accounts.chunks_exact(2) {
            let mut recipient = Account::<FeeRecipient>::try_from(&pair[0])?;
            let ata = InterfaceAccount::<TokenAccount>::try_from(&pair[1])?;
            let index = &[recipient.index]; let seeds: &[&[u8]] = &[b"fee-recipient", index, &[recipient.bump]];
            let mut amount = ata.amount;
            if amount > 0 {
                anchor_spl::token::transfer(CpiContext::new_with_signer(token_program.key(), anchor_spl::token::Transfer { from: ata.to_account_info(), to: admin_quote_ata.to_account_info(), authority: recipient.to_account_info() }, &[seeds]), amount)?;


                total = total.checked_add(amount)                                 .unwrap();
            }
            let mut lamports = 0;
            if native {

                lamports =          recipient.get_lamports() - Rent::get()?.minimum_balance(pair[0].data_len());
            }

            if lamports > 0 {
                **recipient.to_account_info().try_borrow_mut_lamports()? -= lamports;
                **admin.to_account_info().try_borrow_mut_lamports()? += lamports;


                amount = amount.checked_add(lamports)
                    .unwrap();
                total = total.checked_add(lamports)                                 .unwrap();
            }




            if native {
                recipient.field_18 += amount;
            } else {

                recipient.field_26 += amount;
            }
            recipient.exit(&crate::ID)?;
        }
        Ok(total)
    }
}












pub(crate) fn sweep_fees_admin<'info>(ctx: Context<'info, crate::SweepFeesAdmin<'info>>) -> Result<()> {
    let accounts = &*ctx.accounts;
    require!(ctx.remaining_accounts.len() == 16, crate::MayhemError::AccountTypeNotSupported);
    let amount = accounts.sweep(ctx.remaining_accounts)?;
    let event = SweepEvent { amount, timestamp: Clock::get()?.unix_timestamp };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}























pub(crate) fn sweep_fees<'info>(ctx: Context<'info, crate::SweepFees<'info>>) -> Result<()> {

    require!(ctx.remaining_accounts.len() == 16, crate::MayhemError::AccountTypeNotSupported);
    let native = ctx.accounts.quote_mint.key() == anchor_spl::token::spl_token::native_mint::ID; let amount = ctx.accounts.sweep(ctx.remaining_accounts, native, &ctx.accounts.global_params)?;
    let event = SweepEvent { amount, timestamp: Clock::get()?.unix_timestamp };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
