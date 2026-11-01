use anchor_lang::prelude::*;
#[derive(Clone)]
pub struct AccountWrapper<T>(pub T);
impl<T> std::ops::Deref for AccountWrapper<T>{type Target=T;fn deref(&self)->&T{&self.0}}
impl<T> std::ops::DerefMut for AccountWrapper<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
impl<T:Owner> Owner for AccountWrapper<T>{fn owner()->Pubkey{T::owner()}}
impl<T:Discriminator> Discriminator for AccountWrapper<T>{const DISCRIMINATOR:&'static[u8]=T::DISCRIMINATOR;}
impl<T:AccountSerialize> AccountSerialize for AccountWrapper<T>{fn try_serialize<W:std::io::Write>(&self,writer:&mut W)->Result<()>{self.0.try_serialize(writer)}}
impl<T:AccountDeserialize+AccountSerialize+Default+crate::CheckedDeserialize> AccountDeserialize for AccountWrapper<T>{
 fn try_deserialize(buf:&mut &[u8])->Result<Self>{Self::try_deserialize_unchecked(buf)}












































































    fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{
        let mut default=Vec::new();T::default().try_serialize(&mut default)?;
        if buf.len()<default.len(){
        let mut padded = vec![0; default.len()];
        padded[..buf.len()].copy_from_slice(buf);
        let value=T::deserialize_checked(&mut padded.as_slice())?;
        *buf=&buf[buf.len()..];Ok(Self(value))
        }else{T::deserialize_checked(buf).map(Self)}
    }
}
