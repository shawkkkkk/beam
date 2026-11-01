//! Host differential checks against the unmodified, pinned reconstructed crate.
//! These compare selected math/layout/encoding paths, not complete SBF programs.
use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData};
use beam_program as beam;
use pump_reference as reference;

fn encoded_curve(vt: u64, vq: u64) -> Vec<u8> {
    let mut data = vec![0;166];
    data[..8].copy_from_slice(&[23,183,248,55,96,216,172,96]);
    for (index,n) in [vt,vq,793_100_000_000_000,0,1_000_000_000_000_000].into_iter().enumerate() {
        data[8+index*8..16+index*8].copy_from_slice(&n.to_le_bytes());
    }
    data
}
fn result<T: std::fmt::Debug>(r: anchor_lang::Result<T>) -> String {
    match r { Ok(v) => format!("ok:{v:?}"), Err(anchor_lang::error::Error::AnchorError(e)) => {
            let source = match e.error_origin { Some(anchor_lang::error::ErrorOrigin::Source(s)) => Some((s.filename.rsplit('/').next().unwrap().to_string(),s.line)), _ => None };
            format!("err:{}:{}:{}:{:?}:{source:?}",e.error_name,e.error_code_number,e.error_msg,e.compared_values)
        },
        Err(anchor_lang::error::Error::ProgramError(e)) => format!("program:{:?}:{:?}",e.program_error,e.compared_values) }
}

#[test]
fn curve_layout_decoder_and_serialization_match_at_every_truncation_boundary() {
    let data = encoded_curve(1_073_000_000_000_000,30_000_000_000);
    for len in 0..=data.len() {
        let a = beam::trade_types::BondingCurve::try_deserialize(&mut &data[..len]);
        let b = reference::trade_types::BondingCurve::try_deserialize(&mut &data[..len]);
        assert_eq!(a.is_ok(),b.is_ok(),"length {len}");
        match (a,b) {
            (Ok(a),Ok(b)) => { let(mut aa,mut bb)=(Vec::new(),Vec::new());a.try_serialize(&mut aa).unwrap();b.try_serialize(&mut bb).unwrap();assert_eq!(aa,bb,"length {len}"); },
            (Err(a),Err(b)) => assert_eq!(a.to_string(),b.to_string(),"length {len}"),
            _ => unreachable!(),
        }
    }
}

#[test]
fn buy_sell_and_exact_input_math_match_reference_across_boundaries() {
    for vt in [1,2,1000,1_073_000_000_000_000,u64::MAX] {
        for vq in [0,1,30_000_000_000,u64::MAX] {
            let data = encoded_curve(vt,vq);
            let a = beam::trade_types::BondingCurve::try_deserialize(&mut &data[..]).unwrap();
            let b = reference::trade_types::BondingCurve::try_deserialize(&mut &data[..]).unwrap();
            for amount in [0,1,2,100,vt.saturating_sub(1),vt,u64::MAX] {
                assert_eq!(result(beam::buy_quote(&a,amount)),result(reference::buy_quote(&b,amount)));
                assert_eq!(result(a.sell_quote(amount)),result(b.sell_quote(amount)));
                assert_eq!(result(beam::tokens_for_quote(&a,amount)),result(reference::tokens_for_quote(&b,amount)));
            }
        }
    }
}

#[test]
fn fee_rounding_matches_reference_including_u64_boundaries() {
    for amount in [0,1,199,200,201,10_000,1_000_000_000,u64::MAX] {
        for bps in [0,1,50,100,10_000,u64::MAX] {
            assert_eq!(result(beam::fee_math::fee(amount,bps)),result(reference::fee_math::fee(amount,bps)));
        }
    }
}

#[test]
fn retained_instruction_encodings_match() {
    assert_eq!(beam::instruction::Initialize {}.data(),reference::instruction::Initialize {}.data());
    for (amount,limit) in [(0,0),(1,9),(u64::MAX,u64::MAX)] {
        let a = beam::instruction::Buy { amount,max_sol_cost:limit,track_volume:beam::OptionBool(true),partial_fill:beam::OptionBool(false) }.data();
        let b = reference::instruction::Buy { amount,max_sol_cost:limit,track_volume:reference::OptionBool(true),partial_fill:reference::OptionBool(false) }.data();
        assert_eq!(a,b);
        assert_eq!(beam::instruction::Sell { amount,min_sol_output:limit }.data(),reference::instruction::Sell { amount,min_sol_output:limit }.data());
    }
}
