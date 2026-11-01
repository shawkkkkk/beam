//! External runtime/token decode primitives only. No donate/swap/distribute hooks.
use crate::*;
pub trait Runtime: crate::generated_exit::AccountRuntime<A=Account> {
 fn with_mint_data<T,F>(&mut self,a:Account,f:F)->R<T> where F:FnOnce(&[u8])->R<T>;
 // Immutable borrow() held through f; conflict panics, release on all returns.
 fn mint_decode_error(&self,e:crate::mint_policy::DecodeError)->Error;
 // Framework close282 primitives preserve order, borrow failures and aliases.
 fn lamports(&mut self,a:Account)->R<u64>;
 fn set_lamports(&mut self,a:Account,value:u64)->R<()>;
 fn assign(&mut self,a:Account,owner:Key)->R<()>;
 fn realloc_zero(&mut self,a:Account)->R<()>;
 fn invoke_signed(&mut self, ix:Cpi, infos:&[Account], seeds:&Seeds)->R<()>;
 // Helpers 8/21/110/111: supported token owner/base/extension decoding and borrow
 // errors, NOT application mint-safety policy 106. Reload observes current data.
 fn with_account_data<T,F>(&mut self,a:Account,f:F)->R<T> where F:FnOnce(&[u8])->R<T>;
 // Fallible immutable borrow; unlike with_mint_data's panicking borrow().
 fn unpack_token(data:&[u8])->R<(Key,Key,u64)>;
 fn unpack_mint(data:&[u8])->R<u8>;
 fn token(&mut self,mut account:Account)->R<TokenView> {
  account.owner=crate::generated_exit::AccountRuntime::owner(self,&account);
  if account.owner==[0;32] && self.lamports_panicking(&account)==0 {return Err(Error::Code(3012));}
  require(account.owner==crate::codec::constants::TOKEN || account.owner==crate::codec::constants::TOKEN2022,3007)?;
  let (mint,authority,amount)=self.with_account_data(account,Self::unpack_token)?;
  Ok(TokenView{account,mint,authority,amount})
 }
 fn mint(&mut self,mut account:Account)->R<MintView> {
  account.owner=crate::generated_exit::AccountRuntime::owner(self,&account);
  if account.owner==[0;32] && self.lamports_panicking(&account)==0 {return Err(Error::Code(3012));}
  require(account.owner==crate::codec::constants::TOKEN || account.owner==crate::codec::constants::TOKEN2022,3007)?;
  let decimals=self.with_account_data(account,Self::unpack_mint)?;
  Ok(MintView{account,decimals})
 }
 fn log_data(&mut self, data:&[u8])->R<()>;
 fn set_return_data(&mut self, data:&[u8])->R<()>;
}
fn meta(a:Account,writable:bool,signer:bool)->Meta {Meta{key:a.key,writable,signer}}
pub fn transfer_checked(rt:&mut impl Runtime,program:Account,source:Account,mint:Account,
 destination:Account,authority:Account,amount:u64,decimals:u8,seeds:&Seeds)->R<()> {
 supported_program(program.key)?;
 let mut data=vec![12];data.extend_from_slice(&amount.to_le_bytes());data.push(decimals);
 rt.invoke_signed(Cpi{program:program.key,metas:vec![meta(source,true,false),meta(mint,false,false),meta(destination,true,false),meta(authority,false,true)],data},
 &[source,mint,destination,authority],seeds)
}
pub fn close_token(rt:&mut impl Runtime,program:Account,source:Account,destination:Account,authority:Account,seeds:&Seeds)->R<()> {
 supported_program(program.key)?;
 rt.invoke_signed(Cpi{program:program.key,metas:vec![meta(source,true,false),meta(destination,true,false),meta(authority,false,true)],data:vec![9]},&[source,destination,authority],seeds)
}
pub fn burn_then_close(rt:&mut impl Runtime,program:Account,source:Account,mint:Account,authority:Account,destination:Account,seeds:&Seeds)->R<()> {
 let residual=rt.token(source)?.amount;
 if residual!=0 {
  supported_program(program.key)?;
  let mut data=vec![8];data.extend_from_slice(&residual.to_le_bytes());
  rt.invoke_signed(Cpi{program:program.key,metas:vec![meta(source,true,false),meta(mint,true,false),meta(authority,false,true)],data},&[source,mint,authority],seeds)?;
 }
 close_token(rt,program,source,destination,authority,seeds)
}
pub fn native_fund_sync(rt:&mut impl Runtime,system:Account,token_program:Account,from:Account,temp:Account,amount:u64)->R<()> {
 let mut data=2u32.to_le_bytes().to_vec();data.extend_from_slice(&amount.to_le_bytes());
 rt.invoke_signed(Cpi{program:system.key,metas:vec![meta(from,true,true),meta(temp,true,false)],data},&[from,temp],&vec![])?;
 rt.invoke_signed(Cpi{program:token_program.key,metas:vec![meta(temp,true,false)],data:vec![17]},&[temp],&vec![])
}
// Concrete application106 adapter: borrow, scan, preserve distinct decode error.
pub fn safe_mint_106(rt:&mut impl Runtime,mint:Account)->R<bool> {
 // DecodeError is carried without being collapsed to custom6031.
 let owner=crate::generated_exit::AccountRuntime::owner(rt,&mint);
 let value=rt.with_mint_data(mint,|data|Ok(crate::mint_policy::safe_mint_data(data,owner==crate::codec::constants::TOKEN2022)))?;
 value.map_err(|e|rt.mint_decode_error(e))
}

fn supported_program(key:Key)->R<()> {
 if key==crate::codec::constants::TOKEN || key==crate::codec::constants::TOKEN2022 {Ok(())} else {Err(Error::IncorrectProgramId)}
}
// Framework program-account close is implemented only by generated_exit::close282.
// This module owns token CloseAccount CPIs, not generated program-account exits.
