use anchor_lang::prelude::*; use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint,TokenAccount,TokenInterface}}; use crate::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};
pub const PUMP_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");









#[account]
pub struct GlobalParams {
    pub field_0: u32, // serialized 8; original object 232
    pub field_1: u32, // serialized 12; original object 236
    pub field_2: u64, // serialized 16; original object 96
    pub field_3: u64, // serialized 24; original object 104
    pub field_4: u64, // serialized 32; original object 112
    pub field_5: u64, // serialized 40; original object 120
    pub paused: bool, // serialized 48; original object 245
    pub admin: Pubkey, // serialized 49; original object 0
    pub trader: Pubkey, // serialized 81; original object 32
    pub sol_deployed: i128, // serialized 113; original object 64
    pub field_9: u64, // serialized 129; original object 128
    pub field_6: u8, // serialized 137; original object 244
    pub field_138: u64, // serialized 138; original object 136
    pub field_7: u64, // serialized 146; original object 144
    pub field_8: u64, // serialized 154; original object 152
    pub version: u32, // serialized 162; original object 240
    pub field_10: u64, // serialized 166; original object 160
    pub max_sol_per_sweep: u64, // serialized 174; original object 168
    pub field_11: u64, // serialized 182; original object 176
    pub field_12: u64, // serialized 190; original object 184
    pub field_13: u64, // serialized 198; original object 192
    pub stable_deployed: i128, // serialized 206; original object 80
    pub field_14: u64, // serialized 222; original object 200
    pub field_15: u64, // serialized 230; original object 208
    pub field_238: u64, // serialized 238; original object 216
    pub max_stable_per_sweep: u64, // serialized 246; original object 224
}

/// Serialized offsets are recovered from 0x685f0 and 0x68ce0.
/// Neutral field names avoid claiming unavailable historical semantics.













#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MayhemStatus { Variant0, Variant1, Ended }

/// A separate namespace suppresses the automatic Mayhem Owner implementation.
/// The codec at 0x6dc00 writes 82 bytes including this account discriminator.
pub use crate::BondingCurve;









































impl GlobalParams {
    pub fn validate_native_withdraw(&mut self)->Result<()> { let current_time = Clock::get()?.unix_timestamp; let last = self.field_138 as i64; let interval = self.field_8 as i64;
        if last != 0 && interval != 0 {
        let next =    last + interval;


                                   require!(current_time >= next, crate::MayhemError::WithdrawRateLimit);
        }
        self.field_138 = current_time as u64;
        Ok(())
    }





    pub fn validate_quote_withdraw(&mut self)->Result<()> { let current_time = Clock::get()?.unix_timestamp; let last = self.field_238 as i64; let interval = self.field_15 as i64;
        if last != 0 && interval != 0 {
        let next =    last + interval;


                                   require!(current_time >= next, crate::MayhemError::WithdrawRateLimit);
        }
        self.field_238 = current_time as u64;
        Ok(())
    }
}

















































































impl GlobalParams {
    pub fn stop_trading(&mut self)->Result<()> {

                                   require_eq!(self.paused, false, crate::MayhemError::TradingPaused);
        self.paused = true;
        Ok(())
    }
    pub fn resume_trading(&mut self)->Result<()> {

                                   require_eq!(self.paused, true, crate::MayhemError::TradingPaused);
        self.paused = false;
        Ok(())
    }
}




pub fn record_amm_buy(global_params:&mut GlobalParams,state:&mut MayhemState,quote:u64,tokens:u64,quote_mint:&Pubkey)->Result<()> {
    if *quote_mint==Pubkey::default() || *quote_mint==anchor_spl::token::spl_token::native_mint::ID {
        global_params.sol_deployed=global_params.sol_deployed.checked_add(quote as i128).ok_or(crate::MayhemError::MathOverflow)?;
    } else { global_params.stable_deployed=global_params.stable_deployed.checked_add(quote as i128).ok_or(crate::MayhemError::MathOverflow)?; }
    state.field_56=state.field_56.checked_add(quote as i128).ok_or(crate::MayhemError::MathOverflow)?;
    state.field_72=state.field_72.checked_sub(tokens as i128).ok_or(crate::MayhemError::MathOverflow)?;
    validate_amm_buy_risk(global_params,state,quote_mint)?; Ok(()) } #[inline(always)] fn validate_amm_buy_risk(global_params:&GlobalParams,state:&MayhemState,quote_mint:&Pubkey)->Result<()> { let stable=*quote_mint!=Pubkey::default() && *quote_mint!=anchor_spl::token::spl_token::native_mint::ID;
    let global_deployed=if stable {global_params.stable_deployed}else{global_params.sol_deployed};
    let overall_limit=if stable {global_params.field_12}else{global_params.field_4};
    let coin_limit=if stable {global_params.field_13}else{global_params.field_5};


    if global_deployed!=0 && global_deployed>=0 {
        require_gt!(overall_limit,global_deployed as u64,crate::MayhemError::OverallRiskExceeded);
    }




    if state.field_56>0 {
        require_gt!(coin_limit,state.field_56 as u64,crate::MayhemError::CoinRiskExceeded);
    }
    Ok(())
}
























































pub(crate) fn retained_basis_points(retained_bps:u16)->u16 {










          let retained = 10000u16 - retained_bps;
    retained
}
















#[inline(always)]
fn initial_trade_output<C:TradeCap>(vt:u128,c:&curve_accounts::BondingCurve,cap:C,tokens:u128,retained_bps:u16)->u64 {
 let vs=c.virtual_sol_reserves as u128;let raw_quote=vs*tokens/(vt+tokens);







  let mut output=u64::try_from(raw_quote).unwrap();
 let real_sol_reserves=c.real_sol_reserves;output=cap.early(output);let retained=retained_basis_points(retained_bps);let max=(real_sol_reserves as u128*retained as u128/10_000)as u64;
 output=cap.late(output);
 output.min(max)
}





#[inline(always)]
pub fn buy_amounts<C:TradeCap>(vt:u128,c:&curve_accounts::BondingCurve,cap:C,tokens:u128,retained_bps:u16)->(u64,u64) {
 let retained=retained_basis_points(retained_bps);let output=initial_trade_output(vt,c,cap,tokens,retained_bps);let vs=c.virtual_sol_reserves as u128;














 let ratio=(output as u128*vt).checked_div(vs+output as u128)
                                        .ok_or(crate::MayhemError::PumpCpiMathOverflow).unwrap();
  let input=u64::try_from(ratio).unwrap().min(c.real_token_reserves);
 (input,output)
}



#[inline(always)]
pub fn sell_amounts<C:TradeCap>(vt:u64,c:&curve_accounts::BondingCurve,cap:C,balance:u64,retained_bps:u16)->(u64,u64) {
    let retained=retained_basis_points(retained_bps);
    let output=initial_trade_output(vt as u128,c,cap,balance as u128,retained_bps);let vs=c.virtual_sol_reserves;
    let ratio=vs.checked_sub(output).and_then(|denominator| if denominator == 0 { None } else { Some(output as u128*vt as u128/denominator as u128) })
        .ok_or(crate::MayhemError::PumpCpiMathOverflow)







                                                                                        .unwrap();
  let input=u64::try_from(ratio).unwrap();
    (input,output.saturating_sub(1))
}
















fn initial_quote(base_reserve:u64,quote_reserve:u64,balance:u64)->u64 {
    match (balance as u128*quote_reserve as u128).checked_div(base_reserve as u128+balance as u128) {



        Some(value)=>{
                     u64::try_from(value).unwrap()
        },
        None=>u64::MAX,
    }
}





















pub fn buy_quote(global_params: &GlobalParams, base_reserve: u64, quote_reserve: u64, balance: u64, retained_bps: u16, quote_mint: Pubkey) -> Result<(u64,u64)> {
    let _ = retained_basis_points(retained_bps);
    let quote=initial_quote(base_reserve,quote_reserve,balance);
    let cap = if quote_mint == Pubkey::default() || quote_mint == anchor_spl::token::spl_token::native_mint::id() { &global_params.field_2 } else { &global_params.field_11 };
    let quote = if *cap != 0 { quote.min(*cap) } else { quote };



    let retained=retained_basis_points(retained_bps); let quote=quote.min((quote_reserve as u128*retained as u128/10000)as u64);

    let input=(quote as u128*base_reserve as u128)
        .checked_div(quote_reserve as u128+quote as u128)
        .ok_or(crate::MayhemError::PumpSwapCpiMathOverflow)
            .unwrap();
  let input=u64::try_from(input).unwrap();
    Ok((input,quote))
}










pub fn sell_quote(global_params: &GlobalParams, base_reserve: u64, quote_reserve: u64, balance: u64, retained_bps: u16, quote_mint: Pubkey) -> Result<(u64,u64)> {
    let _ = retained_basis_points(retained_bps);
    let quote=initial_quote(base_reserve,quote_reserve,balance);
    let cap = if quote_mint == Pubkey::default() || quote_mint == anchor_spl::token::spl_token::native_mint::id() { global_params.field_2 } else { global_params.field_11 };
    let quote = if cap != 0 { quote.min(cap) } else { quote };



    let retained=retained_basis_points(retained_bps); let quote=quote.min((quote_reserve as u128*retained as u128/10000)as u64);

    let input=(||->std::result::Result<u128,crate::MayhemError>{ let remaining=quote_reserve.checked_sub(quote).ok_or(crate::MayhemError::PumpSwapCpiMathOverflow)?;
        (quote as u128*base_reserve as u128).checked_div(remaining as u128).ok_or(crate::MayhemError::PumpSwapCpiMathOverflow) })()

            .unwrap();
  let input=u64::try_from(input).unwrap();
    Ok((input,quote.saturating_sub(1)))
}











#[account]
pub struct MayhemState {
    pub field_8: u64,             // 8
    pub end_timestamp: i64,        // 16: signed Clock comparison in burn
    pub mint: Pubkey,              // 24
    pub field_56: i128,            // 56: exact width; signedness not proven here
    pub field_72: i128,            // 72: exact width; signedness not proven here
    pub flag_88: bool,             // 88: decoder accepts only 0 or 1
    pub field_89: u64,             // 89
    pub status: MayhemStatus,      // 97: decoder accepts only 0, 1 or 2
    pub burned_amount: u64,        // 98
}










#[inline(always)]
fn initialize_state(mint_key:&Pubkey, params:&crate::GlobalParams, state:&mut MayhemState)->Result<()> {

    let duration_hours=params.field_6 as i64;
    let now=                 Clock::get().unwrap().unix_timestamp; state.field_8=now as u64;
    state.end_timestamp=       now+duration_hours*3600;
    state.mint=*mint_key;
    state.field_56=0;
    state.field_89=now as u64;
    state.flag_88=false;
    state.field_72=0;
    state.status=MayhemStatus::Variant0;
    state.burned_amount=0;
    Ok(())



}


pub fn initialize(ctx:Context<crate::InitializeMayhemState>)->Result<()> { let mut snapshot=InitMint {key:std::mem::MaybeUninit::uninit(),info:ctx.accounts.mint.to_account_info()}; snapshot.key.write(snapshot.info.key()); initialize_state(unsafe { snapshot.key.assume_init_ref() }, &ctx.accounts.global_params, &mut ctx.accounts.mayhem_state) }
#[repr(C)] struct InitMint<'info> { key:std::mem::MaybeUninit<Pubkey>, info:AccountInfo<'info> }




impl MayhemState {
    pub fn validate_trade(&mut self, global_params: &crate::GlobalParams) -> Result<()> {

        require!(!global_params.paused, crate::MayhemError::TradingPaused);

        let current_time = Clock::get().unwrap().unix_timestamp;
        let last = self.field_89 as i64;
        let interval = (global_params.field_7 as i64);
        if last != 0 && interval != 0 {
               let next_time = last + interval;


            require!(current_time >= next_time, crate::MayhemError::TradeRateLimit);
        }
        self.field_89 = current_time as u64;
        Ok(())
    }
}


impl MayhemState {
    pub fn validate_quote(&mut self, reserves: u64, global_params: &crate::GlobalParams, quote_mint: &Pubkey) -> Result<()> {


        let end_timestamp = self.end_timestamp;
                let current_time = Clock::get().unwrap().unix_timestamp;
        require!(end_timestamp > current_time, crate::MayhemError::MayhemModeEnded);
        if self.status == MayhemStatus::Variant1 { return Ok(()); }

        let threshold = if *quote_mint == Pubkey::default() || *quote_mint == anchor_spl::token::spl_token::native_mint::id() {
            global_params.field_9
        } else { global_params.field_14 };








        require!(reserves >= threshold, crate::MayhemError::SolReservesTooLow);
        self.status = MayhemStatus::Variant1;
        Ok(())
    }
}


pub fn record_sell(global_params:&mut GlobalParams,state:&mut MayhemState,quote:u64,tokens:u64,quote_mint:&Pubkey)->Result<()> {
    if *quote_mint==Pubkey::default() || *quote_mint==anchor_spl::token::spl_token::native_mint::ID {
        global_params.sol_deployed=global_params.sol_deployed.checked_sub(quote as i128).ok_or(crate::MayhemError::MathOverflow)?;
    } else { global_params.stable_deployed=global_params.stable_deployed.checked_sub(quote as i128).ok_or(crate::MayhemError::MathOverflow)?; }
    state.field_56=state.field_56.checked_sub(quote as i128).ok_or(crate::MayhemError::MathOverflow)?;
    state.field_72=state.field_72.checked_add(tokens as i128).ok_or(crate::MayhemError::MathOverflow)?;
    Ok(())
} #[derive(Accounts)] pub struct InitializeMayhemState<'info> { #[account(mut)] pub payer: Signer<'info>, #[account(seeds = [b"global-params"], bump)] pub global_params: Account<'info, GlobalParams>, #[account(signer, seeds = [b"mint-authority"], bump, seeds::program = PUMP_ID)] pub mint_authority: UncheckedAccount<'info>, #[account(mut, seeds = [b"sol-vault"], bump)] pub sol_vault: UncheckedAccount<'info>, pub mint: InterfaceAccount<'info, Mint>, #[account(init_if_needed, payer = payer, associated_token::mint = mint, associated_token::authority = sol_vault, associated_token::token_program = token_program)] pub mayhem_token_vault: InterfaceAccount<'info, TokenAccount>, #[account(init, payer = payer, space = 106, seeds = [b"mayhem-state", mint.key().as_ref()], bump)] pub mayhem_state: Account<'info, crate::state::MayhemState>, pub system_program: Program<'info,System>, pub token_program: Program<'info,anchor_spl::token_2022::Token2022>, pub associated_token_program: Program<'info, AssociatedToken>, }

impl GlobalParams {
    pub fn apply_update(&mut self,field_0:Option<u32>,field_1:Option<u32>,field_2:Option<u64>,field_3:Option<u64>,field_4:Option<u64>,field_5:Option<u64>,field_6:Option<u8>,field_7:Option<u64>,field_8:Option<u64>,field_9:Option<u64>,field_10:Option<u64>,field_11:Option<u64>,field_12:Option<u64>,field_13:Option<u64>,field_14:Option<u64>,field_15:Option<u64>)->Result<()> {
        if let Some(v) = field_0 { self.field_0 = v; }
        if let Some(v) = field_1 { self.field_1 = v; }
        if let Some(v) = field_2 { self.field_2 = v; }
        if let Some(v) = field_3 { self.field_3 = v; }
        if let Some(v) = field_4 { self.field_4 = v; }
        if let Some(v) = field_5 { self.field_5 = v; }
        if let Some(v) = field_6 { self.field_6 = v; }
        if let Some(v) = field_7 { self.field_7 = v; }
        if let Some(v) = field_8 { self.field_8 = v; }
        if let Some(v) = field_9 { self.field_9 = v; }
        if let Some(v) = field_10 { self.field_10 = v; }
        if let Some(v) = field_11 { self.field_11 = v; }
        if let Some(v) = field_12 { self.field_12 = v; }
        if let Some(v) = field_13 { self.field_13 = v; }
        if let Some(v) = field_14 { self.field_14 = v; }
        if let Some(v) = field_15 { self.field_15 = v; }
        self.version=self.version.checked_add(1).ok_or(crate::MayhemError::MathOverflow)?;
        Ok(())
    }
}

pub trait TradeCap:Sized { #[inline(always)] fn early(&self,output:u64)->u64 {output} #[inline(always)] fn late(&self,output:u64)->u64 {output} }
impl TradeCap for &u64 { #[inline(always)] fn late(&self,output:u64)->u64 { if **self!=0 {output.min(**self)} else {output} } }
pub struct QuoteCap<'a>{pub global_params:&'a GlobalParams,pub quote_mint:Pubkey}
impl TradeCap for QuoteCap<'_> { fn early(&self,output:u64)->u64 { let cap=if self.quote_mint==Pubkey::default()||self.quote_mint==anchor_spl::token::spl_token::native_mint::id(){&self.global_params.field_2}else{&self.global_params.field_11}; if *cap!=0 {output.min(*cap)} else {output} } }
