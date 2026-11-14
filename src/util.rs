use astro_float::{BigFloat, RoundingMode};

// because BigFloat::round is... broken, or at least doesn't act as expected. when set to RoundingMode::FromZero, will round negative numbers less than half down anyway. (???)
#[must_use]
pub fn sane_round(x: &BigFloat) -> BigFloat {
    if x.is_negative() {
        return sane_round(&x.neg()).neg()
    }
    if *x >= x.int().add_full_prec(&BigFloat::from(0.5)) {
        return x.ceil()
    }
    x.floor()
}

/// Convert's `astro_float`'s `BigFloat` type into a `usize`.
/// 
/// # Panics
/// 
/// Panics if we run out of memory, or if `x` is a NaN or infinite value.
// WARNING: feeding these next two functions NaNs or infinities is a bad idea.
#[must_use]
#[allow(clippy::cast_possible_truncation)] // this is expected when casting to usize, and i'm not testing this on 32-bit targets atm
pub fn bf_to_usize_unchecked(x: &BigFloat) -> usize {
    let mut xc = x.int();

    xc.set_precision(64, RoundingMode::None).unwrap();

    let parts = xc.as_raw_parts().unwrap();

    if parts.0[0] == 0 {
        0
    }
    else {
        (parts.0[0] >> (64 - parts.3)) as usize
    }
}

/// Convert's `astro_float`'s `BigFloat` type into an `i64`.
/// 
/// # Panics
/// 
/// Panics if we run out of memory, or if `x` is a NaN or infinite value.
#[must_use]
pub fn bf_to_i64_unchecked(x: &BigFloat) -> i64 {
    let mut xc = x.int();

    xc.set_precision(64, RoundingMode::None).unwrap();

    let parts = xc.as_raw_parts().unwrap();
    let sign = match parts.2 {
        astro_float::Sign::Neg => -1,
        astro_float::Sign::Pos => 1,
    };

    if parts.0[0] == 0 {
        0
    }
    else {
        (parts.0[0] >> (64 - parts.3)).cast_signed() * sign
    }
}
