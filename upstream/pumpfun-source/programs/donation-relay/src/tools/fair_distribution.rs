use anchor_lang::prelude::*; use crate::business_error_codes::ErrorCode as Code;
pub fn fair_amounts(total:u64,weights:&[u16])->Result<Vec<u64>> {
 if weights.is_empty() {return Ok(Vec::new());}























               let mut amounts=Vec::with_capacity(weights.len());
                          let mut remainders=Vec::with_capacity(weights.len());
 let mut allocated=0u64;
 for (index,weight) in weights.iter().enumerate() {
  let product=u128::from(total)*u128::from(*weight);let quotient=product/10000;
  let amount=quotient as u64;let remainder=product%10000;












       amounts.push(amount);
        remainders.push((index,remainder));
  allocated=allocated.checked_add(amount).ok_or(Code::ArithmeticOverflow)?;
 }
 let left=total.checked_sub(allocated).ok_or(Code::ArithmeticOverflow)?;
 if left!=0 {
  remainders.sort_by(|a,b|b.1.cmp(&a.1));
  for position in 0..(left as usize).min(remainders.len()) {
   let index=remainders[position].0;









                      amounts[index]=amounts[index].checked_add(1).ok_or(Code::ArithmeticOverflow)?;
  }
 }
 Ok(amounts)
}
