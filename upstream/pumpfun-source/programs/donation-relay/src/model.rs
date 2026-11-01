//! One representation shared across all eleven instructions. No repr(C) or SBF layout claim.
pub type Key = [u8;32];
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State { Uninitialized, Initialized }
#[derive(Clone)]
pub struct Tracker {pub bump:u8,pub state:State,pub config_id:Key,pub mint:Key,pub current_epoch:u128}
#[derive(Clone)]
pub struct Debouncer {pub bump:u8,pub state:State,pub config_id:Key,pub mint:Key,pub total_amount:u64}
#[derive(Clone)]
pub struct Distribution {pub bump:u8,pub state:State,pub config_id:Key,pub mint:Key,pub epoch:u128,pub total_amount:u64}
#[derive(Clone)]
pub struct SwapReceipt {pub bump:u8,pub distribution:Key,pub output_amount:u64}
pub struct Whitelist {pub bump:u8,pub mints:Vec<Key>}
#[derive(Clone, Copy)]
pub enum ShareKind {Beneficiary, Fee, Tip}
#[derive(Clone)]
pub struct Share {pub kind:ShareKind,pub authority:Key,pub bps:u16}
pub struct TransferReceipt {pub share:Share,pub amount:u64}
pub struct DonationArgs {pub amount:u64,pub config_id:Key,pub tip_bps:u16,pub message:String,pub credited_to:Key}
#[derive(Clone,Copy)]
pub enum DonationVariant {TokenBytes,TokenPubkey,TokenPubkeyWithPayer,NativeBytes,NativePubkey}
pub enum Event {
 Made {config_id:Key,mint:Key,epoch:u128,gross:u64,tip:u64,message:String,credited_to:Key},
 Closed {config_id:Key,mint:Key,epoch:u128,gross:u64},
 Processed {config_id:Key,mint:Key,epoch:u128,amount:u64,settlement:Key,receipts:Vec<TransferReceipt>},
 Swapped {config_id:Key,mint:Key,epoch:u128,distribution:Key,receipt:Key,output_vault:Key,amount:u64,settlement:Key},
}
// Account handles identify genuine runtime accounts, including alias relationships.
// Copies do not invent independent lamports/data when two handles alias.
#[derive(Clone,Copy)]
pub struct Account {pub(crate) handle:usize,pub(crate) key:Key,pub(crate) owner:Key,pub(crate) writable:bool,pub(crate) signer:bool}
#[derive(Clone,Copy)]
pub struct TokenView {pub account:Account,pub authority:Key,pub mint:Key,pub amount:u64}
#[derive(Clone,Copy)]
pub struct MintView {pub account:Account,pub decimals:u8}
#[derive(Clone)]
pub struct Meta {pub key:Key,pub writable:bool,pub signer:bool}
pub struct Cpi {pub program:Key,pub metas:Vec<Meta>,pub data:Vec<u8>}
pub type Seeds = Vec<Vec<u8>>; // One group when nonempty; no multisig expansion.
pub fn distribution_seeds(d:&Distribution)->Seeds {
 vec![b"distribution_v1".to_vec(),d.config_id.to_vec(),d.mint.to_vec(),d.epoch.to_le_bytes().to_vec(),vec![d.bump]]
}
pub fn debouncer_seeds(d:&Debouncer)->Seeds {
 vec![b"debouncer_v1".to_vec(),d.config_id.to_vec(),d.mint.to_vec(),vec![d.bump]]
}

// Account is the invocation arena capability, not an independent AccountInfo.
// Only the trusted loader adapter constructs it, from real account metadata.
// All copies of handle resolve the SAME live owner/data/lamports/borrow cells.
// `owner` is the interface decoding snapshot, never used for live owner tests.
// Arena lifetime spans body, event, exit and rollback. No user-provided signer bit.
