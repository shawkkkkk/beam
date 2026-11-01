use crate::*;
#[event_cpi]
#[derive(Accounts)]
pub struct AdminSetIdlAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(seeds=[b"global"],bump,has_one=authority @ PumpError::NotAuthorized)]
    pub global: Account<'info,Global>,
    /// CHECK: canonical Anchor IDL account, checked and resized by the handler.
    #[account(mut,address=anchor_lang::idl::IdlAccount::address(&crate::ID))]
    pub idl_account: AccountInfo<'info>,
    pub system_program: Program<'info,System>,
    /// CHECK: the program signs its own canonical empty-seed PDA.
    #[account(seeds=[],bump)]
    pub program_signer: AccountInfo<'info>,
}
pub(crate) fn execute(ctx:Context<AdminSetIdlAuthority>,idl_authority:Pubkey)->Result<()> {
    let account=&ctx.accounts.idl_account;
    if account.owner==&ctx.accounts.system_program.key() {
        let signer_seeds:&[&[&[u8]]]=&[&[&[],&[ctx.bumps.program_signer]]];
        return anchor_lang::system_program::transfer_with_seed(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::TransferWithSeed {
                    from:account.clone(),
                    base:ctx.accounts.program_signer.to_account_info(),
                    to:ctx.accounts.authority.to_account_info(),
                },
                signer_seeds,
            ),
            String::from("anchor:idl"),
            &crate::ID,
            account.lamports(),
        );

// layout-pad(36): aligns the next Source site with the original line



































    } else {
        require!(account.owner==&crate::ID,PumpError::AccountTypeNotSupported);
        account.realloc(10240,false)?;
// layout-pad(5): aligns the next Source site with the original line




        require!(account.data_len()>=8,PumpError::AccountTypeNotSupported);
// layout-pad(10): aligns the next Source site with the original line




        let is_idl_account = account.try_borrow_data()?
            [..8] == *anchor_lang::idl::IdlAccount::DISCRIMINATOR;



        require!(is_idl_account,PumpError::AccountTypeNotSupported);
    }




    let account_data = account.data.borrow();
    let mut header=anchor_lang::idl::IdlAccount::try_deserialize(&mut &account_data[..])?;
    drop(account_data);
    header.authority=idl_authority;

    let mut account_data_mut = account.data.borrow_mut();
    header.try_serialize(&mut &mut account_data_mut[..])?;
    drop(account_data_mut);
    let event=AdminSetIdlAuthorityEvent{idl_authority};emit!(event.clone());emit_cpi!(event);return Ok(());
}
#[event]
#[derive(Clone)]
pub struct AdminSetIdlAuthorityEvent {pub idl_authority:Pubkey}
