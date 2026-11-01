use anchor_lang::prelude::*;
use crate::GlobalParams;






#[derive(Accounts)]
pub struct InitializeGlobalParams<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 254, seeds = [b"global-params"], bump)]
    pub global_params: Account<'info, GlobalParams>,
    pub system_program: Program<'info,System>,
}








#[event(discriminator = [0x63,0xa7,0x5c,0x98,0x98,0xe3,0x26,0x29])]
pub struct GlobalParamsUpdateEvent {
    pub old_version: u32,
    pub new_version: u32,
    pub field_0: u32,
    pub field_1: u32,
    pub field_2: u64,
    pub field_3: u64,
    pub field_4: u64,
    pub field_5: u64,
    pub field_6: u8,
    pub field_7: u64,
    pub field_8: u64,
    pub field_9: u64,
    pub field_10: u64,
    pub field_11: u64,
    pub field_12: u64,
    pub field_13: u64,
    pub field_14: u64,
    pub field_15: u64,
}
