//! Candidate real-runtime adapter. No application validation or successful mock calls.
//! The selected API family is witnessed by retained official archives; historical
//! relay dependency/toolchain identity is unknown. Only the SBF target is allowed.
use std::cell::{Ref,RefMut};
use solana_program::{account_info::AccountInfo,instruction::{AccountMeta,Instruction},
    program::{invoke_signed,set_return_data},program_error::ProgramError,
    pubkey::Pubkey,rent::Rent,sysvar::Sysvar};
use spl_token_2022::{extension::{BaseStateWithExtensions,ExtensionType,StateWithExtensions},
    state::{Account as SplAccount,Mint as SplMint}};
use crate::{Account,Key,Cpi,Seeds,Error,R};
use crate::dispatch::{D,Failure};
use crate::generated_exit::{AccountRuntime,ExitError,X};
use crate::admission::ContextRuntime;
use crate::dispatch_interfaces::DispatchRuntime;
use crate::idl::{I,program_error};

/// Retains clones of loader-created AccountInfos. The SDK loader preserves
/// duplicate-account Rc cells. No AccountInfo::new, data copy or synthesized flags.
/// Integer capabilities cannot escape the crate; they reference this invocation.
pub struct SolanaRuntime<'info> { accounts:Vec<AccountInfo<'info>> }
impl<'info> SolanaRuntime<'info> {
    pub(crate) fn from_loader(accounts:&[AccountInfo<'info>])->(Self,Vec<Account>) {
        let handles=accounts.iter().enumerate().map(|(handle,a)|Account {
            handle,key:a.key.to_bytes(),owner:a.owner.to_bytes(),
            writable:a.is_writable,signer:a.is_signer,
        }).collect();
        (Self{accounts:accounts.to_vec()},handles)
    }
    fn info(&self,a:Account)->&AccountInfo<'info> {
        // All capabilities are constructed above. This is an internal invariant,
        // not a new account-admission check on instruction input.
        &self.accounts[a.handle]
    }
    fn invoke(&self,ix:Cpi,accounts:&[Account],groups:&[Vec<Vec<u8>>])->Result<(),ProgramError> {
        let instruction=Instruction {
            program_id:Pubkey::new_from_array(ix.program),
            accounts:ix.metas.into_iter().map(|m|AccountMeta {
                pubkey:Pubkey::new_from_array(m.key),is_signer:m.signer,is_writable:m.writable,
            }).collect(),data:ix.data,
        };
        let infos:Vec<AccountInfo<'info>>=accounts.iter().map(|a|self.info(*a).clone()).collect();
        let seeds:Vec<Vec<&[u8]>>=groups.iter().map(|g|g.iter().map(Vec::as_slice).collect()).collect();
        let seed_refs:Vec<&[&[u8]]>=seeds.iter().map(Vec::as_slice).collect();
        // Ordinary SDK alias/borrow preflight, then real sol_invoke_signed syscall.
        // Signer bits are instruction requests, not changes to AccountInfo flags;
        // the runtime checks privilege propagation and derives PDA signatures.
        invoke_signed(&instruction,&infos,&seed_refs)
    }
}
fn exit_error(error:ProgramError)->ExitError {error.into()}
fn context_error(error:ProgramError)->Failure {Failure::Program{error,account:None}}

impl AccountRuntime for SolanaRuntime<'_> {
    type A=Account;
    fn owner(&self,a:&Account)->Key {self.info(*a).owner.to_bytes()}
    fn data_empty_panicking(&self,a:&Account)->bool {self.info(*a).data_is_empty()}
    fn with_data_mut<T,F>(&mut self,a:&Account,f:F)->X<T> where F:FnOnce(&mut[u8])->X<T> {
        let mut data=self.info(*a).try_borrow_mut_data().map_err(exit_error)?;
        f(&mut data)
    }
    fn lamports_panicking(&self,a:&Account)->u64 {self.info(*a).lamports()}
    fn set_lamports_panicking(&mut self,a:&Account,value:u64) {**self.info(*a).lamports.borrow_mut()=value;}
    fn assign_system(&mut self,a:&Account) {self.info(*a).assign(&Pubkey::default());}
    fn resize_zero(&mut self,a:&Account)->X<()> {self.info(*a).resize(0).map_err(exit_error)}
    fn set_return_data(&mut self,bytes:&[u8]) {set_return_data(bytes);}
}
impl crate::runtime::Runtime for SolanaRuntime<'_> {
    fn with_mint_data<T,F>(&mut self,a:Account,f:F)->R<T> where F:FnOnce(&[u8])->R<T> {
        let data=self.info(a).data.borrow();f(&data)
    }
    fn mint_decode_error(&self,e:crate::mint_policy::DecodeError)->Error {
        Error::Program(match e {
            crate::mint_policy::DecodeError::InvalidAccountData=>ProgramError::InvalidAccountData,
            crate::mint_policy::DecodeError::UninitializedAccount=>ProgramError::UninitializedAccount,
        })
    }
    fn lamports(&mut self,a:Account)->R<u64> {self.info(a).try_lamports().map_err(Error::Program)}
    fn set_lamports(&mut self,a:Account,value:u64)->R<()> {
        **self.info(a).try_borrow_mut_lamports().map_err(Error::Program)?=value;Ok(())
    }
    fn assign(&mut self,a:Account,owner:Key)->R<()> {self.info(a).assign(&Pubkey::new_from_array(owner));Ok(())}
    fn realloc_zero(&mut self,a:Account)->R<()> {self.info(a).resize(0).map_err(Error::Program)}
    fn invoke_signed(&mut self,ix:Cpi,infos:&[Account],seeds:&Seeds)->R<()> {
        // Empty Seeds denotes no PDA signer group, as in the frozen contract.
        let groups=if seeds.is_empty(){Vec::new()}else{vec![seeds.clone()]};
        self.invoke(ix,infos,&groups).map_err(Error::Program)
    }
    fn with_account_data<T,F>(&mut self,a:Account,f:F)->R<T> where F:FnOnce(&[u8])->R<T> {
        let data=self.info(a).try_borrow_data().map_err(Error::Program)?;f(&data)
    }
    fn unpack_token(data:&[u8])->R<(Key,Key,u64)> {
        let token=StateWithExtensions::<SplAccount>::unpack(data).map_err(Error::Program)?;
        Ok((token.base.mint.to_bytes(),token.base.owner.to_bytes(),token.base.amount))
    }
    fn unpack_mint(data:&[u8])->R<u8> {
        Ok(StateWithExtensions::<SplMint>::unpack(data).map_err(Error::Program)?.base.decimals)
    }
    fn log_data(&mut self,data:&[u8])->R<()> {solana_program::log::sol_log_data(&[data]);Ok(())}
    fn set_return_data(&mut self,data:&[u8])->R<()> {set_return_data(data);Ok(())}
}
impl ContextRuntime for SolanaRuntime<'_> {
    type Rent=Rent;
    fn executable(&self,a:Account)->bool {self.info(a).executable}
    fn rent_get(&mut self)->D<Rent> {Rent::get().map_err(context_error)}
    fn minimum_balance(&self,r:&Rent,space:usize)->u64 {r.minimum_balance(space)}
    fn rent_is_exempt(&self,r:&Rent,lamports:u64,len:usize)->bool {r.is_exempt(lamports,len)}
    fn data_len_panicking(&self,a:Account)->usize {self.info(a).data_len()}
    fn with_data<T,F>(&self,a:Account,f:F)->D<T> where F:FnOnce(&[u8])->D<T> {
        let data=self.info(a).try_borrow_data().map_err(context_error)?;f(&data)
    }
    fn find_pda(&self,seeds:&[&[u8]],program:&Key)->(Key,u8) {
        let (p,b)=Pubkey::find_program_address(seeds,&Pubkey::new_from_array(*program));(p.to_bytes(),b)
    }
    fn create_pda(&self,seeds:&[&[u8]],program:&Key)->Result<Key,()> {
        Pubkey::create_program_address(seeds,&Pubkey::new_from_array(*program)).map(|p|p.to_bytes()).map_err(|_|())
    }
    fn token2022_account_space(&self,mint:Account)->D<usize> {
        let data=self.info(mint).try_borrow_data().map_err(context_error)?;
        let state=StateWithExtensions::<SplMint>::unpack(&data).map_err(context_error)?;
        let extensions=state.get_extension_types().map_err(context_error)?;
        let required=ExtensionType::get_required_init_account_extensions(&extensions);
        ExtensionType::try_calculate_account_len::<SplAccount>(&required).map_err(context_error)
    }
}
impl DispatchRuntime for SolanaRuntime<'_> {
    fn log_instruction(&mut self,text:&str) {solana_program::log::sol_log(text);}
    fn log_event_data(&mut self,data:&[u8]) {solana_program::log::sol_log_data(&[data]);}
    fn find_program_address(&mut self,seeds:&[&[u8]],program:&Key)->(Key,u8) {
        <Self as ContextRuntime>::find_pda(self,seeds,program)
    }
    fn report_error(&mut self,error:&Failure) {crate::error_report::report(error);}
}
impl crate::idl_runtime::IdlRuntime for SolanaRuntime<'_> {
    fn idl_owner(&self,a:Account)->Key {self.info(a).owner.to_bytes()}
    fn idl_executable(&self,a:Account)->bool {self.info(a).executable}
    fn idl_data(&self,a:Account)->I<Ref<'_,[u8]>> {
        Ok(Ref::map(self.info(a).try_borrow_data().map_err(program_error)?,|d|&**d))
    }
    fn idl_data_mut(&self,a:Account)->I<RefMut<'_,[u8]>> {
        Ok(RefMut::map(self.info(a).try_borrow_mut_data().map_err(program_error)?,|d|&mut **d))
    }
    fn idl_lamports_panicking(&self,a:Account)->u64 {self.info(a).lamports()}
    fn idl_set_lamports_panicking(&mut self,a:Account,value:u64) {**self.info(a).lamports.borrow_mut()=value;}
    fn idl_assign_system(&mut self,a:Account) {self.info(a).assign(&Pubkey::default());}
    fn idl_realloc_zero_filled(&mut self,a:Account,new_len:usize)->I<()> {
        self.info(a).resize(new_len).map_err(program_error)
    }
    fn idl_rent(&self)->I<crate::idl_runtime::Rent> {
        let r=Rent::get().map_err(program_error)?;
        Ok(crate::idl_runtime::Rent{lamports_per_byte_year:r.lamports_per_byte_year,
            exemption_threshold:r.exemption_threshold,burn_percent:r.burn_percent})
    }
    fn idl_create_with_seed(&self,base:&Key,seed:&[u8],owner:&Key)->Result<Key,ProgramError> {
        // Only the frozen application literal b"anchor:idl" reaches this adapter.
        let seed=std::str::from_utf8(seed).expect("IDL literal seed is UTF-8");
        Pubkey::create_with_seed(&Pubkey::new_from_array(*base),seed,&Pubkey::new_from_array(*owner))
            .map(|p|p.to_bytes()).map_err(ProgramError::from)
    }
    fn idl_invoke_signed(&mut self,ix:Cpi,infos:&[Account],signers:&[Vec<Vec<u8>>])->I<()> {
        self.invoke(ix,infos,signers).map_err(program_error)
    }
}
