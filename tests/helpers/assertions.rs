/// Custom assertions for testing
///
/// Provides domain-specific assertion helpers

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;

/// Assert that two timestamps are within a duration of each other
pub fn assert_timestamp_near(left: DateTime<Utc>, right: DateTime<Utc>, tolerance: Duration) {
    let diff = (left - right).abs();
    assert!(
        diff <= tolerance,
        "Timestamps not within tolerance: {} vs {} (diff: {})",
        left, right, diff
    );
}

/// Assert that a decimal is positive
pub fn assert_decimal_positive(value: Decimal) {
    assert!(
        value > Decimal::ZERO,
        "Expected positive decimal, got: {}",
        value
    );
}

/// Assert that a decimal is non-negative
pub fn assert_decimal_non_negative(value: Decimal) {
    assert!(
        value >= Decimal::ZERO,
        "Expected non-negative decimal, got: {}",
        value
    );
}

/// Assert that a decimal is within a range
pub fn assert_decimal_in_range(value: Decimal, min: Decimal, max: Decimal) {
    assert!(
        value >= min && value <= max,
        "Expected decimal in range [{}, {}], got: {}",
        min, max, value
    );
}

/// Assert that a collection is not empty
pub fn assert_not_empty<T>(collection: &[T]) {
    assert!(!collection.is_empty(), "Expected non-empty collection");
}

/// Assert that a collection has a specific length
pub fn assert_length<T>(collection: &[T], expected: usize) {
    assert_eq!(
        collection.len(),
        expected,
        "Expected collection length {}, got {}",
        expected,
        collection.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_timestamp_near_assertion() {
        let now = Utc::now();
        let almost_now = now + Duration::milliseconds(100);
        assert_timestamp_near(now, almost_now, Duration::seconds(1));
    }

    #[test]
    #[should_panic]
    fn test_timestamp_near_assertion_fails() {
        let now = Utc::now();
        let later = now + Duration::hours(1);
        assert_timestamp_near(now, later, Duration::seconds(1));
    }

    #[test]
    fn test_decimal_assertions() {
        assert_decimal_positive(Decimal::new(1, 0));
        assert_decimal_non_negative(Decimal::ZERO);
        assert_decimal_in_range(Decimal::new(5, 0), Decimal::ZERO, Decimal::new(10, 0));
    }

    #[test]
    fn test_collection_assertions() {
        let vec = vec![1, 2, 3];
        assert_not_empty(&vec);
        assert_length(&vec, 3);
    }
}
