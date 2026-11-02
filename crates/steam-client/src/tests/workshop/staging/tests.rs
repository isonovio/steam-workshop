use std::num::NonZeroU32;

use crate::Session;

use super::*;

#[test]
fn reports_retained_filename() {
    assert!(
        matches!(finish(UpdateOutcome::Uncertain { filename: "retained".into(), error: CallError::Timeout }),
        Err(CloudError::Uncertain { filename, source: CallError::Timeout }) if filename == "retained")
    );
}

#[test]
#[ignore = "writes and deletes isolated Cloud files; requires SteamAppId/SteamGameId 286160"]
fn live_cleanup_outcomes() {
    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let (steam, _) = session.access();
    steam
        .remote_storage(|storage| {
            for failed_operation in [false, true] {
                let name = temporary_filename().unwrap();
                let file = CloudFile::new(storage, &name, b"cleanup").unwrap();
                let result = match failed_operation {
                    false => Ok(true),
                    true => Err(CallError::Transport),
                };
                let outcome = finish(UpdateOutcome::Complete { file, result });
                match failed_operation {
                    false => assert_eq!(outcome, Ok(true)),
                    true => assert!(matches!(
                        outcome,
                        Err(CloudError::Call(CallError::Transport))
                    )),
                }
                let file = CloudFile::new(storage, &name, b"deleted").unwrap();
                assert!(file.delete().is_ok());
            }
            for failed_operation in [false, true] {
                let name = temporary_filename().unwrap();
                let file = CloudFile::new(storage, &name, b"cleanup failure").unwrap();
                assert!(file.simulate_external_deletion());
                let result = match failed_operation {
                    false => Ok(false),
                    true => Err(CallError::Transport),
                };
                assert!(matches!(cleanup(file, result), Err(CloudError::Cleanup { filename, operation })
                    if filename == name && operation == failed_operation.then_some(CallError::Transport)));
            }
        })
        .unwrap();
}
