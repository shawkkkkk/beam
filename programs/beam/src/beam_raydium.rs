//! Raydium CPMM migration candidate. Default builds cannot enable execution.
//! Source ABI: raydium-cp-swap 3d69122790e97cd6aff24e8ba95940793773f58c.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::{AccountMeta, Instruction}, program::{invoke, invoke_signed}, program_pack::Pack};
use anchor_spl::{associated_token::AssociatedToken, token::{self, Token}, token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked}};
use crate::{Global, PumpError, trade_types::BondingCurve, beam_policy::BeamError};

#[cfg(feature = "raydium-devnet")]
pub const RAYDIUM_ID: Pubkey = anchor_lang::solana_program::pubkey!("DRaycpLY18LhpbydsBWbVJtxpNv9oXPgjRSfpF2bWpYb");
#[cfg(not(feature = "raydium-devnet"))]
pub const RAYDIUM_ID: Pubkey = anchor_lang::solana_program::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
#[cfg(feature = "raydium-devnet")]
pub const CREATE_POOL_FEE: Pubkey = anchor_lang::solana_program::pubkey!("3oE58BKVt8KuYkGxx8zBojugnymWmBiyafWgMrnb6eYy");
#[cfg(not(feature = "raydium-devnet"))]
pub const CREATE_POOL_FEE: Pubkey = anchor_lang::solana_program::pubkey!("DNXgeM9EiiaAbaWvwjHj9fQQLAX5ZsfHyvmYUNRAdNC8");
pub const INITIALIZE_DISCRIMINATOR: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];

#[account]
pub struct BeamRaydiumConfig {
    pub amm_config: Pubkey,
    pub raydium_program: Pubkey,
    pub enabled: bool,
}

#[derive(Accounts)]
pub struct ConfigureRaydium<'info> {
    #[account(seeds = [b"global"], bump, has_one = authority)]
    pub global: Box<Account<'info, Global>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + 32 + 32 + 1, seeds = [b"beam-raydium-config"], bump)]
    pub config: Account<'info, BeamRaydiumConfig>,
    /// CHECK: owner, discriminator, length, index PDA and creation flag checked.
    #[account(owner = RAYDIUM_ID)]
    pub amm_config: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn validate_amm_config(key: &Pubkey, owner: &Pubkey, data: &[u8]) -> Result<()> {
    require_keys_eq!(*owner, RAYDIUM_ID, BeamError::InvalidRaydiumAccount);
    let discriminator = anchor_lang::solana_program::hash::hash(b"account:AmmConfig").to_bytes();
    require!(data.len() == 236 && data[..8] == discriminator[..8], BeamError::InvalidRaydiumAccount);
    require!(data[9] == 0, BeamError::InvalidRaydiumAccount);
    let index = u16::from_le_bytes([data[10], data[11]]);
    let expected = Pubkey::find_program_address(&[b"amm_config", &index.to_be_bytes()], &RAYDIUM_ID).0;
    require_keys_eq!(*key, expected, BeamError::InvalidRaydiumAccount);
    Ok(())
}

pub fn configure(ctx: Context<ConfigureRaydium>) -> Result<()> {
    validate_amm_config(&ctx.accounts.amm_config.key(), ctx.accounts.amm_config.owner, &ctx.accounts.amm_config.try_borrow_data()?)?;
    let c = &mut ctx.accounts.config;
    c.amm_config = ctx.accounts.amm_config.key();
    c.raydium_program = RAYDIUM_ID;
    c.enabled = false;
    Ok(())
}

#[derive(Accounts)]
pub struct SetRaydiumEnabled<'info> {
    #[account(seeds = [b"global"], bump, has_one = authority)]
    pub global: Box<Account<'info, Global>>,
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"beam-raydium-config"], bump)]
    pub config: Account<'info, BeamRaydiumConfig>,
}

pub fn set_enabled(ctx: Context<SetRaydiumEnabled>, enabled: bool) -> Result<()> {
    require!(!enabled || cfg!(feature = "experimental-raydium"), BeamError::MigrationNotEnabled);
    require_keys_eq!(ctx.accounts.config.raydium_program, RAYDIUM_ID, BeamError::InvalidRaydiumAccount);
    ctx.accounts.config.enabled = enabled;
    Ok(())
}

#[account]
pub struct BeamMigrationReceipt {
    pub mint: Pubkey,
    pub pool: Pubkey,
    pub token_amount: u64,
    pub quote_amount: u64,
    pub migration_fee: u64,
    pub lp_burned: u64,
    pub setup_spent: u64,
}

#[derive(Accounts)]
pub struct MigrateRaydium<'info> {
    #[account(seeds = [b"global"], bump, has_one = withdraw_authority)]
    pub global: Box<Account<'info, Global>>,
    /// CHECK: constrained to the configured recipient in Global.
    #[account(mut)]
    pub withdraw_authority: UncheckedAccount<'info>,
    #[account(mint::token_program = base_token_program)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, seeds = [b"bonding-curve", mint.key().as_ref()], bump)]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = bonding_curve, associated_token::token_program = base_token_program)]
    pub curve_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"beam-raydium-config"], bump, constraint = config.enabled @ BeamError::MigrationNotEnabled)]
    pub config: Account<'info, BeamRaydiumConfig>,
    /// CHECK: immutable configured address, parsed again immediately before CPI.
    #[account(address = config.amm_config, owner = RAYDIUM_ID)]
    pub amm_config: UncheckedAccount<'info>,
    /// CHECK: fixed executable program; no frontend-supplied program substitution.
    #[account(address = RAYDIUM_ID, executable)]
    pub raydium_program: UncheckedAccount<'info>,
    /// CHECK: system-owned signer PDA pays Raydium setup, not controlled by user.
    #[account(mut, seeds = [b"pool-authority", mint.key().as_ref()], bump, owner = anchor_lang::system_program::ID)]
    pub migration_authority: UncheckedAccount<'info>,
    #[account(init_if_needed, payer = user, associated_token::mint = mint, associated_token::authority = migration_authority, associated_token::token_program = base_token_program)]
    pub authority_base: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub wsol_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(init_if_needed, payer = user, associated_token::mint = wsol_mint, associated_token::authority = migration_authority, associated_token::token_program = token_program)]
    pub authority_wsol: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Raydium authority PDA checked before CPI.
    pub raydium_authority: UncheckedAccount<'info>,
    /// CHECK: canonical sorted-mint pool PDA; must not already exist.
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
    /// CHECK: canonical Raydium LP mint PDA; initialized by Raydium.
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,
    /// CHECK: canonical SPL Token ATA for the signer PDA; initialized by Raydium.
    #[account(mut)]
    pub authority_lp: UncheckedAccount<'info>,
    /// CHECK: canonical Raydium vault PDA for sorted mint 0.
    #[account(mut)]
    pub vault_0: UncheckedAccount<'info>,
    /// CHECK: canonical Raydium vault PDA for sorted mint 1.
    #[account(mut)]
    pub vault_1: UncheckedAccount<'info>,
    /// CHECK: pinned Raydium pool-creation fee recipient; validated by Raydium too.
    #[account(mut, address = CREATE_POOL_FEE)]
    pub create_pool_fee: UncheckedAccount<'info>,
    /// CHECK: canonical Raydium observation PDA; initialized by Raydium.
    #[account(mut)]
    pub observation: UncheckedAccount<'info>,
    #[account(init, payer = user, space = 8 + 32 + 32 + 5 * 8, seeds = [b"beam-migration", mint.key().as_ref()], bump)]
    pub receipt: Account<'info, BeamMigrationReceipt>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts follow Raydium Initialize exactly, including signer/writable bits.
pub fn initialize_instruction(keys: [Pubkey; 20], amount_0: u64, amount_1: u64, open_time: u64) -> Instruction {
    let mut data = INITIALIZE_DISCRIMINATOR.to_vec();
    for value in [amount_0, amount_1, open_time] { data.extend_from_slice(&value.to_le_bytes()); }
    let writable = [true,false,false,true,false,false,true,true,true,true,true,true,true,true,false,false,false,false,false,false];
    let accounts = keys.into_iter().enumerate().map(|(i,key)| if writable[i] { AccountMeta::new(key, i == 0) } else { AccountMeta::new_readonly(key, false) }).collect();
    Instruction { program_id: RAYDIUM_ID, accounts, data }
}

fn pda(account: &AccountInfo, seeds: &[&[u8]]) -> Result<()> {
    require_keys_eq!(account.key(), Pubkey::find_program_address(seeds, &RAYDIUM_ID).0, BeamError::InvalidRaydiumAccount);
    Ok(())
}

fn supported_mint(mint: &AccountInfo) -> Result<()> {
    if mint.owner == &token::ID { return Ok(()); }
    require_keys_eq!(*mint.owner, anchor_spl::token_2022::ID, BeamError::UnsupportedMint);
    use anchor_spl::token_2022::spl_token_2022::{extension::{BaseStateWithExtensions, StateWithExtensions, ExtensionType}, state::Mint as Mint2022};
    let data = mint.try_borrow_data()?;
    let state = StateWithExtensions::<Mint2022>::unpack(&data)?;
    for ext in state.get_extension_types()? {
        require!(matches!(ext, ExtensionType::MetadataPointer | ExtensionType::TokenMetadata), BeamError::UnsupportedMint);
    }
    Ok(())
}

fn checked_move(from: &AccountInfo, to: &AccountInfo, amount: u64) -> Result<()> {
    let left = from.lamports().checked_sub(amount).ok_or(BeamError::InvalidMigrationBalance)?;
    let right = to.lamports().checked_add(amount).ok_or(PumpError::Overflow)?;
    **from.try_borrow_mut_lamports()? = left;
    **to.try_borrow_mut_lamports()? = right;
    Ok(())
}

pub fn execute<'info>(ctx: Context<'_, '_, '_, 'info, MigrateRaydium<'info>>, setup_budget: u64) -> Result<()> {
    require!(cfg!(feature = "experimental-raydium"), BeamError::MigrationNotEnabled);
    let a = &ctx.accounts;
    require!(a.global.enable_migrate, BeamError::MigrationNotEnabled);
    require_keys_eq!(a.config.raydium_program, RAYDIUM_ID, BeamError::InvalidRaydiumAccount);
    require!(a.bonding_curve.complete && a.bonding_curve.real_token_reserves == 0, PumpError::BondingCurveNotComplete);
    crate::beam_policy::mayhem(a.bonding_curve.is_mayhem_mode)?;
    require_keys_eq!(a.bonding_curve.quote_mint, Pubkey::default(), PumpError::UnsupportedQuoteMint);
    require!(a.pool.data_is_empty() && a.lp_mint.data_is_empty(), BeamError::InvalidRaydiumAccount);
    require!(a.migration_authority.data_is_empty(), BeamError::InvalidRaydiumAccount);
    require!(a.authority_base.amount == 0 && a.authority_wsol.amount == 0, BeamError::InvalidMigrationBalance);
    supported_mint(&a.mint.to_account_info())?;
    validate_amm_config(&a.amm_config.key(), a.amm_config.owner, &a.amm_config.try_borrow_data()?)?;
    let mint = a.mint.key(); let wsol = a.wsol_mint.key(); let config = a.amm_config.key(); let pool = a.pool.key();
    let base_first = mint < wsol;
    let (mint_0, mint_1) = if base_first { (mint, wsol) } else { (wsol, mint) };
    pda(&a.pool.to_account_info(), &[b"pool", config.as_ref(), mint_0.as_ref(), mint_1.as_ref()])?;
    pda(&a.raydium_authority.to_account_info(), &[b"vault_and_lp_mint_auth_seed"])?;
    pda(&a.lp_mint.to_account_info(), &[b"pool_lp_mint", pool.as_ref()])?;
    pda(&a.vault_0.to_account_info(), &[b"pool_vault", pool.as_ref(), mint_0.as_ref()])?;
    pda(&a.vault_1.to_account_info(), &[b"pool_vault", pool.as_ref(), mint_1.as_ref()])?;
    pda(&a.observation.to_account_info(), &[b"observation", pool.as_ref()])?;
    let expected_lp = anchor_spl::associated_token::get_associated_token_address_with_program_id(&a.migration_authority.key(), &a.lp_mint.key(), &token::ID);
    require_keys_eq!(a.authority_lp.key(), expected_lp, BeamError::InvalidRaydiumAccount);
    let total_quote = a.bonding_curve.real_quote_reserves;
    let migration_fee = a.global.pool_migration_fee;
    let quote = total_quote.checked_sub(migration_fee).ok_or(BeamError::InvalidMigrationBalance)?;
    let base = a.curve_token_vault.amount;
    require!(base > 0 && base <= a.bonding_curve.token_total_supply && quote > 0 && setup_budget > 0, BeamError::InvalidMigrationBalance);
    let curve_info = a.bonding_curve.to_account_info();
    let required = Rent::get()?.minimum_balance(curve_info.data_len()).checked_add(total_quote).ok_or(PumpError::Overflow)?;
    require!(curve_info.lamports() >= required, BeamError::InvalidMigrationBalance);
    let authority_before = a.migration_authority.lamports();
    let curve_bump = [ctx.bumps.bonding_curve]; let curve_seeds: &[&[u8]] = &[b"bonding-curve", mint.as_ref(), &curve_bump];
    let authority_bump = [ctx.bumps.migration_authority]; let authority_seeds: &[&[u8]] = &[b"pool-authority", mint.as_ref(), &authority_bump];
    // Setup/rent comes from the caller's bounded budget, separate from LP SOL.
    anchor_lang::system_program::transfer(CpiContext::new(a.system_program.to_account_info(), anchor_lang::system_program::Transfer { from: a.user.to_account_info(), to: a.migration_authority.to_account_info() }), setup_budget)?;
    token_interface::transfer_checked(CpiContext::new_with_signer(a.base_token_program.to_account_info(), TransferChecked { from: a.curve_token_vault.to_account_info(), mint: a.mint.to_account_info(), to: a.authority_base.to_account_info(), authority: curve_info.clone() }, &[curve_seeds]), base, a.mint.decimals)?;
    checked_move(&curve_info, &a.authority_wsol.to_account_info(), quote)?;
    invoke(&token::spl_token::instruction::sync_native(&token::ID, &a.authority_wsol.key())?, &[a.authority_wsol.to_account_info(), a.token_program.to_account_info()])?;
    let (m0,m1,t0,t1,p0,p1,n0,n1) = if base_first {
        (a.mint.to_account_info(),a.wsol_mint.to_account_info(),a.authority_base.to_account_info(),a.authority_wsol.to_account_info(),a.base_token_program.to_account_info(),a.token_program.to_account_info(),base,quote)
    } else {
        (a.wsol_mint.to_account_info(),a.mint.to_account_info(),a.authority_wsol.to_account_info(),a.authority_base.to_account_info(),a.token_program.to_account_info(),a.base_token_program.to_account_info(),quote,base)
    };
    let infos = [a.migration_authority.to_account_info(),a.amm_config.to_account_info(),a.raydium_authority.to_account_info(),a.pool.to_account_info(),m0,m1,a.lp_mint.to_account_info(),t0,t1,a.authority_lp.to_account_info(),a.vault_0.to_account_info(),a.vault_1.to_account_info(),a.create_pool_fee.to_account_info(),a.observation.to_account_info(),a.token_program.to_account_info(),p0,p1,a.associated_token_program.to_account_info(),a.system_program.to_account_info(),a.rent.to_account_info()];
    let ix = initialize_instruction(std::array::from_fn(|i| infos[i].key()), n0, n1, 0);
    let mut cpi_infos = infos.to_vec(); cpi_infos.push(a.raydium_program.to_account_info());
    invoke_signed(&ix, &cpi_infos, &[authority_seeds])?;
    require_keys_eq!(*a.pool.owner, RAYDIUM_ID, BeamError::InvalidMigrationResult);
    require_keys_eq!(*a.lp_mint.owner, token::ID, BeamError::InvalidMigrationResult);
    require_keys_eq!(*a.authority_lp.owner, token::ID, BeamError::InvalidMigrationResult);
    let lp = token::spl_token::state::Account::unpack(&a.authority_lp.try_borrow_data()?)?;
    require_keys_eq!(lp.mint, a.lp_mint.key(), BeamError::InvalidMigrationResult);
    require_keys_eq!(lp.owner, a.migration_authority.key(), BeamError::InvalidMigrationResult);
    require!(lp.amount > 0, BeamError::InvalidMigrationResult);
    token::burn(CpiContext::new_with_signer(a.token_program.to_account_info(), token::Burn { mint: a.lp_mint.to_account_info(), from: a.authority_lp.to_account_info(), authority: a.migration_authority.to_account_info() }, &[authority_seeds]), lp.amount)?;
    require!(token::spl_token::state::Account::unpack(&a.authority_lp.try_borrow_data()?)?.amount == 0, BeamError::InvalidMigrationResult);
    checked_move(&curve_info, &a.withdraw_authority.to_account_info(), migration_fee)?;
    // Return only this caller's unspent setup contribution, never prior donations.
    let refund = a.migration_authority.lamports().saturating_sub(authority_before).min(setup_budget);
    if refund > 0 {
        anchor_lang::system_program::transfer(CpiContext::new_with_signer(a.system_program.to_account_info(), anchor_lang::system_program::Transfer { from: a.migration_authority.to_account_info(), to: a.user.to_account_info() }, &[authority_seeds]), refund)?;
    }
    let receipt = &mut ctx.accounts.receipt;
    receipt.mint = mint; receipt.pool = pool; receipt.token_amount = base; receipt.quote_amount = quote;
    receipt.migration_fee = migration_fee; receipt.lp_burned = lp.amount; receipt.setup_spent = setup_budget - refund;
    let curve = &mut ctx.accounts.bonding_curve;
    curve.real_quote_reserves = 0; curve.real_token_reserves = 0;
    curve.virtual_quote_reserves = 0; curve.virtual_token_reserves = 0;
    emit!(BeamRaydiumMigration { mint, pool, token_amount: base, quote_amount: quote, migration_fee, lp_burned: lp.amount });
    Ok(())
}

#[event]
pub struct BeamRaydiumMigration {
    pub mint: Pubkey, pub pool: Pubkey, pub token_amount: u64,
    pub quote_amount: u64, pub migration_fee: u64, pub lp_burned: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initialize_abi_has_exact_discriminator_arguments_and_privileges() {
        let keys = std::array::from_fn(|_| Pubkey::new_unique());
        let ix = initialize_instruction(keys, 7, u64::MAX, 13);
        assert_eq!(&ix.data[..8], &anchor_lang::solana_program::hash::hash(b"global:initialize").to_bytes()[..8]);
        assert_eq!(ix.data.len(), 32);
        assert_eq!(&ix.data[8..16], &7u64.to_le_bytes());
        assert_eq!(&ix.data[16..24], &u64::MAX.to_le_bytes());
        assert_eq!(&ix.data[24..32], &13u64.to_le_bytes());
        assert_eq!(ix.accounts.iter().filter(|m| m.is_signer).count(), 1);
        assert!(ix.accounts[0].is_signer && ix.accounts[0].is_writable);
        assert_eq!(ix.accounts.iter().filter(|m| m.is_writable).count(), 10);
        assert_eq!(ix.accounts.iter().map(|m| m.pubkey).collect::<Vec<_>>(), keys);
        assert_eq!(ix.program_id, RAYDIUM_ID);
    }
    #[test]
    fn config_checks_owner_length_discriminator_creation_flag_and_index_pda() {
        let mut data = vec![0;236];
        data[..8].copy_from_slice(&anchor_lang::solana_program::hash::hash(b"account:AmmConfig").to_bytes()[..8]);
        data[10..12].copy_from_slice(&7u16.to_le_bytes());
        let key = Pubkey::find_program_address(&[b"amm_config", &7u16.to_be_bytes()], &RAYDIUM_ID).0;
        assert!(validate_amm_config(&key, &RAYDIUM_ID, &data).is_ok());
        assert!(validate_amm_config(&Pubkey::new_unique(), &RAYDIUM_ID, &data).is_err());
        assert!(validate_amm_config(&key, &Pubkey::new_unique(), &data).is_err());
        assert!(validate_amm_config(&key, &RAYDIUM_ID, &data[..10]).is_err());
        data[9] = 1; assert!(validate_amm_config(&key, &RAYDIUM_ID, &data).is_err());
        data[9] = 0; data[0] ^= 1; assert!(validate_amm_config(&key, &RAYDIUM_ID, &data).is_err());
    }
}
