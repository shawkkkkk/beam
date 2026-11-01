use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_lang::prelude::Pubkey;
use beam_program::{accounts, instruction, OptionBool};

fn k(i: u8) -> Pubkey { Pubkey::new_from_array([i; 32]) }

fn print_fixture(name: &str, data: &[u8], metas: &[anchor_lang::solana_program::instruction::AccountMeta]) {
    println!("{name}.programId={}", beam_program::ID);
    print!("{name}.dataHex=");
    for byte in data { print!("{byte:02x}"); }
    println!();
    for (index, meta) in metas.iter().enumerate() {
        println!("{name}.meta={index},{},{},{}", meta.pubkey, meta.is_signer, meta.is_writable);
    }
}

fn main() {
    let buy = instruction::BuyV2 { amount: 42, max_sol_cost: 987_654_321, partial_fill: OptionBool(false) }.data();
    let buy_accounts = accounts::BuyV2 {
        global: k(1), base_mint: k(2), quote_mint: k(3),
        base_token_program: k(4), quote_token_program: k(5), associated_token_program: k(6),
        fee_recipient: k(7), associated_quote_fee_recipient: k(8),
        buyback_fee_recipient: k(9), associated_quote_buyback_fee_recipient: k(10),
        bonding_curve: k(11), associated_base_bonding_curve: k(12),
        associated_quote_bonding_curve: k(13), user: k(14),
        associated_base_user: k(15), associated_quote_user: k(16),
        creator_vault: k(17), associated_creator_vault: k(18),
        sharing_config: k(19), global_volume_accumulator: k(20),
        user_volume_accumulator: k(21), associated_user_volume_accumulator: k(22),
        fee_config: k(23), fee_program: k(24), system_program: k(25),
        event_authority: k(26), program: beam_program::ID,
    }.to_account_metas(None);
    print_fixture("buyV2", &buy, &buy_accounts);

    let sell = instruction::SellV2 { amount: 42, min_sol_output: 123_456_789 }.data();
    let sell_accounts = accounts::SellV2 {
        global: k(1), base_mint: k(2), quote_mint: k(3),
        base_token_program: k(4), quote_token_program: k(5), associated_token_program: k(6),
        fee_recipient: k(7), associated_quote_fee_recipient: k(8),
        buyback_fee_recipient: k(9), associated_quote_buyback_fee_recipient: k(10),
        bonding_curve: k(11), associated_base_bonding_curve: k(12),
        associated_quote_bonding_curve: k(13), user: k(14),
        associated_base_user: k(15), associated_quote_user: k(16),
        creator_vault: k(17), associated_creator_vault: k(18),
        sharing_config: k(19), user_volume_accumulator: k(20),
        associated_user_volume_accumulator: k(21), fee_config: k(22),
        fee_program: k(23), system_program: k(24),
        event_authority: k(25), program: beam_program::ID,
    }.to_account_metas(None);
    print_fixture("sellV2", &sell, &sell_accounts);
}
