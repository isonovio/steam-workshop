use super::*;

fn call() -> ApiCall<'static, ()> {
    ApiCall::new(42, 7, 8, |_| Ok(())).unwrap()
}

#[test]
fn rejects_invalid_handle() {
    assert!(ApiCall::new(0, 7, 8, |_| Ok(())).is_none());
}

#[test]
fn ignores_other_calls_and_checks_matching_descriptor() {
    let call = call();
    assert_eq!(
        call.matches(&Completion {
            handle: 43,
            callback_id: 99,
            size: 0
        }),
        Ok(false)
    );
    assert_eq!(
        call.matches(&Completion {
            handle: 42,
            callback_id: 7,
            size: 8
        }),
        Ok(true)
    );
    assert_eq!(
        call.matches(&Completion {
            handle: 42,
            callback_id: 8,
            size: 8
        }),
        Err(ApiError::InvalidResult)
    );
    assert_eq!(
        call.matches(&Completion {
            handle: 42,
            callback_id: 7,
            size: 9
        }),
        Err(ApiError::InvalidResult)
    );
}

impl<T> ApiCall<'_, T> {
    pub(crate) fn unpolled_for_test() -> Self {
        Self::new(1, 1, 1, |_| panic!("test call must not be decoded")).unwrap()
    }
}
