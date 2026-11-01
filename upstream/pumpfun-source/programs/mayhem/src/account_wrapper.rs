use anchor_lang::prelude::*;
use crate::instructions::sell::SellCurve;
impl AccountDeserialize for SellCurve {
fn try_deserialize(buf:&mut &[u8])->Result<Self>{Self::try_deserialize_unchecked(buf)}


















































































    fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{
        let mut default=Vec::new();curve_accounts::BondingCurve::default().try_serialize(&mut default)?;
        if buf.len()<default.len(){
        let mut padded = vec![0; default.len()];
        padded[..buf.len()].copy_from_slice(buf);
        let value=curve_accounts::try_deserialize_curve(&mut padded.as_slice())?;
        *buf=&buf[buf.len()..];Ok(Self(value))
        }else{curve_accounts::try_deserialize_curve(buf).map(Self)}
    }
}
