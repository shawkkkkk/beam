use super::*;
use crate::PumpAmmError;

























































































































































































































































































































































































impl Pool {
    #[inline(never)]
    pub fn market_cap(&self, base_supply: u64, base_reserve: u64, quote_reserve: u64) -> Result<u128> {
        let supply = if self.is_mayhem_mode { 1_000_000_000_000_000 } else { base_supply };
        ((supply as u128) * (quote_reserve as u128))
            .checked_div(base_reserve as u128)
            .ok_or_else(|| error!(PumpAmmError::DivisionByZero))
    }
}














impl Pool {
    #[inline(never)]
    pub fn total_accrued_fees(&self) -> Result<u64> {
        self.protocol_fees
            .checked_add(self.creator_fees)
            .ok_or_else(|| error!(PumpAmmError::Overflow))
    }
}

impl Pool {
    #[inline(never)]
    pub fn available_quote_reserves(&self, quote_token_reserves: u64) -> Result<u64> {
        quote_token_reserves
            .checked_sub(self.total_accrued_fees()?)
            .ok_or_else(|| error!(PumpAmmError::Overflow))
    }



    #[inline(never)]
    pub fn total_virtual_quote_reserves(&self) -> Result<i128> {
        self.virtual_quote_reserves
            .checked_add(self.total_accrued_fees()? as i128)
            .ok_or_else(|| error!(PumpAmmError::Overflow))
    }

    pub fn can_boost(&self) -> bool {
        self.total_virtual_quote_reserves().unwrap_or_default() != 0
    }
}


impl Pool {
    #[inline(never)]
    pub fn effective_quote_reserves(&self, quote_token_reserves: u64) -> Result<u64> {
        let amount = (quote_token_reserves as i128).checked_add(self.virtual_quote_reserves)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
        u64::try_from(amount).map_err(|_| error!(PumpAmmError::Overflow))
    }
    #[inline(never)]
    pub fn accrue_fees(
        &mut self,
        global_config: &GlobalConfig,
        mut protocol_fee: u64,
        creator_fee: u64,
    ) -> Result<u64> {
        let (buyback_fee, net_protocol_fee) =
            global_config.split_buyback(protocol_fee, self.is_mayhem_mode);
        protocol_fee = net_protocol_fee;






        self.protocol_fees = self.protocol_fees.checked_add(protocol_fee).ok_or_else(|| error!(PumpAmmError::Overflow))?;
        self.creator_fees = self
            .creator_fees
            .checked_add(creator_fee)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
        let total = protocol_fee
            .checked_add(creator_fee)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
        self.virtual_quote_reserves = self
            .virtual_quote_reserves
            .checked_sub(total as i128)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
        Ok(buyback_fee)
    }
}

impl Pool {
    #[inline(never)]
    pub fn take_accrued_fees(&mut self, creator: bool) -> Result<u64> {
        let amount = core::mem::take(if creator {
            &mut self.creator_fees
        } else {
            &mut self.protocol_fees
        });
        self.virtual_quote_reserves = self.virtual_quote_reserves
            .checked_add(amount as i128)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
        Ok(amount)
    }
}
