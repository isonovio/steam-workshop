use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

const FIXTURE: &str = "run::tests::process_fixture";
const SPACED_ARG: &str = "a path with spaces/module.bson";

#[test]
#[ignore = "subprocess fixture, invoked by the process-boundary tests"]
fn process_fixture() {
    println!("process-entry");
    let before = (
        env::var_os(CHILD_APP_ID),
        env::var_os("SteamAppId"),
        env::var_os("SteamGameId"),
    );
    let result = dispatch(NonZeroU32::new(286160).unwrap(), || {
        println!("command-output");
        eprintln!("command-diagnostic");
        assert_eq!(
            env::current_dir().unwrap(),
            std::path::PathBuf::from(env::var_os("TEST_CWD").unwrap())
        );
        assert!(env::args_os().any(|arg| arg == SPACED_ARG));
        match env::var("TEST_CASE").unwrap().as_str() {
            "failure" => Err(RunError::Command(io::Error::other("fixture failure"))),
            _ => Ok(()),
        }
    });
    assert_eq!(
        before,
        (
            env::var_os(CHILD_APP_ID),
            env::var_os("SteamAppId"),
            env::var_os("SteamGameId"),
        )
    );
    let code = match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            23
        }
    };
    std::process::exit(code);
}

fn fixture(case: &str, marker: Option<&str>) -> std::process::Output {
    fixture_for(FIXTURE, case, marker)
}

fn fixture_for(test: &str, case: &str, marker: Option<&str>) -> std::process::Output {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        env::temp_dir().join(format!("steam-client test {} {unique}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let mut command = Command::new(env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            test,
            "--nocapture",
            "--skip",
            SPACED_ARG,
        ])
        .current_dir(&directory)
        .env("TEST_CWD", &directory)
        .env("TEST_CASE", case)
        .env("SteamAppId", "parent-app")
        .env("SteamGameId", "parent-game")
        .env_remove(CHILD_APP_ID);
    if let Some(marker) = marker {
        command.env(CHILD_APP_ID, marker);
    }
    let output = command.output().unwrap();
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
    fs::remove_dir(directory).unwrap();
    output
}

#[test]
fn relaunch_preserves_output_arguments_directory_and_parent_environment() {
    let output = fixture("success", None);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("process-entry").count(), 2);
    assert_eq!(stdout.matches("command-output").count(), 1);
    assert_eq!(
        String::from_utf8(output.stderr)
            .unwrap()
            .matches("command-diagnostic")
            .count(),
        1
    );
}

#[test]
fn propagates_child_failure_without_repeating_command() {
    let output = fixture("failure", None);
    assert_eq!(output.status.code(), Some(23), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("process-entry").count(), 2);
    assert_eq!(stdout.matches("command-output").count(), 1);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("fixture failure")
    );
}

#[test]
fn rejects_invalid_marker_without_relaunching() {
    let output = fixture("success", Some("invalid"));
    assert_eq!(output.status.code(), Some(23));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("process-entry").count(), 1);
    assert!(!stdout.contains("command-output"));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Invalid Steam child process configuration")
    );
}

#[test]
fn rejects_mismatched_child_environment_without_relaunching() {
    let output = fixture("success", Some("286160"));
    assert_eq!(output.status.code(), Some(23));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("process-entry").count(), 1);
    assert!(!stdout.contains("command-output"));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Invalid Steam child process configuration")
    );
}

#[test]
#[ignore = "live subprocess fixture, invoked by live_session_boundary"]
fn session_fixture() {
    println!("session-process-entry");
    let case = env::var("TEST_CASE").unwrap();
    let app_id = NonZeroU32::new(match case.as_str() {
        "initialization-failure" => u32::MAX,
        _ => 286160,
    })
    .unwrap();
    let child = env::var_os(CHILD_APP_ID).is_some();
    let result = run(app_id, |session| {
        assert_ne!(session.user_id(), 0);
        assert!(matches!(
            Session::connect(app_id),
            Err(SessionError::Unavailable)
        ));
        println!("session-command-output");
        match case.as_str() {
            "command-failure" => Err(io::Error::other("session command failed")),
            _ => Ok(()),
        }
    });
    if child {
        match case.as_str() {
            "initialization-failure" => assert!(
                matches!(result, Err(RunError::Initialization(_))),
                "{result:?}"
            ),
            "command-failure" => assert!(matches!(result, Err(RunError::Command(_))), "{result:?}"),
            _ => assert!(matches!(result, Ok(0)), "{result:?}"),
        }
        // This query is valid without initialization; zero confirms SDK shutdown.
        assert_eq!(unsafe { steamworks::sys::SteamAPI_GetHSteamPipe() }, 0);
        if case != "initialization-failure" {
            let session = Session::connect(app_id).unwrap();
            assert_ne!(session.user_id(), 0);
            drop(session);
            assert_eq!(unsafe { steamworks::sys::SteamAPI_GetHSteamPipe() }, 0);
        }
        println!("session-cleanup-verified");
    }
    let code = match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            31
        }
    };
    std::process::exit(code);
}

#[test]
#[ignore = "requires Steam running and logged in with access to app 286160"]
fn live_session_boundary() {
    for (case, code, executions) in [
        ("success", 0, 1),
        ("command-failure", 31, 1),
        ("initialization-failure", 31, 0),
    ] {
        let output = fixture_for("run::tests::session_fixture", case, None);
        assert_eq!(output.status.code(), Some(code), "{case}: {output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout.matches("session-process-entry").count(),
            2,
            "{case}: {stdout}"
        );
        assert_eq!(
            stdout.matches("session-command-output").count(),
            executions,
            "{case}: {stdout}"
        );
        assert_eq!(
            stdout.matches("session-cleanup-verified").count(),
            1,
            "{case}: {stdout}"
        );
        let stderr = String::from_utf8(output.stderr).unwrap();
        match case {
            "command-failure" => assert!(stderr.contains("session command failed"), "{stderr}"),
            "initialization-failure" => {
                assert!(stderr.contains("Steam initialization failed"), "{stderr}")
            }
            _ => (),
        }
    }
}
