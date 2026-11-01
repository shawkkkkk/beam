use anchor_lang::prelude::*;



#[account]
pub struct SwapReceiptV1 {pub bump:u8,pub distribution:Pubkey,pub swap_output_amount:u64}
