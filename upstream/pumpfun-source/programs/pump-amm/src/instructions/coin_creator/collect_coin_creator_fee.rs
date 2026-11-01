use crate::*;
use anchor_spl::token_interface::TokenInterface;

/// Collects the accumulated coin creator fees from the creator vault ATA into
/// the coin creator's own token account.
#[event_cpi]
#[derive(Accounts)]
pub struct CollectCoinCreatorFee<'info> {
    pub quote_mint: InterfaceAccount<'info, Mint>,

    pub quote_token_program: Interface<'info, TokenInterface>,

    /// CHECK: the destination authority is this key; migrated Fee-owned accounts are rejected.
    pub coin_creator: AccountInfo<'info>,

    /// CHECK: PDA constrained by creator-vault seeds.
    #[account(
        seeds = [b"creator_vault", coin_creator.key().as_ref()],
        bump
    )]
    pub coin_creator_vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = coin_creator_vault_authority,
        associated_token::token_program = quote_token_program
    )]
    pub coin_creator_vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = quote_mint,
        token::authority = coin_creator,
        token::token_program = quote_token_program
    )]
    pub coin_creator_token_account: InterfaceAccount<'info, TokenAccount>,
}

#[event]
#[derive(Clone)]
pub struct CollectCoinCreatorFeeEvent {
    /// Unix timestamp of the collection.
    pub timestamp: i64,
    /// Creator whose vault was drained.
    pub coin_creator: Pubkey,
    /// Amount of quote tokens transferred to the creator.
    pub coin_creator_fee: u64,
    /// Creator vault ATA the fees were collected from.
    pub coin_creator_vault_ata: Pubkey,
    /// Destination token account owned by the creator.
    pub coin_creator_token_account: Pubkey,
}

/// Transfers the full balance of the creator vault ATA to the coin creator.
///
/// The vault authority is the `creator_vault` PDA derived from the creator
/// key, so the transfer is signed with those seeds. Creators whose vault has
/// been migrated to a fee sharing config must collect through the fee program
/// instead, so their accounts are rejected here.
///
/// Collecting from an empty vault is a no-op and does not emit an event.
///
/// Emits [`CollectCoinCreatorFeeEvent`] both as a log and as a self-CPI.
pub fn execute(ctx: Context<CollectCoinCreatorFee>) -> Result<()> {
    let a = &ctx.accounts;

    require!(*a.coin_creator.owner != crate::trade::FEE_OWNER, PumpAmmError::CreatorVaultMigratedToSharingConfig);
    let amount = a.coin_creator_vault_ata.amount;
    if amount == 0 {
        msg!("No coin creator fee to collect");
        return Ok(());
    }
    let creator = a.coin_creator.key();
    crate::utils::token::transfer_checked(a.quote_token_program.as_ref(), &a.quote_mint, &a.coin_creator_vault_ata, &a.coin_creator_token_account, a.coin_creator_vault_authority.as_ref(), amount, &[&[b"creator_vault", creator.as_ref(), &[ctx.bumps.coin_creator_vault_authority]]])?;

    let event = CollectCoinCreatorFeeEvent {
        timestamp: Clock::get()?.unix_timestamp,
        coin_creator: creator,
        coin_creator_fee: amount,
        coin_creator_vault_ata: a.coin_creator_vault_ata.key(),
        coin_creator_token_account: a.coin_creator_token_account.key(),
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
