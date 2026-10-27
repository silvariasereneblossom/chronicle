//! Range checks for the numbers the napi binding takes from JavaScript
//! (issue #27).
//!
//! The binding used to cast them with `as`, so a negative number became the
//! largest value of its type and a huge one was taken as given: a query limit
//! of -1 asked for an impossible allocation, whose panic can't unwind into
//! Node and aborts the process, and other arguments answered something other
//! than what was asked. Each argument now arrives as an `f64` and is checked
//! here, and a number out of range is refused with an error naming it.
//!
//! Plain Rust with no napi types, so `cargo test`, which builds without the
//! `napi-bindings` feature, covers it.

#![cfg_attr(not(feature = "napi-bindings"), allow(dead_code))]

/// A count, index, offset or size: a finite, whole, non-negative number. A
/// whole number past `usize::MAX` saturates, so a huge limit means "all".
pub(crate) fn to_usize(name: &str, value: f64) -> Result<usize, String> {
    whole(name, value).map(|value| value as usize)
}

/// A sequence number or a duration: as for [`to_usize`], in `u64`.
pub(crate) fn to_u64(name: &str, value: f64) -> Result<u64, String> {
    whole(name, value).map(|value| value as u64)
}

/// A whole number from `min` to `max`, both included.
pub(crate) fn to_u64_in(name: &str, value: f64, min: u64, max: u64) -> Result<u64, String> {
    match whole(name, value).map(|value| value as u64) {
        Ok(n) if (min..=max).contains(&n) => Ok(n),
        Ok(_) => Err(format!(
            "Invalid {name}: {value} (an integer from {min} to {max})"
        )),
        Err(error) => Err(error),
    }
}

fn whole(name: &str, value: f64) -> Result<f64, String> {
    if value.is_finite() && value >= 0.0 && value.fract() == 0.0 {
        Ok(value)
    } else {
        Err(format!("Invalid {name}: {value} (a non-negative integer)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_non_negative_numbers_are_taken() {
        assert_eq!(to_usize("limit", 0.0), Ok(0));
        assert_eq!(to_usize("limit", 3.0), Ok(3));
        assert_eq!(to_usize("limit", -0.0), Ok(0));
        assert_eq!(to_u64("fromSequence", 9_007_199_254_740_991.0), Ok(9_007_199_254_740_991));
    }

    #[test]
    fn negative_fractional_and_non_finite_numbers_are_refused_by_name() {
        for value in [-1.0, -0.5, 2.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = to_usize("limit", value).unwrap_err();
            assert!(error.starts_with("Invalid limit: "), "{error}");
            assert!(error.ends_with("(a non-negative integer)"), "{error}");
            assert!(to_u64("timeoutMs", value).unwrap_err().starts_with("Invalid timeoutMs: "));
        }
        assert_eq!(
            to_usize("offset", -1.0),
            Err("Invalid offset: -1 (a non-negative integer)".to_string())
        );
    }

    #[test]
    fn a_whole_number_past_the_type_saturates() {
        assert_eq!(to_usize("limit", 1e300), Ok(usize::MAX));
        assert_eq!(to_u64("toSequence", 1e300), Ok(u64::MAX));
    }

    #[test]
    fn bounded_numbers_are_refused_outside_their_range() {
        assert_eq!(to_u64_in("bufferSize", 1.0, 1, 1000), Ok(1));
        assert_eq!(to_u64_in("bufferSize", 1000.0, 1, 1000), Ok(1000));
        assert_eq!(
            to_u64_in("bufferSize", 0.0, 1, 1000),
            Err("Invalid bufferSize: 0 (an integer from 1 to 1000)".to_string())
        );
        assert_eq!(
            to_u64_in("bufferSize", 1e9, 1, 1000),
            Err("Invalid bufferSize: 1000000000 (an integer from 1 to 1000)".to_string())
        );
        assert_eq!(
            to_u64_in("bufferSize", -1.0, 1, 1000),
            Err("Invalid bufferSize: -1 (a non-negative integer)".to_string())
        );
    }
}
