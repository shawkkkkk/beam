//! Typed Relay CPI adapter reconstructed from the saved IDL and captured static ABI.
//! Field names/signers/discriminator come from the IDL; source/type identity is inferred.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::{AccountMeta,Instruction},program::invoke_signed};

#[derive(AnchorSerialize,AnchorDeserialize)]
pub struct DonatePubkeyConfigIdWithPayerV1Args {
    pub amount:u64,
    pub config_id:Pubkey,
    pub tip_bps:u16,
    pub message:String,
    pub credited_to:Pubkey,
}
impl DonatePubkeyConfigIdWithPayerV1Args {
    pub fn data(&self)->Result<Vec<u8>> {
        let mut data=Vec::new();
        data.extend_from_slice(&[120,217,57,241,135,104,139,184]);
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub struct DonatePubkeyConfigIdWithPayerV1<'info> {
    pub program:AccountInfo<'info>,
    pub epoch_tracker:AccountInfo<'info>,
    pub debouncer:AccountInfo<'info>,
    pub debouncer_token_account:AccountInfo<'info>,
    pub mint:AccountInfo<'info>,
    pub from_token_account:AccountInfo<'info>,
    pub from:AccountInfo<'info>,
    pub payer:AccountInfo<'info>,
    pub mint_whitelist:AccountInfo<'info>,
    pub associated_token_program:AccountInfo<'info>,
    pub token_program:AccountInfo<'info>,
    pub system_program:AccountInfo<'info>,
    pub event_authority:AccountInfo<'info>,
}
impl<'info> DonatePubkeyConfigIdWithPayerV1<'info> {
    pub fn to_account_infos(&self)->[AccountInfo<'info>;13] {
        [
            self.epoch_tracker.clone(),
            self.debouncer.clone(),
            self.debouncer_token_account.clone(),
            self.mint.clone(),
            self.from_token_account.clone(),
            self.from.clone(),
            self.payer.clone(),
            self.mint_whitelist.clone(),
            self.associated_token_program.clone(),
            self.token_program.clone(),
            self.system_program.clone(),
            self.event_authority.clone(),
            self.program.clone(),
        ]
    }
    pub fn invoke_signed(self,args:DonatePubkeyConfigIdWithPayerV1Args,seeds:&[&[u8]])->Result<()> {
        let data=args.data()?;
        let ix=Instruction {
            program_id:crate::donation::RELAY,
            accounts:[
                AccountMeta::new(self.epoch_tracker.key(),false),
                AccountMeta::new(self.debouncer.key(),false),
                AccountMeta::new(self.debouncer_token_account.key(),false),
                AccountMeta::new(self.mint.key(),false),
                AccountMeta::new(self.from_token_account.key(),false),
                AccountMeta::new(self.from.key(),true),
                AccountMeta::new(self.payer.key(),true),
                AccountMeta::new_readonly(self.mint_whitelist.key(),false),
                AccountMeta::new_readonly(self.associated_token_program.key(),false),
                AccountMeta::new_readonly(self.token_program.key(),false),
                AccountMeta::new_readonly(self.system_program.key(),false),
                AccountMeta::new_readonly(self.event_authority.key(),false),
                AccountMeta::new_readonly(self.program.key(),false),
            ].to_vec(),
            data,
        };
        let signer_seeds=&[seeds];
        invoke_signed(&ix,&self.to_account_infos(),signer_seeds).map_err(Into::into)
    }
}
