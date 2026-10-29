use std::num::NonZeroU32;

use crate::Session;

use super::*;

#[test]
fn validates_names() {
    assert_eq!(to_filename(""), None);
    assert_eq!(to_filename("bad\0name"), None);
    assert_eq!(
        to_filename("valid.bson").unwrap().to_str().unwrap(),
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
    let mut file = CloudFile::new(session.remote_storage().unwrap(), &name, payload).unwrap();
    assert_eq!(file.name(), name);
    assert_eq!(file.read().unwrap(), payload);
    file.with_session(|session, name| {
        assert!(matches!(
            CloudFile::new(session.remote_storage().unwrap(), name, b"collision"),
            Err(CreateFileError::AlreadyExists)
        ));
    });
    assert_eq!(file.read().unwrap(), payload);
    assert!(file.delete().is_ok());

    let file = CloudFile::new(session.remote_storage().unwrap(), &name, payload).unwrap();
    assert_eq!(file.retain(), name);
    let storage = session.remote_storage().unwrap();
    let native_name = to_filename(&name).unwrap();
    assert!(storage.exists(&native_name));
    assert!(storage.delete(&native_name));
    let mut file = CloudFile::new(session.remote_storage().unwrap(), &name, payload).unwrap();
    file.with_session(|session, name| {
        let storage = session.remote_storage().unwrap();
        let name = to_filename(name).unwrap();
        assert!(storage.delete(&name));
    });
    let file = match file.delete() {
        Ok(()) => panic!("deleting missing file unexpectedly succeeded"),
        Err(file) => file,
    };
    assert_eq!(file.retain(), name);
}

#[test]
#[ignore = "writes and deletes an isolated test Cloud file; requires SteamAppId/SteamGameId 286160"]
fn live_rejected_writes_and_invalid_names() {
    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    for name in ["", "bad\0name"] {
        assert!(matches!(
            CloudFile::new(session.remote_storage().unwrap(), name, b"invalid"),
            Err(CreateFileError::InvalidName)
        ));
    }
    let name = format!(
        "steam-workshop-{:032x}.tmp",
        crate::utils::random_u128().unwrap()
    );
    let native_name = to_filename(&name).unwrap();
    let storage = session.remote_storage().unwrap();
    assert!(!storage.exists(&native_name));
    assert_eq!(storage.read(&native_name), None);

    assert!(matches!(
        CloudFile::new(storage, &name, b""),
        Err(CreateFileError::Write)
    ));

    let storage = session.remote_storage().unwrap();
    assert!(!storage.exists(&native_name));
    assert_eq!(storage.read(&native_name), None);
}
