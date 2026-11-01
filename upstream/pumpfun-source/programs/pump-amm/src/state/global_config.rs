use anchor_lang::prelude::*;
use crate::PumpAmmError;
use super::GlobalConfig;






















































































































































































































impl GlobalConfig {
    pub(crate) fn set_lp_fee_basis_points(&mut self, fee: u64) -> Result<()> {
        require_gte!(1000u64, fee, PumpAmmError::FeeBasisPointsExceedsMaximum);
        self.lp_fee_basis_points = fee;
        Ok(())
    }





    pub(crate) fn set_protocol_fee_basis_points(&mut self, fee: u64) -> Result<()> {
        require_gte!(1000u64, fee, PumpAmmError::FeeBasisPointsExceedsMaximum);
        self.protocol_fee_basis_points = fee;
        Ok(())
    }








    pub(crate) fn set_coin_creator_fee_basis_points(&mut self, fee: u64) -> Result<()> {
        require_gte!(1000u64, fee, PumpAmmError::FeeBasisPointsExceedsMaximum);
        self.coin_creator_fee_basis_points = fee;
        Ok(())
    }








    pub(crate) fn set_protocol_fee_recipients(&mut self, recipients: [Pubkey; 8]) -> Result<()> {
        require!(recipients.iter().all(|key| *key != Pubkey::default()),
            PumpAmmError::AllProtocolFeeRecipientsShouldBeNonZero);





        require!(recipients.windows(2).all(|pair| pair[0] < pair[1]),
            PumpAmmError::UnsortedNotUniqueProtocolFeeRecipients);
        self.protocol_fee_recipients = recipients;
        Ok(())
    }
}





















impl GlobalConfig {
    pub fn validate_boost_authority(&self, authority: &Pubkey) -> Result<()> {
        require_keys_neq!(self.boost_authority, Pubkey::default(), PumpAmmError::InvalidAdmin);




        require_keys_eq!(self.boost_authority, *authority, PumpAmmError::InvalidAdmin);
        Ok(())
    }
}



impl GlobalConfig {
    pub fn set_buyback_basis_points(&mut self, basis_points: u64) -> Result<()> {
        require!(basis_points <= 10_000, PumpAmmError::BuybackBasisPointsOutOfRange);
        self.buyback_basis_points = basis_points;
        Ok(())
    }
}


impl GlobalConfig {
    pub fn validate_buyback_recipient(
        &self,
        recipient: &Pubkey,
        token_account: &Pubkey,
        quote_mint: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<()> {
        require_keys_neq!(*recipient, Pubkey::default(), PumpAmmError::BuybackFeeRecipientNotAuthorized);




        require!(self.buyback_fee_recipients.contains(recipient), PumpAmmError::BuybackFeeRecipientNotAuthorized);
        let expected = anchor_spl::associated_token::get_associated_token_address_with_program_id(
            recipient, quote_mint, token_program,
        );






        require!(expected == *token_account, PumpAmmError::BuybackFeeRecipientNotAuthorized);
        Ok(())
    }







    pub fn set_buyback_fee_recipients(&mut self, recipients: [Pubkey; 8]) -> Result<()> {
        require!(recipients.iter().all(|recipient| *recipient != Pubkey::default()),
            PumpAmmError::AllBuybackFeeRecipientsShouldBeNonZero);
        for i in 0..8 {
            for j in i + 1..8 {






                require_neq!(recipients[i], recipients[j], PumpAmmError::NotUniqueBuybackFeeRecipients);
            }
        }
        self.buyback_fee_recipients = recipients;
        Ok(())
    }
}
















impl GlobalConfig {
    pub fn split_buyback_fee(buyback_basis_points: u64, protocol_fee: u64, is_mayhem_mode: bool) -> (u64, u64) {
        if is_mayhem_mode {
            return (0, protocol_fee);
        }
        let raw = (protocol_fee as u128)
            * (buyback_basis_points as u128)
            / 10_000;
        let fee_amount = u64::try_from(raw).expect("buyback fee is at most protocol_fee");
        let net_protocol = protocol_fee
            .checked_sub(fee_amount)
            .expect("buyback_fee never exceeds protocol_fee");
        (fee_amount, net_protocol)
    }
    pub fn split_buyback(&self, protocol_fee: u64, is_mayhem_mode: bool) -> (u64, u64) {
        Self::split_buyback_fee(self.buyback_basis_points, protocol_fee, is_mayhem_mode)
    }
}
