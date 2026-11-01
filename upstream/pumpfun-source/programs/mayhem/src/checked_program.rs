//! Preserve the captured program-account validation order.
//!
//! Anchor 1.0.0 uses the all-zero ID as a marker for Program<()> and therefore
//! skips the System program's ID check. The captured Mayhem parser checks the
//! requested ID first (3008), then executable (3009). This source wrapper keeps
//! both checks and delegates the remaining account behavior to Anchor.
use anchor_lang::prelude::*;
use anchor_lang::error::ErrorCode;
use std::{collections::BTreeSet, ops::Deref};

#[derive(Clone)]
pub struct CheckedSystemProgram<'info>(anchor_lang::accounts::program::Program<'info, System>);
#[derive(Debug, Default)]
pub struct CheckedSystemProgramBumps;

impl<'info> TryFrom<&'info AccountInfo<'info>> for CheckedSystemProgram<'info> {
    type Error = anchor_lang::error::Error;
    fn try_from(info: &'info AccountInfo<'info>) -> Result<Self> {
        if info.key != &System::id() {
            return Err(Error::from(ErrorCode::InvalidProgramId).with_pubkeys((*info.key, System::id())).with_account_name("system_program"));
        }
        anchor_lang::accounts::program::Program::try_from(info).map(Self).map_err(|e| e.with_account_name("system_program"))
    }
}

impl<'info> Accounts<'info, CheckedSystemProgramBumps> for CheckedSystemProgram<'info> {
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &'info [AccountInfo<'info>],
        _ix_data: &[u8],
        _bumps: &mut CheckedSystemProgramBumps,
        _reallocs: &mut BTreeSet<Pubkey>,
    ) -> Result<Self> {
        let (info, rest) = accounts.split_first().ok_or_else(|| Error::from(ErrorCode::AccountNotEnoughKeys).with_account_name("system_program"))?;
        *accounts = rest;
        Self::try_from(info)
    }
}

impl ToAccountMetas for CheckedSystemProgram<'_> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        self.0.to_account_metas(is_signer)
    }
}
impl<'info> ToAccountInfos<'info> for CheckedSystemProgram<'info> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> { self.0.to_account_infos() }
}
impl<'info> AsRef<AccountInfo<'info>> for CheckedSystemProgram<'info> {
    fn as_ref(&self) -> &AccountInfo<'info> { self.0.as_ref() }
}
impl<'info> Deref for CheckedSystemProgram<'info> {
    type Target = AccountInfo<'info>;
    fn deref(&self) -> &Self::Target { self.0.as_ref() }
}
impl<'info> AccountsExit<'info> for CheckedSystemProgram<'info> {}
impl Key for CheckedSystemProgram<'_> {
    fn key(&self) -> Pubkey { *self.0.as_ref().key }
}

// These client shapes flatten to the same single readonly account as Program<System>.
pub(crate) mod __client_accounts_checked_system_program {
    use super::*;
    #[derive(AnchorSerialize)]
    pub struct CheckedSystemProgram { pub key: Pubkey }
    impl ToAccountMetas for CheckedSystemProgram {
        fn to_account_metas(&self, _: Option<bool>) -> Vec<AccountMeta> {
            vec![AccountMeta::new_readonly(self.key, false)]
        }
    }
}
pub(crate) mod __cpi_client_accounts_checked_system_program {
    use super::*;
    pub struct CheckedSystemProgram<'info> { pub account: AccountInfo<'info> }
    impl ToAccountMetas for CheckedSystemProgram<'_> {
        fn to_account_metas(&self, _: Option<bool>) -> Vec<AccountMeta> {
            vec![AccountMeta::new_readonly(*self.account.key, false)]
        }
    }
    impl<'info> ToAccountInfos<'info> for CheckedSystemProgram<'info> {
        fn to_account_infos(&self) -> Vec<AccountInfo<'info>> { vec![self.account.clone()] }
    }
}

impl anchor_lang::DuplicateMutableAccountKeys for CheckedSystemProgram<'_> {
    fn duplicate_mutable_account_keys(&self) -> Vec<Pubkey> { vec![] }
}
