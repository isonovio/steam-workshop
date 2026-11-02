use super::*;

#[test]
fn rejects_zero_and_expired_budgets() {
    assert!(Timeout::new(Duration::ZERO).check().is_none());
    let expired = Timeout {
        start: Instant::now().checked_sub(Duration::from_secs(2)).unwrap(),
        duration: Duration::from_secs(1),
    };
    assert!(expired.check().is_none());
    assert_eq!(expired.remaining(), Duration::ZERO);
}

#[test]
fn checking_and_copying_preserve_the_original_budget() {
    let timeout = Timeout {
        start: Instant::now().checked_sub(Duration::from_secs(30)).unwrap(),
        duration: Duration::from_secs(60),
    };
    let checked = timeout.check().unwrap();
    assert_eq!(checked.start, timeout.start);
    assert_eq!(checked.duration, timeout.duration);
    assert!(checked.remaining() <= Duration::from_secs(30));
    let next = checked.check().unwrap();
    assert_eq!(next.start, timeout.start);
    assert_eq!(next.duration, timeout.duration);
}
