use std::num::NonZeroU32;

use steam_client::{Session, SessionError};

#[test]
#[ignore = "requires Steam running with SteamAppId and SteamGameId set to 286160"]
fn session_lifetime() {
    let app_id = NonZeroU32::new(286160).unwrap();
    let session = Session::connect(app_id).unwrap();
    assert_ne!(session.user_id(), 0);
    assert!(matches!(
        Session::connect(app_id),
        Err(SessionError::Unavailable)
    ));
    drop(session);

    let session = Session::connect(app_id).unwrap();
    assert_ne!(session.user_id(), 0);
    drop(session);

    let wrong_app = NonZeroU32::new(1).unwrap();
    assert!(matches!(
        Session::connect(wrong_app),
        Err(SessionError::AppMismatch { .. })
    ));
    assert!(Session::connect(app_id).is_ok());
}

#[test]
#[ignore = "requires an environment where Steam initialization is unavailable"]
fn initialization_failure() {
    let app_id = NonZeroU32::new(286160).unwrap();
    assert!(matches!(
        Session::connect(app_id),
        Err(SessionError::Initialization(_))
    ));
    assert!(matches!(
        Session::connect(app_id),
        Err(SessionError::Initialization(_))
    ));
}
