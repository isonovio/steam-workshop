use std::cell::Cell;
use std::num::NonZeroU32;

use super::*;

#[test]
fn cleans_up_after_success_and_failure() {
    let cleaned = Cell::new(0);
    let cleanup = || {
        cleaned.set(cleaned.get() + 1);
        true
    };
    assert_eq!(finish(Ok(42), "test".into(), cleanup).unwrap(), 42);
    assert!(matches!(
        finish::<()>(Err(CallError::Transport), "test".into(), cleanup),
        Err(CloudError::Call(CallError::Transport))
    ));
    assert_eq!(cleaned.get(), 2);
}

#[test]
fn preserves_cleanup_and_operation_failures() {
    assert!(matches!(
        finish(Ok(()), "test".into(), || false),
        Err(CloudError::Cleanup {
            operation: None,
            ..
        })
    ));
    assert!(matches!(
        finish::<()>(Err(CallError::Transport), "test".into(), || false),
        Err(CloudError::Cleanup {
            operation: Some(CallError::Transport),
            ..
        })
    ));
}

#[test]
fn timeout_retains_staging_filename_without_cleanup() {
    let result = finish::<()>(Err(CallError::Timeout), "retained".into(), || {
        panic!("deleted pending upload")
    });
    assert!(matches!(result, Err(CloudError::Uncertain { filename }) if filename == "retained"));
}

#[test]
#[ignore = "writes and deletes isolated test Cloud files; requires SteamAppId/SteamGameId 286160"]
fn live_temporary_cloud_files() {
    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let payload = b"steam-workshop temporary Cloud test";
    let mut names = Vec::new();
    for fail in [false, true] {
        let result = session.with_temporary_cloud_file(payload, |_, filename| {
            names.push(filename.to_owned());
            let name = CString::new(filename).unwrap();
            let mut downloaded = vec![0; payload.len()];
            // Read only the generated staging file while its owning session is borrowed.
            let read = unsafe {
                sys::SteamAPI_ISteamRemoteStorage_FileRead(
                    sys::SteamAPI_SteamRemoteStorage_v016(),
                    name.as_ptr(),
                    downloaded.as_mut_ptr().cast(),
                    downloaded.len() as i32,
                )
            };
            let verified = read == payload.len() as i32 && downloaded == payload;
            match verified && !fail {
                true => Ok(()),
                false => Err(CallError::InvalidResult),
            }
        });
        match fail {
            false => result.unwrap(),
            true => assert!(matches!(
                result,
                Err(CloudError::Call(CallError::InvalidResult))
            )),
        }
        let name = CString::new(names.last().unwrap().as_str()).unwrap();
        // The session is still alive; verify deletion of exactly the generated filename.
        assert!(!unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileExists(
                sys::SteamAPI_SteamRemoteStorage_v016(),
                name.as_ptr(),
            )
        });
    }
    assert_ne!(names[0], names[1]);
}
