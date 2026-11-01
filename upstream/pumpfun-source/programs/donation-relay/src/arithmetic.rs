use crate::*;
// 97/98/100: checked u32 accumulator. This is a handler gate, not helper113 policy.
pub fn admit_weights(shares:&[Share])->R<()> {
 let mut sum=0u32;
 for s in shares {sum=sum.checked_add(u32::from(s.bps)).ok_or_else(overflow)?;}
 require(sum==10000,6022)
}
// Successful value contract of113 under the CALLER's already admitted sum=10000.
// Stable library sort is a natural hypothesis, not historical allocation/sort identity.
// No public arbitrary-weight helper contract is claimed.
pub(crate) fn fair_amounts(total:u64,shares:&[Share])->Vec<u64> {
 let mut amounts=Vec::with_capacity(shares.len());let mut remainders=Vec::with_capacity(shares.len());let mut allocated=0u64;
 for (i,s) in shares.iter().enumerate() {
  let product=u128::from(total)*u128::from(s.bps);
  let q=(product/10000) as u64;allocated+=q;amounts.push(q);
  remainders.push((i,product%10000));
 }
 // Stable decreasing remainder; equal keys retain original occurrence order.
 remainders.sort_by(|a,b|b.1.cmp(&a.1));
 let left=total-allocated;
 for (i,_) in remainders.into_iter().take(left as usize) {amounts[i]+=1;}
 amounts
}
