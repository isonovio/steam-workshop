Resources can coexist while execution access is held, and pass through ordinary helpers.

```no_run
use steam_client::{CloudFile, Executor, RemoteStorage, Session};
fn resources(storage: RemoteStorage<'_, '_>, executor: &mut Executor<'_>) {
    let first = CloudFile::new(storage, "first", b"first").unwrap();
    let second = CloudFile::new(storage, "second", b"second").unwrap();
    assert!(first.read().is_some());
    assert!(second.read().is_some());
    let _ = executor;
    assert!(first.delete().is_ok());
    assert!(second.delete().is_ok());
}
fn run(session: &mut Session) {
    let (steam, mut executor) = session.access();
    steam.remote_storage(|storage| resources(storage, &mut executor)).unwrap();
}
```

A live executor prevents acquisition of another dispatcher.

```compile_fail,E0499
use steam_client::Session;
fn overlapping(session: &mut Session) {
    let (_, first) = session.access();
    let (_, second) = session.access();
    drop(first);
    drop(second);
}
```

Resources cannot outlive their session.

```compile_fail,E0505
use steam_client::Session;
fn escape(mut session: Session) {
    let (steam, executor) = session.access();
    drop(executor);
    drop(session);
    let _ = steam.remote_storage(|_| ()).unwrap();
}
```
