pub mod create_fee_sharing_config; pub mod create_donation_fee_pda; pub mod create_social_fee_pda;
pub mod update_fee_shares_v2;
use anchor_lang::prelude::*;
use crate::FeeError;







































   pub mod claim_social_fee_pda; pub mod update_fee_shares;

use crate::donation::{BondingCurve, AMM, PUMP};




pub fn preprocess_remaining<'remaining, 'info>(
    remaining: &'remaining [AccountInfo<'info>],
    curve: &BondingCurve,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
) -> Result<&'remaining [AccountInfo<'info>]> {
    if !curve.complete {
        return Ok(remaining);
    }
    require!(!remaining.is_empty(), FeeError::NotEnoughRemainingAccounts);

    let (authority, _) = Pubkey::find_program_address(
        &[b"pool-authority", base_mint.as_ref()], &crate::pump::ID,
    );
    let (expected, _) = Pubkey::find_program_address(
        &[b"pool", &0u16.to_le_bytes(), authority.as_ref(),
          base_mint.as_ref(), quote_mint.as_ref()], &crate::pump_amm::ID,
    );


    let last_index = remaining.len() - 1;
    let last = &remaining[last_index];

    require_keys_eq!(last.key(), expected, anchor_lang::error::ErrorCode::ConstraintSeeds);

    if last.owner != &crate::pump_amm::ID { return Ok(&remaining[..last_index]); }
    let data = last.try_borrow_data()?;

    if data.len() < 287 {
        Ok(&remaining[..last_index])
    } else {
        let word = u64::from_le_bytes(data[279..287].try_into().unwrap());
        require!(word == 0, FeeError::PoolCreatorFeesNotSwept);
        Ok(&remaining[..last_index])
    }

}






pub fn legacy_authorize(authority: &Signer<'_>, state: &crate::donation::SharingConfig) -> Result<()> {


    if authority.key() != state.admin { return Err(error!(FeeError::NotAuthorized)); }
    if state.admin_revoked { return Err(FeeError::SharingConfigAdminRevoked.into()); }
    require!(state.status == crate::donation::ConfigStatus::Active, FeeError::SharingConfigNotActive);
    Ok(())
}




























































































































































// Reconstructed name; the original routine's historical identifier is unknown.
pub fn admit_associated_token_account<'info>(
    associated_account: &AccountInfo<'info>,
    wallet: &AccountInfo<'info>,
    mint_value: Pubkey,
    token_program_value: Pubkey,
    mint_info: &AccountInfo<'info>,
    token_program_info: &AccountInfo<'info>,
    create_if_missing: bool,
    payer: &Signer<'info>,
    system_program: &Program<'info, System>,
) -> Result<()> { let mint=&mint_value; let token_program=&token_program_value; let creation_result: Result<()> = try {
    let expected = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &wallet.key(), mint, token_program,
    );
    require_keys_eq!(associated_account.key(), expected, anchor_lang::error::ErrorCode::ConstraintSeeds);
    if associated_account.owner == &anchor_lang::system_program::ID {



        require!(create_if_missing, anchor_lang::error::ErrorCode::AccountNotInitialized);
        let instruction = anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &payer.key(), &wallet.key(), mint, token_program,
        );
        anchor_lang::solana_program::program::invoke(
            &instruction,
            &[
                payer.to_account_info(),
                associated_account.clone(),
                wallet.clone(),
                mint_info.clone(),
                system_program.to_account_info(),
                token_program_info.clone(),
            ],
        )?;
    }





    require_keys_eq!(*associated_account.owner, *token_program, FeeError::AccountTypeNotSupported); () }; creation_result?;
    let data = associated_account.try_borrow_data()?;
    let account = anchor_spl::token_interface::TokenAccount::try_deserialize(&mut &data[..])?;




    require_keys_eq!(account.mint, *mint, FeeError::InvalidMint);
    Ok(())
}


