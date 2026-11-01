//! Concrete ordered application admission. Primitive adapters MUST be authentic.
use crate::{*,codec::constants::*,runtime::Runtime,generated_exit::AccountRuntime};
use crate::dispatch::{D,Failure,FrameworkFailure};
use crate::frames::*;
pub const ASSOCIATED:Key=[140,151,37,143,78,36,137,241,187,61,16,41,20,142,13,131,11,90,19,153,218,255,16,132,4,142,123,216,219,233,248,89];
pub const SYSTEM:Key=[0;32];

/// These are framework/syscall operations, not context-validation callbacks.
/// No default implementation. Rent and PDA failures must propagate or panic as
/// specified, and CPI must use original account capabilities/transaction rollback.
pub trait ContextRuntime: Runtime {
 type Rent;
 fn executable(&self,a:Account)->bool;
 fn rent_get(&mut self)->D<Self::Rent>;
 fn minimum_balance(&self,r:&Self::Rent,space:usize)->u64;
 fn rent_is_exempt(&self,r:&Self::Rent,lamports:u64,len:usize)->bool;
 fn data_len_panicking(&self,a:Account)->usize;
 fn with_data<T,F>(&self,a:Account,f:F)->D<T> where F:FnOnce(&[u8])->D<T>;
 // with_data uses try_borrow_data; borrowed data remains held until f returns.
 fn find_pda(&self,seeds:&[&[u8]],program:&Key)->(Key,u8);
 fn create_pda(&self,seeds:&[&[u8]],program:&Key)->Result<Key,()>;
 // Only token-library extension sizing; NOT relay safe-mint policy106.
 // Parse mint/TLV, get required account extensions, calculate account size.
 fn token2022_account_space(&self,mint:Account)->D<usize>;
 // SPL StateWithExtensions/base decoding only; owner checks are concrete below.
 // Preserve base initialization/COption tags, extension headers and library errors.
 // Runtime::unpack_token/unpack_mint supply those library contracts.
}
// One decoded account allocation survives admission, body and generated exit.
// This candidate representation addresses the observed SBF stack overflow;
// it is not a claim that the historical source used this exact Box layout.
pub type Loaded<T> = Box<LoadedValue<T>>;
pub struct LoadedValue<T> {pub info:Account,pub value:T}
impl<T> LoadedValue<T> {
 pub fn state(&self)->crate::generated_exit::StateAccount<'_,Account,T> {
  crate::generated_exit::StateAccount{info:&self.info,value:&self.value}
 }
 pub fn interface(&self)->crate::generated_exit::Interface<'_,Account> {
  crate::generated_exit::Interface{info:&self.info,cached_owner:self.info.owner}
 }
}
fn error(code:u32,field:Option<&'static str>,compared_keys:Option<(Key,Key)>)->Failure {
 Failure::Framework(FrameworkFailure{code,account:field,compared_keys,origin:None})
}
fn check(ok:bool,code:u32,field:Option<&'static str>)->D<()> {
 if ok {Ok(())} else {Err(error(code,field,None))}
}
fn equal(a:Key,b:Key,code:u32,field:&'static str)->D<()> {
 if a==b {Ok(())} else {Err(error(code,Some(field),Some((a,b))))}
}
fn named(e:Failure,field:&'static str)->Failure {
 match e {
  Failure::Framework(mut x)=>{x.account=Some(field);x.origin=None;Failure::Framework(x)},
  Failure::Program{error,..}=>Failure::Program{error,account:Some(field)},
  Failure::Body(Error::Program(error))=>Failure::Program{error,account:Some(field)},
  x=>x
 }
}
fn decode_error(e:Error,field:&'static str)->Failure {
 match e {Error::Code(c)=>error(c,Some(field),None),Error::Program(error)=>Failure::Program{error,account:Some(field)},x=>Failure::Body(x)}
}
pub struct Cursor<'a> {rest:&'a [Account]}
impl<'a> Cursor<'a> {
 fn new(a:&'a [Account])->Self{Self{rest:a}}
 fn take(&mut self,name:&'static str)->D<Account> {
  let (first,rest)=self.rest.split_first().ok_or_else(||error(3005,Some(name),None))?;
  self.rest=rest;Ok(*first)
 }
 fn remaining(&self)->Vec<Account>{self.rest.to_vec()}
}
fn snapshot<R:ContextRuntime>(r:&R,mut a:Account)->Account {
 a.owner=AccountRuntime::owner(r,&a);a
}
fn writable(a:Account,n:&'static str)->D<()>{check(a.writable,2000,Some(n))}
fn signer(c:&mut Cursor,n:&'static str)->D<Account> {
 let a=c.take(n)?;check(a.signer,3010,Some(n))?;Ok(a)
}
fn program<R:ContextRuntime>(r:&R,c:&mut Cursor,keys:&[Key],n:&'static str)->D<Account> {
 let a=c.take(n)?;check(keys.contains(&a.key),3008,Some(n))?;
 check(r.executable(a),3009,Some(n))?;Ok(a)
}
fn interface_owner<R:ContextRuntime>(r:&R,a:Account,n:&'static str)->D<()> {
 if a.owner==SYSTEM && r.lamports_panicking(&a)==0 {return Err(error(3012,Some(n),None));}
 // W8/9 include the actual-owner diagnostic; exact owner-list formatting is open.
 check(a.owner==TOKEN||a.owner==TOKEN2022,3007,Some(n))
}
fn token<R:ContextRuntime>(r:&mut R,a:Account,n:&'static str)->D<Loaded<TokenView>> {
 let a=snapshot(r,a);interface_owner(r,a,n)?;
 let (mint,authority,amount)=r.with_data(a,|b|R::unpack_token(b).map_err(|e|decode_error(e,n))).map_err(|e|named(e,n))?;
 let value=TokenView{account:a,mint,authority,amount};Ok(Box::new(LoadedValue{info:a,value}))
}
fn load_mint<R:ContextRuntime>(r:&mut R,a:Account,n:&'static str)->D<Loaded<MintView>> {
 let a=snapshot(r,a);interface_owner(r,a,n)?;
 let decimals=r.with_data(a,|b|R::unpack_mint(b).map_err(|e|decode_error(e,n))).map_err(|e|named(e,n))?;
 let value=MintView{account:a,decimals};Ok(Box::new(LoadedValue{info:a,value}))
}
pub trait StateWire:Sized {fn decode(data:&[u8],checked:bool)->crate::R<Self>;}
macro_rules! wire {($t:ty,$f:ident)=>{impl StateWire for $t {fn decode(b:&[u8],c:bool)->crate::R<Self>{codec::$f(b,c)}}};}
wire!(Tracker,tracker_mode);wire!(Debouncer,debouncer_mode);wire!(Distribution,distribution_mode);
wire!(SwapReceipt,swap_receipt_mode);wire!(Whitelist,whitelist_mode);
fn state<R:ContextRuntime,T:StateWire>(r:&R,a:Account,checked:bool,n:&'static str)->D<Loaded<T>> {
 let a=snapshot(r,a);
 // W13/14: System-owner test then ordinary lamport borrow, before wrong owner.
 if a.owner==SYSTEM && r.lamports_panicking(&a)==0 {return Err(error(3012,Some(n),None));}
 equal(a.owner,RELAY,3007,n)?;
 let value=r.with_data(a,|b|T::decode(b,checked).map_err(|e|decode_error(e,n))).map_err(|e|named(e,n))?;
 Ok(Box::new(LoadedValue{info:a,value}))
}
fn rent_exempt<R:ContextRuntime>(r:&R,a:Account,rent:&R::Rent,n:&'static str)->D<()> {
 let balance=r.lamports_panicking(&a); // 326 BEFORE fallible328 data borrow
 let len=r.with_data(a,|b|Ok(b.len()))?;
 check(r.rent_is_exempt(rent,balance,len),2005,Some(n))
}
fn canonical<R:ContextRuntime>(r:&R,p:&Key,a:Account,seeds:&[&[u8]],n:&'static str)->D<u8> {
 let (key,bump)=r.find_pda(seeds,p);equal(a.key,key,2006,n)?;Ok(bump)
}
fn stored<R:ContextRuntime>(r:&R,p:&Key,a:Account,seeds:&[&[u8]],bump:u8,n:&'static str)->D<()> {
 let b=[bump];let mut s=seeds.to_vec();s.push(&b);
 let key=r.create_pda(&s,p).map_err(|_|error(2006,Some(n),None))?;
 equal(a.key,key,2006,n)
}
fn associated<R:ContextRuntime>(r:&R,authority:Key,program:Key,mint:Key)->Key {
 r.find_pda(&[&authority,&program,&mint],&ASSOCIATED).0
}
fn meta(a:Account,writable:bool,signer:bool)->Meta{Meta{key:a.key,writable,signer}}
fn invoke<R:ContextRuntime>(r:&mut R,program:Key,metas:Vec<Meta>,data:Vec<u8>,accounts:&[Account],seeds:&[&[u8]])->D<()> {
 r.invoke_signed(Cpi{program,metas,data},accounts,&seeds.iter().map(|s|s.to_vec()).collect()).map_err(Failure::Body)
}
fn system_create<R:ContextRuntime>(r:&mut R,system:Account,payer:Account,a:Account,lamports:u64,size:usize,owner:Key,seeds:&[&[u8]])->D<()> {
 let mut data=0u32.to_le_bytes().to_vec();data.extend(lamports.to_le_bytes());data.extend((size as u64).to_le_bytes());data.extend(owner);
 invoke(r,system.key,vec![meta(payer,true,true),meta(a,true,true)],data,&[payer,a],seeds)
}
fn system_transfer<R:ContextRuntime>(r:&mut R,system:Account,payer:Account,a:Account,amount:u64)->D<()> {
 let mut data=2u32.to_le_bytes().to_vec();data.extend(amount.to_le_bytes());
 invoke(r,system.key,vec![meta(payer,true,true),meta(a,true,false)],data,&[payer,a],&[])
}
fn allocate_assign<R:ContextRuntime>(r:&mut R,system:Account,a:Account,size:usize,owner:Key,seeds:&[&[u8]])->D<()> {
 let mut data=8u32.to_le_bytes().to_vec();data.extend((size as u64).to_le_bytes());
 invoke(r,system.key,vec![meta(a,true,true)],data,&[a],seeds)?;
 let mut data=1u32.to_le_bytes().to_vec();data.extend(owner);
 invoke(r,system.key,vec![meta(a,true,true)],data,&[a],seeds)
}
fn initialize<R:ContextRuntime,T:StateWire>(r:&mut R,p:&Key,a:Account,payer:Account,system:Account,rent:&R::Rent,size:usize,seeds:&[&[u8]],name:&'static str)->D<Loaded<T>> {
 let balance=r.lamports_panicking(&a);
 if balance==0 {let minimum=r.minimum_balance(rent,size);system_create(r,system,payer,a,minimum,size,*p,seeds)?;}
 else {
  if payer.key==a.key {return Err(error(4101,None,Some((payer.key,a.key))));}
  let needed=r.minimum_balance(rent,size).max(1);
  if needed>balance {system_transfer(r,system,payer,a,needed-balance)?;}
  allocate_assign(r,system,a,size,*p,seeds)?;
 }
 state(r,a,false,name)
}
fn init_if_needed<R:ContextRuntime,T:StateWire>(r:&mut R,p:&Key,a:Account,payer:Account,system:Account,rent:&R::Rent,size:usize,seeds:&[&[u8]],name:&'static str)->D<Loaded<T>> {
 if AccountRuntime::owner(r,&a)==SYSTEM {return initialize(r,p,a,payer,system,rent,size,seeds,name);}
 let value=state(r,a,true,name)?; // checked decode BEFORE size and owner constraints
 check(r.data_len_panicking(a)==size,2019,Some(name))?;
 equal(AccountRuntime::owner(r,&a),*p,2004,name)?;
 check(r.lamports_panicking(&a)>=r.minimum_balance(rent,size),2005,Some(name))?;
 Ok(value)
}
fn create_ata<R:ContextRuntime>(r:&mut R,target:Account,payer:Account,authority:Account,m:Account,tp:Account,system:Account,ata:Account,name:&'static str)->D<Loaded<TokenView>> {
 let key=associated(r,authority.key,tp.key,m.key);
 let metas=vec![meta(payer,true,true),Meta{key,writable:true,signer:false},meta(authority,false,false),meta(m,false,false),meta(system,false,false),meta(tp,false,false)];
 invoke(r,ata.key,metas,vec![0],&[payer,target,authority,m,system,tp],&[])?;
 token(r,target,name)
}
fn destination_ata<R:ContextRuntime>(r:&mut R,a:Account,payer:Account,authority:Account,m:Account,tp:Account,system:Account,ata:Account)->D<Loaded<TokenView>> {
 let name="debouncer_token_account";
 let value=if AccountRuntime::owner(r,&a)==SYSTEM {create_ata(r,a,payer,authority,m,tp,system,ata,name)?} else {token(r,a,name)?};
 equal(value.value.mint,m.key,2014,name)?;
 equal(value.value.authority,authority.key,2015,name)?;
 check(AccountRuntime::owner(r,&a)==tp.key,2023,Some(name))?;
 equal(a.key,associated(r,authority.key,tp.key,m.key),3014,name)?;
 Ok(value)
}
fn temp_space<R:ContextRuntime>(r:&R,m:Account)->D<usize> {
 if AccountRuntime::owner(r,&m)==TOKEN2022 {r.token2022_account_space(m)} else {Ok(165)}
}
fn native_temp<R:ContextRuntime>(r:&mut R,a:Account,from:Account,m:Account,tp:Account,system:Account,rent:&R::Rent,seeds:&[&[u8]])->D<Loaded<TokenView>> {
 let balance=r.lamports_panicking(&a);
 if balance==0 {let size=temp_space(r,m)?;let minimum=r.minimum_balance(rent,size);system_create(r,system,from,a,minimum,size,tp.key,seeds)?;}
 else {
  if from.key==a.key {return Err(error(4101,None,Some((from.key,a.key))));}
  let size=temp_space(r,m)?;let needed=r.minimum_balance(rent,size).max(1);
  if needed>balance {system_transfer(r,system,from,a,needed-balance)?;}
  let size=temp_space(r,m)?; // MUST repeat extension sizing after the transfer
  allocate_assign(r,system,a,size,tp.key,seeds)?;
 }
 let mut data=vec![18];data.extend(from.key); // SPL InitializeAccount3
 invoke(r,tp.key,vec![meta(a,true,false),meta(m,false,false)],data,&[a,m],&[])?;
 token(r,a,"temp_wsol_token_account")
}
fn existing_ata<R:ContextRuntime>(r:&R,v:&Loaded<TokenView>,authority:Account,m:Account,tp:Account,n:&'static str)->D<()> {
 check(AccountRuntime::owner(r,&v.info)==tp.key,2023,None)?;
 equal(v.value.authority,authority.key,2015,n)?;
 equal(v.info.key,associated(r,authority.key,tp.key,m.key),2009,n)?;
 writable(v.info,n)
}
fn mint_constraints<R:ContextRuntime>(r:&R,m:Account,tp:Account,expected:Key,n:&'static str,code:u32)->D<()> {
 writable(m,n)?;equal(m.key,expected,code,n)?;
 check(AccountRuntime::owner(r,&m)==tp.key,2022,None)
}
fn fixed_relayer(a:Account)->D<()> {writable(a,"signer")?;equal(a.key,RELAYER,2012,"signer")}
fn not_close(a:Account,s:Account,n:&'static str)->D<()>{check(a.key!=s.key,2011,Some(n))}
fn distribution_seeds<R:ContextRuntime>(r:&R,p:&Key,t:&Loaded<Tracker>,b:&Loaded<Debouncer>,d:&Loaded<Distribution>,mint_key:Key)->D<()> {
 stored(r,p,t.info,&[b"epoch_tracker_v1",&d.value.config_id,&mint_key],t.value.bump,"epoch_tracker")?;
 stored(r,p,b.info,&[b"debouncer_v1",&d.value.config_id,&mint_key],b.value.bump,"debouncer")?;
 stored(r,p,d.info,&[b"distribution_v1",&d.value.config_id,&d.value.mint,&d.value.epoch.to_le_bytes()],d.value.bump,"distribution")?;
 writable(d.info,"distribution")
}
pub struct Contexts;
fn consume_142<R:ContextRuntime>(r:&mut R,c:&mut Cursor)->D<RawEpoch> {
 let distribution=c.take("distribution")?;
 let distribution_token_account=c.take("distribution_token_account")?;
 let epoch_tracker_raw=c.take("epoch_tracker")?;
 let epoch_tracker=state(r,epoch_tracker_raw,true,"epoch_tracker")?;
 let debouncer_raw=c.take("debouncer")?;
 let debouncer=state(r,debouncer_raw,true,"debouncer")?;
 let debouncer_token_account_raw=c.take("debouncer_token_account")?;
 let debouncer_token_account=token(r,debouncer_token_account_raw,"debouncer_token_account")?;
 let mint_raw=c.take("mint")?;
 let mint=load_mint(r,mint_raw,"mint")?;
 let token_program=program(r,c,&[TOKEN,TOKEN2022],"token_program")?;
 let signer=signer(c,"signer")?;
 let associated_token_program=program(r,c,&[ASSOCIATED],"associated_token_program")?;
 let system_program=program(r,c,&[SYSTEM],"system_program")?;
 let event_authority=c.take("event_authority")?;
 let program=c.take("program")?;
 Ok(RawEpoch{distribution,distribution_token_account,epoch_tracker,debouncer,debouncer_token_account,mint,token_program,signer,associated_token_program,system_program,event_authority,program})
}
struct RawEpoch {
 distribution:Account,
 distribution_token_account:Account,
 epoch_tracker:Loaded<Tracker>,
 debouncer:Loaded<Debouncer>,
 debouncer_token_account:Loaded<TokenView>,
 mint:Loaded<MintView>,
 token_program:Account,
 signer:Account,
 associated_token_program:Account,
 system_program:Account,
 event_authority:Account,
 program:Account,
}
// Epoch consuming fields: distribution,distribution_token_account,epoch_tracker,debouncer,debouncer_token_account,mint,token_program,signer,associated_token_program,system_program,event_authority,program
fn consume_146<R:ContextRuntime>(r:&mut R,c:&mut Cursor)->D<RawAfter> {
 let epoch_tracker_raw=c.take("epoch_tracker")?;
 let epoch_tracker=state(r,epoch_tracker_raw,true,"epoch_tracker")?;
 let debouncer_raw=c.take("debouncer")?;
 let debouncer=state(r,debouncer_raw,true,"debouncer")?;
 let distribution_raw=c.take("distribution")?;
 let distribution=state(r,distribution_raw,true,"distribution")?;
 let swap_receipt_raw=c.take("swap_receipt")?;
 let swap_receipt=state(r,swap_receipt_raw,true,"swap_receipt")?;
 let output_distribution_token_account_raw=c.take("output_distribution_token_account")?;
 let output_distribution_token_account=token(r,output_distribution_token_account_raw,"output_distribution_token_account")?;
 let output_mint_raw=c.take("output_mint")?;
 let output_mint=load_mint(r,output_mint_raw,"output_mint")?;
 let output_mint_program=program(r,c,&[TOKEN,TOKEN2022],"output_mint_program")?;
 let signer=signer(c,"signer")?;
 let associated_token_program=program(r,c,&[ASSOCIATED],"associated_token_program")?;
 let system_program=program(r,c,&[SYSTEM],"system_program")?;
 let event_authority=c.take("event_authority")?;
 let program=c.take("program")?;
 Ok(RawAfter{epoch_tracker,debouncer,distribution,swap_receipt,output_distribution_token_account,output_mint,output_mint_program,signer,associated_token_program,system_program,event_authority,program})
}
struct RawAfter {
 epoch_tracker:Loaded<Tracker>,
 debouncer:Loaded<Debouncer>,
 distribution:Loaded<Distribution>,
 swap_receipt:Loaded<SwapReceipt>,
 output_distribution_token_account:Loaded<TokenView>,
 output_mint:Loaded<MintView>,
 output_mint_program:Account,
 signer:Account,
 associated_token_program:Account,
 system_program:Account,
 event_authority:Account,
 program:Account,
}
// After consuming fields: epoch_tracker,debouncer,distribution,swap_receipt,output_distribution_token_account,output_mint,output_mint_program,signer,associated_token_program,system_program,event_authority,program
fn consume_148<R:ContextRuntime>(r:&mut R,c:&mut Cursor)->D<RawNonSwap> {
 let epoch_tracker_raw=c.take("epoch_tracker")?;
 let epoch_tracker=state(r,epoch_tracker_raw,true,"epoch_tracker")?;
 let debouncer_raw=c.take("debouncer")?;
 let debouncer=state(r,debouncer_raw,true,"debouncer")?;
 let distribution_raw=c.take("distribution")?;
 let distribution=state(r,distribution_raw,true,"distribution")?;
 let distribution_token_account_raw=c.take("distribution_token_account")?;
 let distribution_token_account=token(r,distribution_token_account_raw,"distribution_token_account")?;
 let mint_raw=c.take("mint")?;
 let mint=load_mint(r,mint_raw,"mint")?;
 let signer=signer(c,"signer")?;
 let token_program=program(r,c,&[TOKEN,TOKEN2022],"token_program")?;
 let associated_token_program=program(r,c,&[ASSOCIATED],"associated_token_program")?;
 let system_program=program(r,c,&[SYSTEM],"system_program")?;
 let event_authority=c.take("event_authority")?;
 let program=c.take("program")?;
 Ok(RawNonSwap{epoch_tracker,debouncer,distribution,distribution_token_account,mint,signer,token_program,associated_token_program,system_program,event_authority,program})
}
struct RawNonSwap {
 epoch_tracker:Loaded<Tracker>,
 debouncer:Loaded<Debouncer>,
 distribution:Loaded<Distribution>,
 distribution_token_account:Loaded<TokenView>,
 mint:Loaded<MintView>,
 signer:Account,
 token_program:Account,
 associated_token_program:Account,
 system_program:Account,
 event_authority:Account,
 program:Account,
}
// NonSwap consuming fields: epoch_tracker,debouncer,distribution,distribution_token_account,mint,signer,token_program,associated_token_program,system_program,event_authority,program
fn consume_150<R:ContextRuntime>(r:&mut R,c:&mut Cursor)->D<RawJust> {
 let epoch_tracker_raw=c.take("epoch_tracker")?;
 let epoch_tracker=state(r,epoch_tracker_raw,true,"epoch_tracker")?;
 let debouncer_raw=c.take("debouncer")?;
 let debouncer=state(r,debouncer_raw,true,"debouncer")?;
 let distribution_raw=c.take("distribution")?;
 let distribution=state(r,distribution_raw,true,"distribution")?;
 let swap_receipt=c.take("swap_receipt")?;
 let distribution_token_account_raw=c.take("distribution_token_account")?;
 let distribution_token_account=token(r,distribution_token_account_raw,"distribution_token_account")?;
 let output_distribution_token_account=c.take("output_distribution_token_account")?;
 let mint_raw=c.take("mint")?;
 let mint=load_mint(r,mint_raw,"mint")?;
 let output_mint_raw=c.take("output_mint")?;
 let output_mint=load_mint(r,output_mint_raw,"output_mint")?;
 let output_mint_program=program(r,c,&[TOKEN,TOKEN2022],"output_mint_program")?;
 let signer=signer(c,"signer")?;
 let token_program=program(r,c,&[TOKEN,TOKEN2022],"token_program")?;
 let jupiter_program=c.take("jupiter_program")?;
 let associated_token_program=program(r,c,&[ASSOCIATED],"associated_token_program")?;
 let system_program=program(r,c,&[SYSTEM],"system_program")?;
 let event_authority=c.take("event_authority")?;
 let program=c.take("program")?;
 Ok(RawJust{epoch_tracker,debouncer,distribution,swap_receipt,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program})
}
struct RawJust {
 epoch_tracker:Loaded<Tracker>,
 debouncer:Loaded<Debouncer>,
 distribution:Loaded<Distribution>,
 swap_receipt:Account,
 distribution_token_account:Loaded<TokenView>,
 output_distribution_token_account:Account,
 mint:Loaded<MintView>,
 output_mint:Loaded<MintView>,
 output_mint_program:Account,
 signer:Account,
 token_program:Account,
 jupiter_program:Account,
 associated_token_program:Account,
 system_program:Account,
 event_authority:Account,
 program:Account,
}
// Just consuming fields: epoch_tracker,debouncer,distribution,swap_receipt,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program
fn consume_154<R:ContextRuntime>(r:&mut R,c:&mut Cursor)->D<RawCombined> {
 let epoch_tracker_raw=c.take("epoch_tracker")?;
 let epoch_tracker=state(r,epoch_tracker_raw,true,"epoch_tracker")?;
 let debouncer_raw=c.take("debouncer")?;
 let debouncer=state(r,debouncer_raw,true,"debouncer")?;
 let distribution_raw=c.take("distribution")?;
 let distribution=state(r,distribution_raw,true,"distribution")?;
 let distribution_token_account_raw=c.take("distribution_token_account")?;
 let distribution_token_account=token(r,distribution_token_account_raw,"distribution_token_account")?;
 let output_distribution_token_account=c.take("output_distribution_token_account")?;
 let mint_raw=c.take("mint")?;
 let mint=load_mint(r,mint_raw,"mint")?;
 let output_mint_raw=c.take("output_mint")?;
 let output_mint=load_mint(r,output_mint_raw,"output_mint")?;
 let output_mint_program=program(r,c,&[TOKEN,TOKEN2022],"output_mint_program")?;
 let signer=signer(c,"signer")?;
 let token_program=program(r,c,&[TOKEN,TOKEN2022],"token_program")?;
 let jupiter_program=c.take("jupiter_program")?;
 let associated_token_program=program(r,c,&[ASSOCIATED],"associated_token_program")?;
 let system_program=program(r,c,&[SYSTEM],"system_program")?;
 let event_authority=c.take("event_authority")?;
 let program=c.take("program")?;
 Ok(RawCombined{epoch_tracker,debouncer,distribution,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program})
}
struct RawCombined {
 epoch_tracker:Loaded<Tracker>,
 debouncer:Loaded<Debouncer>,
 distribution:Loaded<Distribution>,
 distribution_token_account:Loaded<TokenView>,
 output_distribution_token_account:Account,
 mint:Loaded<MintView>,
 output_mint:Loaded<MintView>,
 output_mint_program:Account,
 signer:Account,
 token_program:Account,
 jupiter_program:Account,
 associated_token_program:Account,
 system_program:Account,
 event_authority:Account,
 program:Account,
}
// Combined consuming fields: epoch_tracker,debouncer,distribution,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program
fn consume_156<R:ContextRuntime>(r:&mut R,c:&mut Cursor)->D<RawWhitelistFrameData> {
 let mint_whitelist=c.take("mint_whitelist")?;
 let signer=signer(c,"signer")?;
 let system_program=program(r,c,&[SYSTEM],"system_program")?;
 Ok(RawWhitelistFrameData{mint_whitelist,signer,system_program})
}
struct RawWhitelistFrameData {
 mint_whitelist:Account,
 signer:Account,
 system_program:Account,
}
// WhitelistFrameData consuming fields: mint_whitelist,signer,system_program
impl<R:ContextRuntime> crate::dispatch_interfaces::Admission<R> for Contexts {
 type Donation=Donation;type Epoch=Epoch;type After=After;type NonSwap=NonSwap;
 type Just=Just;type Combined=Combined;type Whitelist=WhitelistFrameData;
 fn donation(&mut self,r:&mut R,p:&Key,accounts:&[Account],raw_args:&[u8],kind:DonationVariant,_args:&DonationArgs)->D<Donation> {
  // Parser repeats the full donation decode BEFORE taking any accounts.
  let name=match kind {DonationVariant::TokenBytes=>"donate_v1",DonationVariant::TokenPubkey=>"donate_pubkey_config_id_v1",DonationVariant::TokenPubkeyWithPayer=>"donate_pubkey_config_id_with_payer_v1",DonationVariant::NativeBytes=>"donate_native_v1",DonationVariant::NativePubkey=>"donate_native_pubkey_config_id_v1"};
  let decoded=codec::decode_arguments(name,raw_args).map_err(Failure::Body)?;
  let config=match decoded {codec::Instruction::Donation(_,a)=>a.config_id,_=>unreachable!("fixed decoder selection")};
  let native=matches!(kind,DonationVariant::NativeBytes|DonationVariant::NativePubkey);
  let with_payer=matches!(kind,DonationVariant::TokenPubkeyWithPayer);
  let mut c=Cursor::new(accounts);
  let epoch=c.take("epoch_tracker")?;let debouncer=c.take("debouncer")?;
  let destination=c.take("debouncer_token_account")?;
  let mint_raw=c.take("mint")?;let mint=load_mint(r,mint_raw,"mint")?;
  let source_name=if native {"temp_wsol_token_account"} else {"from_token_account"};
  let source_raw=c.take(source_name)?;
  let source=if native {None} else {Some(token(r,source_raw,source_name)?)};
  let from=signer(&mut c,"from")?;
  let payer=if with_payer {signer(&mut c,"payer")?} else {from};
  let whitelist_raw=c.take("mint_whitelist")?;
  let whitelist:Loaded<Whitelist>=state(r,whitelist_raw,true,"mint_whitelist")?;
  let associated=program(r,&mut c,&[ASSOCIATED],"associated_token_program")?;
  let token_program=program(r,&mut c,&[TOKEN,TOKEN2022],"token_program")?;
  let system=program(r,&mut c,&[SYSTEM],"system_program")?;
  let event_authority=c.take("event_authority")?;let program=c.take("program")?;
  let remaining=c.remaining();
  let rent=r.rent_get()?;
  let m=mint.info.key;let tp=token_program.key;
  let epoch_bump=canonical(r,p,epoch,&[b"epoch_tracker_v1",&config,&m],"epoch_tracker")?;
  let epoch_tracker=init_if_needed(r,p,epoch,payer,system,&rent,90,&[b"epoch_tracker_v1",&config,&m,&[epoch_bump]],"epoch_tracker")?;
  writable(epoch,"epoch_tracker")?;rent_exempt(r,epoch,&rent,"epoch_tracker")?;
  let debouncer_bump=canonical(r,p,debouncer,&[b"debouncer_v1",&config,&m],"debouncer")?;
  let debouncer=init_if_needed(r,p,debouncer,payer,system,&rent,82,&[b"debouncer_v1",&config,&m,&[debouncer_bump]],"debouncer")?;
  writable(debouncer.info,"debouncer")?;rent_exempt(r,debouncer.info,&rent,"debouncer")?;
  let destination=destination_ata(r,destination,payer,debouncer.info,mint.info,token_program,system,associated)?;
  writable(destination.info,"debouncer_token_account")?;rent_exempt(r,destination.info,&rent,"debouncer_token_account")?;
  let (source,temp_bump)=if native {
   let bump=canonical(r,p,source_raw,&[b"donate_native_wsol_tmp_v1",&from.key,&tp],source_name)?;
   let source=native_temp(r,source_raw,from,mint.info,token_program,system,&rent,&[b"donate_native_wsol_tmp_v1",&from.key,&tp,&[bump]])?;
   writable(source.info,source_name)?;rent_exempt(r,source.info,&rent,source_name)?;(source,Some(bump))
  } else {(source.expect("ordinary source decoded above"),None)};
  writable(mint.info,"mint")?;
  if native {check((tp==TOKEN&&m==NATIVE)||(tp==TOKEN2022&&m==NATIVE2022),6006,Some("mint"))?;}
  check(AccountRuntime::owner(r,&mint.info)==tp,2022,None)?;
  if !native {existing_ata(r,&source,from,mint.info,token_program,"from_token_account")?;}
  writable(from,"from")?;if with_payer {writable(payer,"payer")?;}
  stored(r,p,whitelist.info,&[b"mint_whitelist_v1"],whitelist.value.bump,"mint_whitelist")?;
  if native {check(tp==TOKEN||tp==TOKEN2022,6029,Some("token_program"))?;}
  let event_bump=canonical(r,p,event_authority,&[b"__event_authority"],"event_authority")?;
  Ok(Donation{epoch_tracker,debouncer,destination,mint,source,from,payer,whitelist,token_program,system,associated,event_authority,program,epoch_bump,debouncer_bump,event_bump,temp_bump,remaining})
 }
 fn epoch(&mut self,r:&mut R,p:&Key,accounts:&[Account],raw_args:&[u8],_config:Key,_epoch:u128)->D<Epoch> {
  // W142 repeats precisely config32 + epoch16; does not demand exhaustion.
  check(raw_args.len()>=48,102,None)?;
  let config:Key=raw_args[..32].try_into().unwrap();let epoch_bytes=&raw_args[32..48];
  let mut c=Cursor::new(accounts);
  let RawEpoch{distribution,distribution_token_account,epoch_tracker,debouncer,debouncer_token_account,mint,token_program,signer,associated_token_program,system_program,event_authority,program}=consume_142(r,&mut c)?;
  let remaining=c.remaining();
  let rent=r.rent_get()?;
  let seeds:&[&[u8]]=&[b"distribution_v1",&config,&mint.info.key,epoch_bytes];
  let distribution_bump=canonical(r,p,distribution,seeds,"distribution")?;
  let bump=[distribution_bump];let mut signed=seeds.to_vec();signed.push(&bump);
  let distribution=initialize(r,p,distribution,signer,system_program,&rent,98,&signed,"distribution")?;
  writable(distribution.info,"distribution")?;rent_exempt(r,distribution.info,&rent,"distribution")?;
  let rent=r.rent_get()?;
  let distribution_token_account=create_ata(r,distribution_token_account,signer,distribution.info,mint.info,token_program,system_program,associated_token_program,"distribution_token_account")?;
  writable(distribution_token_account.info,"distribution_token_account")?;rent_exempt(r,distribution_token_account.info,&rent,"distribution_token_account")?;
  stored(r,p,epoch_tracker.info,&[b"epoch_tracker_v1",&config,&mint.info.key],epoch_tracker.value.bump,"epoch_tracker")?;
  writable(epoch_tracker.info,"epoch_tracker")?;
  stored(r,p,debouncer.info,&[b"debouncer_v1",&config,&mint.info.key],debouncer.value.bump,"debouncer")?;
  writable(debouncer.info,"debouncer")?;
  existing_ata(r,&debouncer_token_account,debouncer.info,mint.info,token_program,"debouncer_token_account")?;
  mint_constraints(r,mint.info,token_program,debouncer.value.mint,"mint",2012)?;
  fixed_relayer(signer)?;
  let event_bump=canonical(r,p,event_authority,&[b"__event_authority"],"event_authority")?;
  Ok(Epoch{distribution,distribution_token_account,epoch_tracker,debouncer,debouncer_token_account,mint,token_program,signer,associated_token_program,system_program,event_authority,program,event_bump,distribution_bump,remaining})
 }
 fn after(&mut self,r:&mut R,p:&Key,accounts:&[Account],_raw_args:&[u8],_shares:&[Share])->D<After> {
  let mut c=Cursor::new(accounts);
  let RawAfter{epoch_tracker,debouncer,distribution,swap_receipt,output_distribution_token_account,output_mint,output_mint_program,signer,associated_token_program,system_program,event_authority,program}=consume_146(r,&mut c)?;
  let remaining=c.remaining();
  distribution_seeds(r,p,&epoch_tracker,&debouncer,&distribution,distribution.value.mint)?;
  not_close(distribution.info,signer,"distribution")?;
  stored(r,p,swap_receipt.info,&[b"swap_receipt_v1",&distribution.info.key],swap_receipt.value.bump,"swap_receipt")?;
  writable(swap_receipt.info,"swap_receipt")?;
  check(swap_receipt.value.distribution==distribution.info.key,6033,Some("swap_receipt"))?;
  not_close(swap_receipt.info,signer,"swap_receipt")?;
  existing_ata(r,&output_distribution_token_account,distribution.info,output_mint.info,output_mint_program,"output_distribution_token_account")?;
  mint_constraints(r,output_mint.info,output_mint_program,SETTLEMENT,"output_mint",6006)?;
  fixed_relayer(signer)?;
  let event_bump=canonical(r,p,event_authority,&[b"__event_authority"],"event_authority")?;
  Ok(After{epoch_tracker,debouncer,distribution,swap_receipt,output_distribution_token_account,output_mint,output_mint_program,signer,associated_token_program,system_program,event_authority,program,event_bump,remaining})
 }
 fn non_swap(&mut self,r:&mut R,p:&Key,accounts:&[Account],_raw_args:&[u8],_shares:&[Share])->D<NonSwap> {
  let mut c=Cursor::new(accounts);
  let RawNonSwap{epoch_tracker,debouncer,distribution,distribution_token_account,mint,signer,token_program,associated_token_program,system_program,event_authority,program}=consume_148(r,&mut c)?;
  let remaining=c.remaining();
  distribution_seeds(r,p,&epoch_tracker,&debouncer,&distribution,mint.info.key)?;
  not_close(distribution.info,signer,"distribution")?;
  existing_ata(r,&distribution_token_account,distribution.info,mint.info,token_program,"distribution_token_account")?;
  mint_constraints(r,mint.info,token_program,SETTLEMENT,"mint",6006)?;
  fixed_relayer(signer)?;
  let event_bump=canonical(r,p,event_authority,&[b"__event_authority"],"event_authority")?;
  Ok(NonSwap{epoch_tracker,debouncer,distribution,distribution_token_account,mint,signer,token_program,associated_token_program,system_program,event_authority,program,event_bump,remaining})
 }
 fn just_swap(&mut self,r:&mut R,p:&Key,accounts:&[Account],_raw_args:&[u8],_swap_data:&[u8])->D<Just> {
  let mut c=Cursor::new(accounts);
  let RawJust{epoch_tracker,debouncer,distribution,swap_receipt,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program}=consume_150(r,&mut c)?;
  let remaining=c.remaining();
  let rent=r.rent_get()?;
  let receipt_bump=canonical(r,p,swap_receipt,&[b"swap_receipt_v1",&distribution.info.key],"swap_receipt")?;
  let swap_receipt=initialize(r,p,swap_receipt,signer,system_program,&rent,49,&[b"swap_receipt_v1",&distribution.info.key,&[receipt_bump]],"swap_receipt")?;
  writable(swap_receipt.info,"swap_receipt")?;rent_exempt(r,swap_receipt.info,&rent,"swap_receipt")?;
  let rent=r.rent_get()?;
  let output_distribution_token_account=create_ata(r,output_distribution_token_account,signer,distribution.info,output_mint.info,output_mint_program,system_program,associated_token_program,"output_distribution_token_account")?;
  writable(output_distribution_token_account.info,"output_distribution_token_account")?;rent_exempt(r,output_distribution_token_account.info,&rent,"output_distribution_token_account")?;
  distribution_seeds(r,p,&epoch_tracker,&debouncer,&distribution,distribution.value.mint)?;
  existing_ata(r,&distribution_token_account,distribution.info,mint.info,token_program,"distribution_token_account")?;
  mint_constraints(r,mint.info,token_program,distribution.value.mint,"mint",2012)?;
  mint_constraints(r,output_mint.info,output_mint_program,SETTLEMENT,"output_mint",2012)?;
  fixed_relayer(signer)?;equal(jupiter_program.key,JUPITER,2012,"jupiter_program")?;
  let event_bump=canonical(r,p,event_authority,&[b"__event_authority"],"event_authority")?;
  Ok(Just{epoch_tracker,debouncer,distribution,swap_receipt,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program,event_bump,receipt_bump,remaining})
 }
 fn combined(&mut self,r:&mut R,p:&Key,accounts:&[Account],_raw_args:&[u8],_swap_data:&[u8],_shares:&[Share])->D<Combined> {
  let mut c=Cursor::new(accounts);
  let RawCombined{epoch_tracker,debouncer,distribution,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program}=consume_154(r,&mut c)?;
  let remaining=c.remaining();
  let rent=r.rent_get()?;
  let output_distribution_token_account=create_ata(r,output_distribution_token_account,signer,distribution.info,output_mint.info,output_mint_program,system_program,associated_token_program,"output_distribution_token_account")?;
  writable(output_distribution_token_account.info,"output_distribution_token_account")?;rent_exempt(r,output_distribution_token_account.info,&rent,"output_distribution_token_account")?;
  distribution_seeds(r,p,&epoch_tracker,&debouncer,&distribution,distribution.value.mint)?;
  not_close(distribution.info,signer,"distribution")?;
  existing_ata(r,&distribution_token_account,distribution.info,mint.info,token_program,"distribution_token_account")?;
  mint_constraints(r,mint.info,token_program,distribution.value.mint,"mint",2012)?;
  mint_constraints(r,output_mint.info,output_mint_program,SETTLEMENT,"output_mint",2012)?;
  fixed_relayer(signer)?;equal(jupiter_program.key,JUPITER,2012,"jupiter_program")?;
  let event_bump=canonical(r,p,event_authority,&[b"__event_authority"],"event_authority")?;
  Ok(Combined{epoch_tracker,debouncer,distribution,distribution_token_account,output_distribution_token_account,mint,output_mint,output_mint_program,signer,token_program,jupiter_program,associated_token_program,system_program,event_authority,program,event_bump,remaining})
 }
 fn whitelist(&mut self,r:&mut R,p:&Key,accounts:&[Account],_raw_args:&[u8],_mints:&[Key])->D<WhitelistFrameData> {
  let mut c=Cursor::new(accounts);
  let RawWhitelistFrameData{mint_whitelist,signer,system_program}=consume_156(r,&mut c)?;
  let remaining=c.remaining();
  let rent=r.rent_get()?;
  let whitelist_bump=canonical(r,p,mint_whitelist,&[b"mint_whitelist_v1"],"mint_whitelist")?;
  let mint_whitelist=if AccountRuntime::owner(r,&mint_whitelist)==SYSTEM {
   initialize(r,p,mint_whitelist,signer,system_program,&rent,1037,&[b"mint_whitelist_v1",&[whitelist_bump]],"mint_whitelist")?
  } else {state(r,mint_whitelist,true,"mint_whitelist")?};
  check(r.data_len_panicking(mint_whitelist.info)==1037,2019,Some("mint_whitelist"))?;
  equal(AccountRuntime::owner(r,&mint_whitelist.info),*p,2004,"mint_whitelist")?;
  rent_exempt(r,mint_whitelist.info,&rent,"mint_whitelist")?;
  writable(mint_whitelist.info,"mint_whitelist")?;
  rent_exempt(r,mint_whitelist.info,&rent,"mint_whitelist")?;
  fixed_relayer(signer)?;
  Ok(WhitelistFrameData{mint_whitelist,signer,system_program,whitelist_bump,remaining})
 }
}
