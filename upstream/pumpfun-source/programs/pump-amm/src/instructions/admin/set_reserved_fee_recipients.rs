use crate::*;

/// One primary reserved fee recipient plus seven additional ones.
pub const RESERVED_FEE_RECIPIENTS_LEN: usize = 8;

#[event_cpi]
#[derive(Accounts)]
pub struct SetReservedFeeRecipients<'info> {
    #[account(
        mut,
        seeds = [b"global_config"],
        bump,
        has_one = admin @ PumpAmmError::InvalidAdmin,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub admin: Signer<'info>,
}

/// Sets the reserved fee recipients and the whitelist PDA.
///
/// The reserved fee recipients are passed as the first
/// `RESERVED_FEE_RECIPIENTS_LEN` remaining accounts. The first one becomes
/// `GlobalConfig::reserved_fee_recipient`, the other seven become
/// `GlobalConfig::reserved_fee_recipients`.
///
/// Every recipient must be a non-default, non-executable, rent exempt
/// account so that fee transfers to it can never fail.
pub fn handler(ctx: Context<SetReservedFeeRecipients>, whitelist_pda: Pubkey) -> Result<()> {
    let remaining_accounts = ctx.remaining_accounts;
    let global_config = &mut ctx.accounts.global_config;
    // remaining_accounts:
    //   [0]    reserved_fee_recipient
    //   [1..8] reserved_fee_recipients
    //
    // Any further remaining accounts are ignored. The admin is trusted to
    // pass distinct accounts; duplicates are not rejected, they only reduce
    // the number of distinct reserved fee recipients the trade instructions
    // can pick from.
    require_gte!(
        remaining_accounts.len(),
        RESERVED_FEE_RECIPIENTS_LEN,
        PumpAmmError::NotEnoughRemainingAccounts
    );

    let recipients: [Pubkey; RESERVED_FEE_RECIPIENTS_LEN] = remaining_accounts[..RESERVED_FEE_RECIPIENTS_LEN]
        .iter()
        .map(|account| account.key())
        .collect::<Vec<_>>()
        .try_into().map_err(|_| ErrorCode::AccountNotEnoughKeys)
        .unwrap();

    let rent = Rent::get()?;
    {
        let mut accounts = remaining_accounts
            .iter()
            .take(RESERVED_FEE_RECIPIENTS_LEN)
            .enumerate();
        while let Some((i, account)) = accounts.next() {
            require_keys_neq!(
                account.key(),
                Pubkey::default(),
                PumpAmmError::AllProtocolFeeRecipientsShouldBeNonZero
            );
            if account.executable {
                return Err(Error::from(ErrorCode::ConstraintExecutable)
                    .with_account_name(format!("reserved_fee_recipients[{}]", i)));
            }
            if unlikely(!rent.is_exempt(account.lamports(), account.try_data_len()?)) {
                return Err(Error::from(ErrorCode::ConstraintRentExempt)
                    .with_account_name(format!("reserved_fee_recipients[{}]", i)));
            }
        }
    }

    global_config.whitelist_pda = whitelist_pda;
    global_config.reserved_fee_recipient = recipients[0];
    global_config.reserved_fee_recipients.copy_from_slice(&recipients[1..]);

    let event = ReservedFeeRecipientsEvent {
        timestamp: Clock::get()?.unix_timestamp,
        reserved_fee_recipient: global_config.reserved_fee_recipient,
        reserved_fee_recipients: global_config.reserved_fee_recipients,
    };
    emit!(event);
    emit_cpi!(event);

    Ok(())
}


#[inline]
#[cold]
fn cold() {}

#[inline]
fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}
