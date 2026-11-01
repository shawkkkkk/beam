use anchor_lang::{
    accounts::{account::Account, interface_account::InterfaceAccount, program::Program},
    prelude::{AccountInfo, CpiContext, Key, Pubkey, Result, System, ToAccountInfo},
    require,
};
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount},
};
use crate::{
    state::{Pool, UserVolumeAccumulator},
    PumpAmmError,
};

pub fn validate_self_transfer(
    source: &Pubkey,
    source_owner: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
) -> Result<()> {
    require!(source != destination || source_owner == authority, PumpAmmError::SelfTransferNotAllowed);
    Ok(())
}

pub fn transfer_checked_with_ata<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    ata_owner: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, anchor_spl::associated_token::AssociatedToken>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    validate_self_transfer(&source.key(), &source.owner, destination.key, authority.key)?;
    create_ata_if_empty(token_program, mint, ata_owner, destination, authority,
        system_program, associated_token_program)?;
    token_interface::transfer_checked(CpiContext::new_with_signer(
        token_program.clone(),
        token_interface::TransferChecked {
            from: source.to_account_info(), mint: mint.to_account_info(),
            to: destination.clone(), authority: authority.clone(),
        },
        signer_seeds,
    ), amount, mint.decimals)
}

#[inline(never)]
pub fn transfer_checked_to_account_info<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    destination: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    validate_self_transfer(&source.key(), &source.owner, destination.key, authority.key)?;
    token_interface::transfer_checked(CpiContext::new(
        token_program.clone(),
        token_interface::TransferChecked {
            from: source.to_account_info(),
            mint: mint.to_account_info(),
            to: destination.clone(),
            authority: authority.clone(),
        },
    ), amount, mint.decimals)
}

pub fn create_ata_if_empty<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    ata: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, anchor_spl::associated_token::AssociatedToken>,
) -> Result<()> {
    if ata.data_is_empty() {
        anchor_spl::associated_token::create_idempotent(CpiContext::new(
            associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: payer.clone(),
                associated_token: ata.clone(),
                authority: authority.clone(),
                mint: mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.clone(),
            },
        ))?;
    }
    Ok(())
}

pub fn transfer_from_pool_to_raw<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    pool: &Account<'info, Pool>,
    destination: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    validate_self_transfer(&source.key(), &source.owner, destination.key, &pool.key())?;
    token_interface::transfer_checked(CpiContext::new_with_signer(
        token_program.clone(),
        token_interface::TransferChecked {
            from: source.to_account_info(),
            mint: mint.to_account_info(),
            to: destination.clone(),
            authority: pool.to_account_info(),
        },
        &[&[
            b"pool",
            &pool.index.to_le_bytes(),
            pool.creator.as_ref(),
            pool.base_mint.as_ref(),
            pool.quote_mint.as_ref(),
            &[pool.pool_bump],
        ]],
    ), amount, mint.decimals)
}

pub fn transfer_from_pool<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    destination: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    pool: &Account<'info, Pool>,
) -> Result<()> {
    let pool_info = pool.to_account_info();
    let seeds: &[&[&[u8]]] = &[&[
        b"pool",
        &pool.index.to_le_bytes(),
        pool.creator.as_ref(),
        pool.base_mint.as_ref(),
        pool.quote_mint.as_ref(),
        &[pool.pool_bump],
    ]];
    transfer_checked(
        token_program,
        mint,
        source,
        destination,
        &pool_info,
        amount,
        seeds,
    )
}

pub fn transfer_from_user_volume_accumulator<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    destination: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    user: &Pubkey,
    accumulator: &Account<'info, UserVolumeAccumulator>,
    bump: u8,
) -> Result<()> {
    let authority = accumulator.to_account_info();
    let result = transfer_checked(
        token_program,
        mint,
        source,
        destination,
        &authority,
        amount,
        &[&[b"user_volume_accumulator", user.as_ref(), &[bump]]],
    );
    result
}

const PUMP_MINT: Pubkey = Pubkey::new_from_array([12, 69, 247, 223, 141, 158, 114, 149, 98, 132, 147, 63, 109, 152, 183, 87, 3, 46, 131, 223, 132, 96, 79, 181, 225, 23, 255, 246, 29, 91, 18, 249]);

pub fn transfer_checked<'info>(
    token_program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    destination: &InterfaceAccount<'info, TokenAccount>,
    authority: &AccountInfo<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    validate_self_transfer(&source.key(), &source.owner, &destination.key(), authority.key)?;
    token_interface::transfer_checked(CpiContext::new_with_signer(
        token_program.clone(),
        token_interface::TransferChecked {
            from: source.to_account_info(),
            mint: mint.to_account_info(),
            to: destination.to_account_info(),
            authority: authority.clone(),
        },
        signer_seeds,
    ), amount, mint.decimals)
}

pub fn mint_to_pool<'info>(
    token_program: &Program<'info, Token2022>,
    mint: &InterfaceAccount<'info, Mint>,
    destination: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    pool: &Account<'info, Pool>,
) -> Result<()> {
    token_interface::mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            token_interface::MintTo {
                mint: mint.to_account_info(),
                to: destination.to_account_info(),
                authority: pool.to_account_info(),
            },
            &[&[
                b"pool",
                &pool.index.to_le_bytes(),
                pool.creator.as_ref(),
                pool.base_mint.as_ref(),
                pool.quote_mint.as_ref(),
                &[pool.pool_bump],
            ]],
        ),
        amount,
    )
}


#[inline(never)]
pub fn burn_from_user<'info>(
    token_program: &Program<'info, Token2022>,
    mint: &InterfaceAccount<'info, Mint>,
    source: &InterfaceAccount<'info, TokenAccount>,
    authority: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    token_interface::burn(CpiContext::new(
        token_program.to_account_info(),
        token_interface::Burn {
            mint: mint.to_account_info(),
            from: source.to_account_info(),
            authority: authority.clone(),
        },
    ), amount)
}















































































pub fn supported_mint(ai: &AccountInfo, allow_extensions: bool) -> bool {
    if ai.key() == PUMP_MINT {
        return true;
    }
    native_transfer_fee::supported_mint_data(
        &ai.to_account_info().data.borrow(),
        allow_extensions,
    )
}

pub fn supported_mint_account(mint: &InterfaceAccount<Mint>, allow_extensions: bool) -> bool {
    supported_mint(mint.as_ref(), allow_extensions)
}
