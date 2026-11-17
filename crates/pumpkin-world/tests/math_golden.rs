//! Differential tests for `f64::log10` and `f64::powf` against Java `StrictMath`.
//!
//! Vanilla worldgen calls `Math.log10` and `Math.pow`, which Java routes through
//! fdlibm (`StrictMath`) for reproducibility. Rust uses the platform libm, so the
//! results can differ by a few ulps. The golden data was produced by
//! `StrictMath` on `OpenJDK` 21 with `new Random(42)`; each line lists raw IEEE-754
//! bits as hex, input(s) first and the expected result last.
//!
//! Measured on x86_64 Linux/glibc 2.4x: `log10` differs from `StrictMath` in 18
//! of 4000 cases (13 by 1 ulp, 5 by 2 ulp), `pow` in 417 of 4000 cases (all 1
//! ulp). The bounds below encode the observed gap; exact equality would require
//! porting fdlibm.

const LOG10_GOLDEN: &str = include_str!("data/log10_golden.txt");
const POW_GOLDEN: &str = include_str!("data/pow_golden.txt");

/// Number of representable doubles between two values, treating the sign bit as
/// a continuation of the magnitude ordering.
const fn ulp_distance(a: f64, b: f64) -> u64 {
    const fn ordered(value: f64) -> i64 {
        let bits = value.to_bits() as i64;
        if bits < 0 {
            i64::MIN.wrapping_sub(bits)
        } else {
            bits
        }
    }

    ordered(a).abs_diff(ordered(b))
}

#[expect(
    clippy::expect_used,
    reason = "golden data is a checked-in fixture; a bad line is a test bug"
)]
fn parse_bits(token: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(token, 16).expect("golden data must be hex bits"))
}

fn lines(data: &str) -> impl Iterator<Item = Vec<&str>> {
    data.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().collect())
}

#[test]
fn log10_matches_strictmath_within_two_ulp() {
    let mut checked = 0usize;
    let mut max_ulp = 0u64;

    for fields in lines(LOG10_GOLDEN) {
        assert_eq!(fields.len(), 2, "malformed log10 golden line: {fields:?}");
        let input = parse_bits(fields[0]);
        let expected = parse_bits(fields[1]);

        max_ulp = max_ulp.max(ulp_distance(input.log10(), expected));
        checked += 1;
    }

    assert_eq!(checked, 4000, "golden log10 vector count changed");
    assert!(
        max_ulp <= 2,
        "f64::log10 drifted {max_ulp} ulp from StrictMath"
    );
}

#[test]
fn pow_matches_strictmath_within_one_ulp() {
    let mut checked = 0usize;
    let mut max_ulp = 0u64;

    for fields in lines(POW_GOLDEN) {
        assert_eq!(fields.len(), 3, "malformed pow golden line: {fields:?}");
        let base = parse_bits(fields[0]);
        let exponent = parse_bits(fields[1]);
        let expected = parse_bits(fields[2]);

        max_ulp = max_ulp.max(ulp_distance(base.powf(exponent), expected));
        checked += 1;
    }

    assert_eq!(checked, 4000, "golden pow vector count changed");
    assert!(
        max_ulp <= 1,
        "f64::powf drifted {max_ulp} ulp from StrictMath"
    );
}
