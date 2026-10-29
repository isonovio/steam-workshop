use std::num::NonZeroU32;

use super::*;

#[test]
fn validates_names() {
    assert_eq!(filename(""), None);
    assert_eq!(filename("bad\0name"), None);
    assert_eq!(
        filename("valid.bson").unwrap().to_str().unwrap(),
        "valid.bson"
    );
}

#[test]
#[ignore = "writes and deletes isolated test Cloud files; requires SteamAppId/SteamGameId 286160"]
fn live_owned_file() {
    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let name = format!(
        "steam-workshop-{:032x}.tmp",
        crate::utils::random_u128().unwrap()
    );
    let payload = b"owned Cloud file test";
    let mut file = session
        .remote_storage()
        .unwrap()
        .write_new(&name, payload)
        .unwrap();
    assert_eq!(file.read().unwrap(), payload);
    file.with_session(|session, name| {
        assert!(matches!(
            session
                .remote_storage()
                .unwrap()
                .write_new(name, b"collision"),
            Err(CreateFileError::AlreadyExists)
        ));
    });
    assert_eq!(file.read().unwrap(), payload);
    assert!(file.delete().is_ok());

    let file = session
        .remote_storage()
        .unwrap()
        .write_new(&name, payload)
        .unwrap();
    assert_eq!(file.retain(), name);
    // Simulate a failed native write that left a partial file.
    assert!(matches!(
        session
            .remote_storage()
            .unwrap()
            .finish_write(filename(&name).unwrap(), false),
        Err(CreateFileError::Write)
    ));
    let mut file = session
        .remote_storage()
        .unwrap()
        .write_new(&name, payload)
        .unwrap();
    file.with_session(|session, name| {
        let storage = session.remote_storage().unwrap();
        let name = filename(name).unwrap();
        assert!(unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileDelete(storage.pointer.as_ptr(), name.as_ptr())
        });
    });
    let file = match file.delete() {
        Ok(()) => panic!("deleting missing file unexpectedly succeeded"),
        Err(file) => file,
    };
    assert_eq!(file.retain(), name);
}
