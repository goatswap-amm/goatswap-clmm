pub use crate::curve::fees::floor_div;
use crate::curve::FEE_RATE_DENOMINATOR_VALUE;

/// Calculate tax amount
pub fn tax_amount(amount: u64, tax_rate: u64) -> Option<u128> {
    floor_div(
        u128::from(amount),
        u128::from(tax_rate),
        u128::from(FEE_RATE_DENOMINATOR_VALUE),
    )
}
