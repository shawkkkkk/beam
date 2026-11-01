












































{
        require!(new_authority != Pubkey::default(), FeeError::InvalidAdmin);
        let old_authority = ctx.accounts.fee_program_global.authority;
        ctx.accounts.fee_program_global.authority = new_authority;
        let event = SetAuthorityEvent { timestamp: Clock::get()?.unix_timestamp, old_authority, new_authority };
        emit!(event);
        emit_cpi!(event);
        Ok(())
}
