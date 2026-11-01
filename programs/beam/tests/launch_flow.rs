use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::{associated_token, token};
use beam_core::{BondingCurve, FeePolicy, LaunchConfig};
use beam_launch::{accounts, instruction, CreateArgs, Launch};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

fn process_instruction<'info>(
    program_id: &anchor_lang::prelude::Pubkey,
    accounts: &[anchor_lang::prelude::AccountInfo<'info>],
    instruction_data: &[u8],
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    // Anchor's native entrypoint ties the slice and AccountInfo lifetimes
    // together. Clone the Rc-backed account handles for this test invocation.
    let copied = accounts.to_vec().into_boxed_slice();
    let anchored: &'info [anchor_lang::prelude::AccountInfo<'info>] = Box::leak(copied);
    beam_launch::entry(program_id, anchored, instruction_data)
}

#[tokio::test]
async fn create_buy_sell_and_reject_slippage() {
    let context = ProgramTest::new(
        "beam_launch",
        beam_launch::id(),
        processor!(process_instruction),
    )
    .start_with_context()
    .await;
    let creator = context.payer.pubkey();
    let mint = Keypair::new();
    let (launch, _) = anchor_lang::prelude::Pubkey::find_program_address(
        &[b"launch", mint.pubkey().as_ref()],
        &beam_launch::id(),
    );
    let vault = associated_token::get_associated_token_address(&launch, &mint.pubkey());
    let creator_tokens = associated_token::get_associated_token_address(&creator, &mint.pubkey());
    let args = CreateArgs {
        total_supply: 1_000_000_000,
        initial_real_token_reserves: 800_000_000,
        initial_virtual_token_reserves: 1_073_000_000,
        initial_virtual_quote_reserves: 30_000_000,
    };

    let create = Instruction {
        program_id: beam_launch::id(),
        accounts: accounts::Create {
            creator,
            launch,
            mint: mint.pubkey(),
            vault,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::Create { args }.data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[create],
        Some(&creator),
        &[&context.payer, &mint],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let mint_account = context
        .banks_client
        .get_account(mint.pubkey())
        .await
        .unwrap()
        .unwrap();
    let mint_state =
        <token::spl_token::state::Mint as anchor_lang::solana_program::program_pack::Pack>::unpack(
            &mint_account.data,
        )
        .unwrap();
    assert_eq!(mint_state.supply, args.total_supply);
    assert!(mint_state.mint_authority.is_none());
    let initial_vault = context
        .banks_client
        .get_account(vault)
        .await
        .unwrap()
        .unwrap();
    let initial_vault_state = <token::spl_token::state::Account as anchor_lang::solana_program::program_pack::Pack>::unpack(&initial_vault.data).unwrap();
    assert_eq!(initial_vault_state.amount, args.total_supply);

    let buy_accounts = accounts::Buy {
        buyer: creator,
        launch,
        mint: mint.pubkey(),
        vault,
        buyer_tokens: creator_tokens,
        token_program: token::ID,
        associated_token_program: associated_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    let buy_tokens = 100_000_000;
    let mut expected = BondingCurve::new(LaunchConfig {
        total_supply: args.total_supply,
        initial_real_token_reserves: args.initial_real_token_reserves,
        initial_virtual_token_reserves: args.initial_virtual_token_reserves,
        initial_virtual_quote_reserves: args.initial_virtual_quote_reserves,
        migration_fee: 0,
    })
    .unwrap();
    let buy_trade = expected
        .buy(buy_tokens, u64::MAX, FeePolicy::default())
        .unwrap();
    let buy = Instruction {
        program_id: beam_launch::id(),
        accounts: buy_accounts.clone(),
        data: instruction::Buy {
            tokens: buy_tokens,
            max_lamports: buy_trade.trader_quote,
        }
        .data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[buy],
        Some(&creator),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let launch_account = context
        .banks_client
        .get_account(launch)
        .await
        .unwrap()
        .unwrap();
    let state = Launch::try_deserialize(&mut &launch_account.data[..]).unwrap();
    assert_eq!(state.real_token_reserves, expected.real_token_reserves);
    assert_eq!(state.real_quote_reserves, buy_trade.quote);
    assert_eq!(state.rewards_accrued, buy_trade.fees.rewards);
    assert_eq!(state.treasury_accrued, buy_trade.fees.treasury);
    assert_eq!(state.founder_accrued, buy_trade.fees.founder);

    let bad_buy = Instruction {
        program_id: beam_launch::id(),
        accounts: buy_accounts.clone(),
        data: instruction::Buy {
            tokens: buy_tokens,
            max_lamports: 1,
        }
        .data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[bad_buy],
        Some(&creator),
        &[&context.payer],
        context.last_blockhash,
    );
    assert!(context
        .banks_client
        .process_transaction(transaction)
        .await
        .is_err());
    let unchanged = context
        .banks_client
        .get_account(launch)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.data, launch_account.data);

    let sell_tokens = 40_000_000;
    let sell_trade = expected.sell(sell_tokens, 0, FeePolicy::default()).unwrap();
    let sell = Instruction {
        program_id: beam_launch::id(),
        accounts: accounts::Sell {
            seller: creator,
            launch,
            mint: mint.pubkey(),
            vault,
            seller_tokens: creator_tokens,
            token_program: token::ID,
        }
        .to_account_metas(None),
        data: instruction::Sell {
            tokens: sell_tokens,
            min_lamports: sell_trade.trader_quote,
        }
        .data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[sell],
        Some(&creator),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let launch_account = context
        .banks_client
        .get_account(launch)
        .await
        .unwrap()
        .unwrap();
    let state = Launch::try_deserialize(&mut &launch_account.data[..]).unwrap();
    assert_eq!(state.real_token_reserves, expected.real_token_reserves);
    assert_eq!(state.real_quote_reserves, expected.real_quote_reserves);
    assert_eq!(
        state.rewards_accrued,
        buy_trade.fees.rewards + sell_trade.fees.rewards
    );
    assert_eq!(
        state.treasury_accrued,
        buy_trade.fees.treasury + sell_trade.fees.treasury
    );
    assert_eq!(
        state.founder_accrued,
        buy_trade.fees.founder + sell_trade.fees.founder
    );
    let rent = context.banks_client.get_rent().await.unwrap();
    assert_eq!(
        launch_account.lamports,
        rent.minimum_balance(launch_account.data.len())
            + state.real_quote_reserves
            + state.rewards_accrued
            + state.treasury_accrued
            + state.founder_accrued
    );
    let user_token_account = context
        .banks_client
        .get_account(creator_tokens)
        .await
        .unwrap()
        .unwrap();
    let user_tokens = <token::spl_token::state::Account as anchor_lang::solana_program::program_pack::Pack>::unpack(&user_token_account.data).unwrap();
    assert_eq!(user_tokens.amount, buy_tokens - sell_tokens);

    let final_tokens = expected.real_token_reserves;
    let final_buy = expected
        .buy(final_tokens, u64::MAX, FeePolicy::default())
        .unwrap();
    let complete = Instruction {
        program_id: beam_launch::id(),
        accounts: buy_accounts,
        data: instruction::Buy {
            tokens: final_tokens,
            max_lamports: final_buy.trader_quote,
        }
        .data(),
    };
    let transaction = Transaction::new_signed_with_payer(
        &[complete],
        Some(&creator),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
    let final_account = context
        .banks_client
        .get_account(launch)
        .await
        .unwrap()
        .unwrap();
    let final_state = Launch::try_deserialize(&mut &final_account.data[..]).unwrap();
    assert!(final_state.complete);
    assert_eq!(final_state.real_token_reserves, 0);
    assert_eq!(
        final_state.reserved_pool_tokens,
        args.total_supply - args.initial_real_token_reserves
    );
    let final_vault = context
        .banks_client
        .get_account(vault)
        .await
        .unwrap()
        .unwrap();
    let final_vault_state = <token::spl_token::state::Account as anchor_lang::solana_program::program_pack::Pack>::unpack(&final_vault.data).unwrap();
    assert_eq!(final_vault_state.amount, final_state.reserved_pool_tokens);
}
