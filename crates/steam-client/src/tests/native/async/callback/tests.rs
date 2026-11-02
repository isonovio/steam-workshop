use crate::native::{Dispatch, Runtime};

use super::*;

#[test]
fn checks_native_retrieval_before_decoding() {
    assert_eq!(retrieval_status(true, false), Ok(()));
    assert_eq!(retrieval_status(false, false), Err(ApiError::InvalidResult));
    assert_eq!(retrieval_status(true, true), Err(ApiError::Transport));
    assert_eq!(retrieval_status(false, true), Err(ApiError::Transport));
}

#[test]
#[ignore = "run alone; read-only; requires Steam running with SteamAppId/SteamGameId 286160"]
fn live_release_after_invalid_callback() {
    use std::time::{Duration, Instant};

    let runtime = Runtime::new().unwrap();
    let storage = crate::native::remote_storage::RemoteStorage::new(&runtime).unwrap();
    let _first = storage.published_file(3826709471).get_details().unwrap();
    {
        let mut dispatch = unsafe { Dispatch::new(&runtime) };
        let start = Instant::now();
        loop {
            assert!(start.elapsed() < Duration::from_secs(30));
            dispatch.run_frame();
            if let Some(mut callback) = dispatch.next()
                && callback.completion().unwrap().is_some()
            {
                callback.message.m_cubParam = 0;
                assert!(matches!(
                    callback.completion(),
                    Err(ApiError::InvalidResult)
                ));
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    let mut dispatch = unsafe { Dispatch::new(&runtime) };
    let second = storage.published_file(3826709471).get_details().unwrap();
    let start = Instant::now();
    loop {
        assert!(start.elapsed() < Duration::from_secs(30));
        dispatch.run_frame();
        if let Some(callback) = dispatch.next()
            && let Some(details) = second.resolve(&callback).unwrap()
        {
            assert_eq!(details.id, 3826709471);
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
