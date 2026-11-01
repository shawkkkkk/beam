//! Beam AMM v4 preflight/funding/settlement exercised with synthetic accounts.
//! ProgramTest registers test-only processors under the Beam, SPL Token and
//! Raydium devnet IDs. None is the captured original binary or Beam's real
//! migration entrypoint. This says nothing about live Raydium CPI, LP rights,
//! authorized fee recipients, full setup-rent attribution or enabled migration.
use anchor_lang::solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey,
};
use beam_program::{
    beam_amm_v4::{
        derive_pool_addresses, expected_initial_user_lp, initialize2_instruction,
        preflight_native_sol_initialize2, user_lp_ata, validate_create_pool_fee_account,
        Initialize2Accounts, Initialize2Args, MintKey, AMM_CONFIG_DATA_LEN, DEVNET_PROGRAM_ID,
        SPL_TOKEN_PROGRAM_ID,
    },
    beam_amm_v4_funding::{plan, BaseMintProgram, FundingInputs, QuoteAsset},
    beam_amm_v4_settlement::{expected_keys, verify, Balances, Observation, SettlementKeys},
};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account, bpf_loader_upgradeable, pubkey::Pubkey, signature::Signer,
    system_instruction, system_program, transaction::Transaction,
};

const BEAM_MINT: Pubkey = Pubkey::new_from_array([42; 32]);
const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const BASE_AMOUNT: u64 = 2_000_000;
const POOL_QUOTE: u64 = 1_990_000;
const MIGRATION_FEE: u64 = 10_000;
const CREATE_FEE: u64 = 50_000;
const RENT_RESERVE: u64 = 3_000_000;
const CURVE_START: u64 = 5_220_000;
const SETUP_START: u64 = 2_000_000;
const SETUP_BUDGET: u64 = 100_000;
const FEE_DEST_START: u64 = RENT_RESERVE;
const MIGRATION_RECIPIENT_START: u64 = 1_000_000;
/// Stand-in for a recipient selected by an independent, stored Beam policy.
const STORED_MIGRATION_RECIPIENT: Pubkey = Pubkey::new_from_array([73; 32]);
const ROGUE_MIGRATION_RECIPIENT: Pubkey = Pubkey::new_from_array([74; 32]);
const RECEIPT_REPLAY: u32 = 93;
const MOCK_LATE_ERROR: u32 = 94;
const CURVE_SEED: &[u8] = b"bonding-curve";

fn setup_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool-authority", BEAM_MINT.as_ref()], &beam_program::ID)
}

fn curve_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CURVE_SEED, BEAM_MINT.as_ref()], &beam_program::ID)
}

fn get_u64(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    Ok(u64::from_le_bytes(
        data.get(offset..offset + 8)
            .ok_or(ProgramError::InvalidAccountData)?
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    ))
}

fn put_u64(data: &mut [u8], offset: usize, value: u64) -> ProgramResult {
    data.get_mut(offset..offset + 8)
        .ok_or(ProgramError::InvalidAccountData)?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

// SPL's fixed 82-byte Mint and 165-byte Account layouts. The real Beam
// preflight unpacks these bytes with SPL Token; the mock token program below
// only changes amounts and native backing in this local test.
fn mint_bytes(decimals: u8) -> Vec<u8> {
    let mut data = vec![0; 82];
    data[44] = decimals;
    data[45] = 1;
    data
}

fn token_bytes(mint: Pubkey, authority: Pubkey, amount: u64, native: bool) -> Vec<u8> {
    let mut data = vec![0; 165];
    data[..32].copy_from_slice(mint.as_ref());
    data[32..64].copy_from_slice(authority.as_ref());
    data[64..72].copy_from_slice(&amount.to_le_bytes());
    data[108] = 1; // Initialized
    if native {
        data[109..113].copy_from_slice(&1u32.to_le_bytes());
        data[113..121].copy_from_slice(&RENT_RESERVE.to_le_bytes());
    }
    data
}

fn token_amount(info: &AccountInfo<'_>) -> Result<u64, ProgramError> {
    get_u64(&info.try_borrow_data()?, 64)
}

fn native_reserve(info: &AccountInfo<'_>) -> Result<u64, ProgramError> {
    let data = info.try_borrow_data()?;
    if data.get(109..113) != Some(1u32.to_le_bytes().as_slice()) {
        return Err(ProgramError::InvalidAccountData);
    }
    get_u64(&data, 113)
}

fn token_ix(tag: u8, metas: Vec<AccountMeta>, amount: u64) -> Instruction {
    let mut data = vec![tag];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction {
        program_id: SPL_TOKEN_PROGRAM_ID,
        accounts: metas,
        data,
    }
}

// Test-only SPL behavior: sync native, transfer exact amount, mint a synthetic
// LP amount. This is not the SPL Token binary or its full validation logic.
fn mock_token<'a>(id: &Pubkey, infos: &[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    if *id != SPL_TOKEN_PROGRAM_ID || data.len() != 9 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let amount = get_u64(data, 1)?;
    match data[0] {
        1 => {
            if !(infos.len() == 1 || infos.len() == 2) || infos[0].owner != id {
                return Err(ProgramError::InvalidAccountData);
            }
            let reserve = native_reserve(&infos[0])?;
            let observed = infos[0]
                .lamports()
                .checked_sub(reserve)
                .ok_or(ProgramError::InsufficientFunds)?;
            put_u64(&mut infos[0].try_borrow_mut_data()?, 64, observed)?;
        }
        2 => {
            if infos.len() != 3
                || infos[0].owner != id
                || infos[1].owner != id
                || !infos[2].is_signer
            {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let source = infos[0].try_borrow_data()?;
            let dest = infos[1].try_borrow_data()?;
            if &source[32..64] != infos[2].key.as_ref() || source[..32] != dest[..32] {
                return Err(ProgramError::InvalidAccountData);
            }
            let source_native = source[109..113] == 1u32.to_le_bytes();
            let dest_native = dest[109..113] == 1u32.to_le_bytes();
            if source_native != dest_native {
                return Err(ProgramError::InvalidAccountData);
            }
            let source_after = get_u64(&source, 64)?
                .checked_sub(amount)
                .ok_or(ProgramError::InsufficientFunds)?;
            let dest_after = get_u64(&dest, 64)?
                .checked_add(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            drop((source, dest));
            put_u64(&mut infos[0].try_borrow_mut_data()?, 64, source_after)?;
            put_u64(&mut infos[1].try_borrow_mut_data()?, 64, dest_after)?;
            if source_native {
                **infos[0].try_borrow_mut_lamports()? = infos[0]
                    .lamports()
                    .checked_sub(amount)
                    .ok_or(ProgramError::InsufficientFunds)?;
                **infos[1].try_borrow_mut_lamports()? = infos[1]
                    .lamports()
                    .checked_add(amount)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
            }
        }
        3 => {
            if infos.len() != 3
                || infos[0].owner != id
                || infos[1].owner != id
                || !infos[2].is_signer
                || infos[1].try_borrow_data()?[..32] != infos[0].key.to_bytes()
            {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let after = token_amount(&infos[1])?
                .checked_add(amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            put_u64(&mut infos[1].try_borrow_mut_data()?, 64, after)?;
            let supply = get_u64(&infos[0].try_borrow_data()?, 36)?;
            put_u64(
                &mut infos[0].try_borrow_mut_data()?,
                36,
                supply
                    .checked_add(amount)
                    .ok_or(ProgramError::ArithmeticOverflow)?,
            )?;
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}

fn sync_native<'a>(
    info: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    changed_counterparty: Option<&AccountInfo<'a>>,
) -> ProgramResult {
    let mut metas = vec![AccountMeta::new(*info.key, false)];
    let mut infos = vec![info.clone(), token_program.clone()];
    if let Some(other) = changed_counterparty {
        // ProgramTest's native CPI shim copies only inner instruction metas.
        // Include the curve whose lamports were debited directly by Beam so
        // this test-only sync sees the conserved pair of balance changes.
        metas.push(AccountMeta::new(*other.key, false));
        infos.push(other.clone());
    }
    invoke(&token_ix(1, metas, 0), &infos)
}

fn transfer_tokens<'a>(
    source: &AccountInfo<'a>,
    dest: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    let ix = token_ix(
        2,
        vec![
            AccountMeta::new(*source.key, false),
            AccountMeta::new(*dest.key, false),
            AccountMeta::new_readonly(*authority.key, true),
        ],
        amount,
    );
    invoke(
        &ix,
        &[
            source.clone(),
            dest.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )
}

fn transfer_curve_base<'a>(infos: &[AccountInfo<'a>], amount: u64) -> ProgramResult {
    let ix = token_ix(
        2,
        vec![
            AccountMeta::new(*infos[24].key, false),
            AccountMeta::new(*infos[16].key, false),
            AccountMeta::new_readonly(*infos[20].key, true),
        ],
        amount,
    );
    let (_, bump) = curve_pda();
    invoke_signed(
        &ix,
        &[
            infos[24].clone(),
            infos[16].clone(),
            infos[20].clone(),
            infos[0].clone(),
        ],
        &[&[CURVE_SEED, BEAM_MINT.as_ref(), &[bump]]],
    )
}

// Test-only mock registered at the *official devnet ID* inside ProgramTest.
fn mock_amm_v4<'a>(id: &Pubkey, infos: &[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    if *id != DEVNET_PROGRAM_ID
        || infos.len() != 19
        || data.len() != 26
        || !infos[15].is_signer
        || data[0] != 1
    {
        return Err(ProgramError::InvalidInstructionData);
    }
    let base = get_u64(data, 18)?;
    let quote = get_u64(data, 10)?;
    transfer_tokens(&infos[16], &infos[9], &infos[15], &infos[0], base)?;
    transfer_tokens(&infos[17], &infos[10], &infos[15], &infos[0], quote)?;
    let lp = expected_initial_user_lp(base, quote, 6)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let lp_to_mint = if get_u64(data, 2)? == 3 { lp - 1 } else { lp };
    let mint = token_ix(
        3,
        vec![
            AccountMeta::new(*infos[6].key, false),
            AccountMeta::new(*infos[18].key, false),
            AccountMeta::new_readonly(*infos[5].key, true),
        ],
        lp_to_mint,
    );
    let (_, authority_bump) = Pubkey::find_program_address(&[b"amm authority"], id);
    invoke_signed(
        &mint,
        &[
            infos[6].clone(),
            infos[18].clone(),
            infos[5].clone(),
            infos[0].clone(),
        ],
        &[&[b"amm authority", &[authority_bump]]],
    )?;
    let fee = get_u64(&infos[12].try_borrow_data()?, 536)?;
    let payment = system_instruction::transfer(infos[15].key, infos[13].key, fee);
    invoke(
        &payment,
        &[infos[15].clone(), infos[13].clone(), infos[2].clone()],
    )?;
    sync_native(&infos[13], &infos[0], None)?;
    infos[4].try_borrow_mut_data()?[0] = 1;
    if get_u64(data, 2)? == 2 {
        return Err(ProgramError::Custom(MOCK_LATE_ERROR));
    }
    Ok(())
}

fn observed_keys(infos: &[AccountInfo<'_>]) -> SettlementKeys {
    SettlementKeys {
        curve: *infos[20].key,
        setup_authority: *infos[15].key,
        migration_fee_recipient: *infos[21].key,
        curve_base_vault: *infos[24].key,
        base_source: *infos[16].key,
        wsol_source: *infos[17].key,
        base_vault: *infos[9].key,
        wsol_vault: *infos[10].key,
        lp_destination: *infos[18].key,
        create_fee_destination: *infos[13].key,
    }
}

fn observation(infos: &[AccountInfo<'_>]) -> Result<Observation, ProgramError> {
    Ok(Observation {
        keys: observed_keys(infos),
        balances: Balances {
            curve_lamports: infos[20].lamports(),
            setup_lamports: infos[15].lamports(),
            migration_fee_recipient_lamports: infos[21].lamports(),
            curve_base_vault_tokens: token_amount(&infos[24])?,
            base_source_tokens: token_amount(&infos[16])?,
            wsol_source_tokens: token_amount(&infos[17])?,
            wsol_source_lamports: infos[17].lamports(),
            wsol_source_rent_reserve: native_reserve(&infos[17])?,
            base_vault_tokens: token_amount(&infos[9])?,
            wsol_vault_tokens: token_amount(&infos[10])?,
            wsol_vault_lamports: infos[10].lamports(),
            wsol_vault_rent_reserve: native_reserve(&infos[10])?,
            lp_destination_tokens: token_amount(&infos[18])?,
            create_fee_destination_tokens: token_amount(&infos[13])?,
            create_fee_destination_lamports: infos[13].lamports(),
            create_fee_destination_rent_reserve: native_reserve(&infos[13])?,
            receipt_written: infos[22].try_borrow_data()?[0] == 1,
        },
    })
}

// Registered at Beam's ID locally, but deliberately NOT Beam's entrypoint.
// All account writes below are synthetic and the real migration gate stays off.
fn test_only_beam_caller<'a>(id: &Pubkey, infos: &[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    if *id != beam_program::ID || infos.len() != 25 || data.len() != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if infos[22].try_borrow_data()?[0] != 0 {
        return Err(ProgramError::Custom(RECEIPT_REPLAY));
    }
    let mode = data[0];
    // First reject aliases even if the supplied key is otherwise plausible;
    // then bind it to the independently stored synthetic policy recipient.
    expected_keys(
        &DEVNET_PROGRAM_ID,
        &BEAM_MINT,
        infos[14].key,
        *infos[20].key,
        *infos[21].key,
    )
    .map_err(|_| ProgramError::InvalidAccountData)?;
    let expected = expected_keys(
        &DEVNET_PROGRAM_ID,
        &BEAM_MINT,
        infos[14].key,
        *infos[20].key,
        STORED_MIGRATION_RECIPIENT,
    )
    .map_err(|_| ProgramError::InvalidAccountData)?;
    if observed_keys(infos) != expected || infos[20].key != &curve_pda().0 {
        return Err(ProgramError::InvalidAccountData);
    }
    let current_fee = validate_create_pool_fee_account(&DEVNET_PROGRAM_ID, &infos[12], CREATE_FEE)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let input = FundingInputs {
        quote_asset: QuoteAsset::NativeSol,
        base_mint_program: BaseMintProgram::ClassicSpl,
        base_tokens_to_pool: BASE_AMOUNT,
        curve_lamports: infos[20].lamports(),
        curve_rent_floor: RENT_RESERVE,
        real_quote_reserves: 1_500_000,
        post_complete_quote_in: 500_000,
        creator_fee_liability: 100_000,
        protocol_fee_liability: 100_000,
        migration_fee: MIGRATION_FEE,
        caller_setup_budget: SETUP_BUDGET,
        setup_authority_before: infos[15].lamports(),
        current_create_pool_fee: current_fee,
        max_create_pool_fee: CREATE_FEE,
        setup_rent_cap: 0,
    };
    let funding = plan(input).map_err(|_| ProgramError::InvalidAccountData)?;
    if funding.pool_quote != POOL_QUOTE {
        return Err(ProgramError::InvalidAccountData);
    }
    let initial = observation(infos)?;

    // Synthetic staging: Beam-owned curve base ATA -> setup ATA, curve SOL ->
    // setup WSOL ATA, then caller's separate setup budget -> signer PDA.
    transfer_curve_base(infos, BASE_AMOUNT)?;
    let curve_after = infos[20]
        .lamports()
        .checked_sub(funding.pool_quote)
        .ok_or(ProgramError::InsufficientFunds)?;
    let wsol_after = infos[17]
        .lamports()
        .checked_add(funding.pool_quote)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **infos[20].try_borrow_mut_lamports()? = curve_after;
    **infos[17].try_borrow_mut_lamports()? = wsol_after;
    sync_native(&infos[17], &infos[0], Some(&infos[20]))?;
    let setup = system_instruction::transfer(infos[23].key, infos[15].key, SETUP_BUDGET);
    invoke(
        &setup,
        &[infos[23].clone(), infos[15].clone(), infos[2].clone()],
    )?;
    let before_cpi = observation(infos)?;
    let args = Initialize2Args {
        open_time: u64::from(mode),
        init_pc_amount: funding.pool_quote,
        init_coin_amount: BASE_AMOUNT,
    };
    let actual =
        preflight_native_sol_initialize2(&infos[19], &infos[..19], &BEAM_MINT, args, CREATE_FEE)
            .map_err(|_| ProgramError::InvalidAccountData)?;
    let ix = initialize2_instruction(DEVNET_PROGRAM_ID, &actual, args)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let (_, bump) = setup_pda();
    invoke_signed(
        &ix,
        infos,
        &[&[b"pool-authority", BEAM_MINT.as_ref(), &[bump]]],
    )?;

    // Synthetic migration fee and receipt. A real adapter still needs an
    // authorized recipient, pool state decoding and actual receipt account.
    let curve_after_fee = infos[20]
        .lamports()
        .checked_sub(MIGRATION_FEE)
        .ok_or(ProgramError::InsufficientFunds)?;
    let recipient_after = infos[21]
        .lamports()
        .checked_add(MIGRATION_FEE)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **infos[20].try_borrow_mut_lamports()? = curve_after_fee;
    **infos[21].try_borrow_mut_lamports()? = recipient_after;
    infos[22].try_borrow_mut_data()?[0] = 1;
    let after_cpi = observation(infos)?;
    let settled = verify(
        expected,
        funding,
        BASE_AMOUNT,
        6,
        initial,
        before_cpi,
        after_cpi,
    )
    .map_err(|_| ProgramError::InvalidAccountData)?;
    let refund = system_instruction::transfer(infos[15].key, infos[23].key, settled.caller_refund);
    invoke_signed(
        &refund,
        &[infos[15].clone(), infos[23].clone(), infos[2].clone()],
        &[&[b"pool-authority", BEAM_MINT.as_ref(), &[bump]]],
    )?;
    Ok(())
}

struct Fixture {
    bank: ProgramTestContext,
    ix: Instruction,
    curve: Pubkey,
    setup: Pubkey,
    recipient: Pubkey,
    curve_base: Pubkey,
    base_source: Pubkey,
    wsol_source: Pubkey,
    base_vault: Pubkey,
    wsol_vault: Pubkey,
    lp_destination: Pubkey,
    fee_destination: Pubkey,
    amm: Pubkey,
    receipt: Pubkey,
    rogue_fee: Pubkey,
    rogue_migration_recipient: Pubkey,
}

fn account(owner: Pubkey, lamports: u64, data: Vec<u8>, executable: bool) -> Account {
    Account {
        lamports,
        data,
        owner,
        executable,
        rent_epoch: 0,
    }
}

async fn fixture(extra_base: u64, extra_wsol_lamports: u64) -> Fixture {
    let setup = setup_pda().0;
    let curve = curve_pda().0;
    let market_seed = Pubkey::new_unique();
    let p = derive_pool_addresses(&DEVNET_PROGRAM_ID, &market_seed).unwrap();
    let accounts = Initialize2Accounts {
        coin_mint: MintKey {
            key: BEAM_MINT,
            owner: SPL_TOKEN_PROGRAM_ID,
        },
        pc_mint: MintKey {
            key: WSOL_MINT,
            owner: SPL_TOKEN_PROGRAM_ID,
        },
        market_seed,
        user_wallet: setup,
        user_coin_source: user_lp_ata(&setup, &BEAM_MINT),
        user_pc_source: user_lp_ata(&setup, &WSOL_MINT),
        user_lp_ata: user_lp_ata(&setup, &p.lp_mint),
    };
    let built = initialize2_instruction(
        DEVNET_PROGRAM_ID,
        &accounts,
        Initialize2Args {
            open_time: 0,
            init_pc_amount: POOL_QUOTE,
            init_coin_amount: BASE_AMOUNT,
        },
    )
    .unwrap();
    let recipient = STORED_MIGRATION_RECIPIENT;
    let receipt = Pubkey::new_unique();
    let rogue_fee = Pubkey::new_unique();
    let rogue_migration_recipient = ROGUE_MIGRATION_RECIPIENT;
    let curve_base = user_lp_ata(&curve, &BEAM_MINT);
    let mut bank = ProgramTest::new(
        "test_only_beam_migration",
        beam_program::ID,
        processor!(test_only_beam_caller),
    );
    bank.add_program(
        "test_only_spl_token",
        SPL_TOKEN_PROGRAM_ID,
        processor!(mock_token),
    );
    bank.add_program(
        "test_only_amm_v4",
        DEVNET_PROGRAM_ID,
        processor!(mock_amm_v4),
    );
    for (i, meta) in built.accounts.iter().enumerate() {
        if matches!(i, 0 | 2 | 3) {
            continue;
        } // registered SPL mock and native programs/sysvar
        let acc = match i {
            1 => account(bpf_loader_upgradeable::ID, 1_000_000, Vec::new(), true),
            4 => account(DEVNET_PROGRAM_ID, 1_000_000, vec![0; 8], false),
            6 => account(SPL_TOKEN_PROGRAM_ID, RENT_RESERVE, mint_bytes(9), false),
            7 => account(SPL_TOKEN_PROGRAM_ID, RENT_RESERVE, mint_bytes(6), false),
            8 => account(SPL_TOKEN_PROGRAM_ID, RENT_RESERVE, mint_bytes(9), false),
            9 => account(
                SPL_TOKEN_PROGRAM_ID,
                RENT_RESERVE,
                token_bytes(BEAM_MINT, p.authority, 0, false),
                false,
            ),
            10 => account(
                SPL_TOKEN_PROGRAM_ID,
                RENT_RESERVE,
                token_bytes(WSOL_MINT, p.authority, 0, true),
                false,
            ),
            12 => {
                let mut data = vec![0; AMM_CONFIG_DATA_LEN];
                data[536..544].copy_from_slice(&CREATE_FEE.to_le_bytes());
                account(DEVNET_PROGRAM_ID, 1_000_000, data, false)
            }
            13 => account(
                SPL_TOKEN_PROGRAM_ID,
                FEE_DEST_START,
                token_bytes(WSOL_MINT, Pubkey::new_unique(), 0, true),
                false,
            ),
            15 => account(system_program::ID, SETUP_START, Vec::new(), false),
            16 => account(
                SPL_TOKEN_PROGRAM_ID,
                RENT_RESERVE,
                token_bytes(BEAM_MINT, setup, 0, false),
                false,
            ),
            17 => account(
                SPL_TOKEN_PROGRAM_ID,
                RENT_RESERVE + extra_wsol_lamports,
                token_bytes(WSOL_MINT, setup, 0, true),
                false,
            ),
            18 => account(
                SPL_TOKEN_PROGRAM_ID,
                RENT_RESERVE,
                token_bytes(p.lp_mint, setup, 0, false),
                false,
            ),
            _ => account(system_program::ID, 1_000_000, Vec::new(), false),
        };
        bank.add_account(meta.pubkey, acc);
    }
    bank.add_account(
        curve,
        account(beam_program::ID, CURVE_START, vec![0], false),
    );
    bank.add_account(
        recipient,
        account(
            system_program::ID,
            MIGRATION_RECIPIENT_START,
            Vec::new(),
            false,
        ),
    );
    bank.add_account(
        rogue_migration_recipient,
        account(
            system_program::ID,
            MIGRATION_RECIPIENT_START,
            Vec::new(),
            false,
        ),
    );
    bank.add_account(
        receipt,
        account(beam_program::ID, 1_000_000, vec![0], false),
    );
    bank.add_account(
        curve_base,
        account(
            SPL_TOKEN_PROGRAM_ID,
            RENT_RESERVE,
            token_bytes(BEAM_MINT, curve, BASE_AMOUNT + extra_base, false),
            false,
        ),
    );
    bank.add_account(
        rogue_fee,
        account(
            SPL_TOKEN_PROGRAM_ID,
            RENT_RESERVE,
            token_bytes(WSOL_MINT, Pubkey::new_unique(), 0, true),
            false,
        ),
    );
    let mut ix = Instruction {
        program_id: beam_program::ID,
        accounts: built.accounts,
        data: vec![0],
    };
    ix.accounts[15].is_signer = false; // Beam's PDA signs only at the inner CPI
    ix.accounts.extend([
        AccountMeta::new_readonly(DEVNET_PROGRAM_ID, false),
        AccountMeta::new(curve, false),
        AccountMeta::new(recipient, false),
        AccountMeta::new(receipt, false),
        AccountMeta::new(Pubkey::default(), true), // replaced with bank payer below
        AccountMeta::new(curve_base, false),
    ]);
    let context = bank.start_with_context().await;
    ix.accounts[23].pubkey = context.payer.pubkey();
    Fixture {
        bank: context,
        ix,
        curve,
        setup,
        recipient,
        curve_base,
        base_source: accounts.user_coin_source,
        wsol_source: accounts.user_pc_source,
        base_vault: p.coin_vault,
        wsol_vault: p.pc_vault,
        lp_destination: accounts.user_lp_ata,
        fee_destination: p.create_pool_fee_destination,
        amm: p.amm,
        receipt,
        rogue_fee,
        rogue_migration_recipient,
    }
}

fn tx(
    f: &Fixture,
    mode: u8,
    wrong_create_fee_recipient: bool,
    alias_recipient: bool,
    wrong_migration_recipient: bool,
) -> Transaction {
    let mut ix = f.ix.clone();
    ix.data[0] = mode;
    if wrong_create_fee_recipient {
        ix.accounts[13].pubkey = f.rogue_fee;
    }
    if alias_recipient {
        ix.accounts[21].pubkey = f.curve;
    } else if wrong_migration_recipient {
        ix.accounts[21].pubkey = f.rogue_migration_recipient;
    }
    Transaction::new_signed_with_payer(
        &[ix],
        Some(&f.bank.payer.pubkey()),
        &[&f.bank.payer],
        f.bank.last_blockhash,
    )
}

async fn state(f: &Fixture) -> Vec<Account> {
    let keys = [
        f.curve,
        f.setup,
        f.recipient,
        f.curve_base,
        f.base_source,
        f.wsol_source,
        f.base_vault,
        f.wsol_vault,
        f.lp_destination,
        f.fee_destination,
        f.amm,
        f.receipt,
        f.rogue_fee,
        f.rogue_migration_recipient,
    ];
    let mut out = Vec::with_capacity(keys.len());
    for key in keys {
        out.push(f.bank.banks_client.get_account(key).await.unwrap().unwrap());
    }
    out
}

fn amount(account: &Account) -> u64 {
    get_u64(&account.data, 64).unwrap()
}

#[tokio::test]
async fn beam_preflight_funding_and_recipient_bound_mock_settlement() {
    let f = fixture(0, 0).await;
    f.bank
        .banks_client
        .process_transaction(tx(&f, 0, false, false, false))
        .await
        .unwrap();
    let after = state(&f).await;
    let lp = expected_initial_user_lp(BASE_AMOUNT, POOL_QUOTE, 6).unwrap();
    assert_eq!(after[0].lamports, CURVE_START - POOL_QUOTE - MIGRATION_FEE);
    assert_eq!(after[1].lamports, SETUP_START); // unspent caller budget refunded
    assert_eq!(after[2].lamports, MIGRATION_RECIPIENT_START + MIGRATION_FEE);
    assert_eq!(amount(&after[3]), 0); // curve base ATA
    assert_eq!(amount(&after[4]), 0); // setup base ATA
    assert_eq!(amount(&after[5]), 0); // setup WSOL ATA
    assert_eq!(after[5].lamports, RENT_RESERVE);
    assert_eq!(amount(&after[6]), BASE_AMOUNT); // pool base vault
    assert_eq!(amount(&after[7]), POOL_QUOTE); // pool WSOL vault
    assert_eq!(after[7].lamports, RENT_RESERVE + POOL_QUOTE);
    assert_eq!(amount(&after[8]), lp); // synthetic LP mint, no custody claim
    assert_eq!(amount(&after[9]), CREATE_FEE); // official fee destination WSOL ATA
    assert_eq!(after[9].lamports, FEE_DEST_START + CREATE_FEE);
    assert_eq!(after[10].data[0], 1); // test-only pool marker
    assert_eq!(after[11].data[0], 1); // test-only receipt marker

    let replay = f
        .bank
        .banks_client
        .process_transaction(tx(&f, 4, false, false, false))
        .await
        .unwrap_err();
    assert!(format!("{replay:?}").contains(&format!("Custom({RECEIPT_REPLAY})")));
    assert_eq!(state(&f).await, after);
}

#[tokio::test]
async fn recipient_alias_and_donation_rejections_preserve_every_fixture_account() {
    for (base_donation, wsol_donation, wrong_create_fee, alias_recipient, wrong_migration) in [
        (0, 0, true, false, false),
        (0, 0, false, true, false),
        (0, 0, false, false, true),
        (1, 0, false, false, false),
        (0, 1, false, false, false),
    ] {
        let f = fixture(base_donation, wsol_donation).await;
        let before = state(&f).await;
        let result = f
            .bank
            .banks_client
            .process_transaction(tx(
                &f,
                0,
                wrong_create_fee,
                alias_recipient,
                wrong_migration,
            ))
            .await;
        assert!(result.is_err(), "fault unexpectedly succeeded");
        assert_eq!(state(&f).await, before);
    }
}

#[tokio::test]
async fn late_mock_error_and_wrong_lp_delta_roll_back_funding_and_cpi() {
    for mode in [2, 3] {
        let f = fixture(0, 0).await;
        let before = state(&f).await;
        let error = f
            .bank
            .banks_client
            .process_transaction(tx(&f, mode, false, false, false))
            .await
            .unwrap_err();
        let expected = if mode == 2 {
            format!("Custom({MOCK_LATE_ERROR})")
        } else {
            "InvalidAccountData".into()
        };
        assert!(format!("{error:?}").contains(&expected), "{error:?}");
        assert_eq!(state(&f).await, before);
    }
}
