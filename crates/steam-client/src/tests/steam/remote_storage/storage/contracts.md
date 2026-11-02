A scope creates one storage identity. Copies preserve it, and resources can be
passed through ordinary helpers and stored together.

```no_run
use std::num::NonZeroU64;
use steam_client::{Access, CloudFile, PublishedFile, RemoteStorage};
struct Upload<'runtime, 'storage> {
    item: PublishedFile<'runtime, 'storage>,
    file: CloudFile<'runtime, 'storage>,
}
fn prepare<'runtime, 'storage>(storage: RemoteStorage<'runtime, 'storage>) -> Upload<'runtime, 'storage> {
    let copied = storage;
    Upload {
        item: storage.published_file(NonZeroU64::new(42).unwrap()),
        file: CloudFile::new(copied, "temporary", b"payload").unwrap(),
    }
}
fn run(steam: Access<'_>) {
    steam.remote_storage(|storage| {
        let upload = prepare(storage);
        assert_eq!(upload.item.item_id().get(), 42);
        assert!(upload.file.delete().is_ok());
    }).unwrap();
}
```

Independent scopes cannot supply a file and item with the same identity.

```compile_fail
use std::num::NonZeroU64;
use steam_client::{Access, CloudFile, Executor};
fn mixed<'runtime>(steam: Access<'runtime>, executor: &mut Executor<'runtime>) {
    steam.remote_storage(|first| {
        let item = first.published_file(NonZeroU64::new(42).unwrap());
        steam.remote_storage(|second| {
            let file = CloudFile::new(second, "temporary", b"payload").unwrap();
            let _ = item.update_with(file, executor, std::time::Duration::from_secs(30));
        });
    });
}
```

An owned file cannot escape its storage scope.

```compile_fail
use steam_client::{Access, CloudFile};
fn escape(steam: Access<'_>) {
    let file = steam.remote_storage(|storage| {
        CloudFile::new(storage, "temporary", b"payload").unwrap()
    }).unwrap();
    let _ = file.read();
}
```

A published-file reference cannot escape either.

```compile_fail
use std::num::NonZeroU64;
use steam_client::Access;
fn escape(steam: Access<'_>) {
    let item = steam.remote_storage(|storage| {
        storage.published_file(NonZeroU64::new(42).unwrap())
    }).unwrap();
    let _ = item.item_id();
}
```

Successful deletion consumes ownership.

```compile_fail,E0382
use steam_client::CloudFile;
fn deleted(file: CloudFile<'_, '_>) {
    assert!(file.delete().is_ok());
    let _ = file.read();
}
```

Submission consumes the file; the caller cannot delete it while the update runs.

```compile_fail,E0382
use steam_client::{CloudFile, Executor, PublishedFile};
fn pending<'runtime, 'storage>(item: PublishedFile<'runtime, 'storage>, file: CloudFile<'runtime, 'storage>, executor: &mut Executor<'runtime>) {
    let pending = item.update_with(file, executor, std::time::Duration::from_secs(30));
    let _ = file.delete();
    drop(pending);
}
```

Execution access is released when the operation returns, so another update can follow.

```no_run
use std::time::Duration;
use steam_client::{CloudFile, Executor, PublishedFile};
fn sequential<'runtime, 'storage>(first: PublishedFile<'runtime, 'storage>, second: PublishedFile<'runtime, 'storage>, a: CloudFile<'runtime, 'storage>, b: CloudFile<'runtime, 'storage>, executor: &mut Executor<'runtime>) {
    let first_outcome = first.update_with(a, executor, Duration::from_secs(30));
    let second_outcome = second.update_with(b, executor, Duration::from_secs(30));
}
```

Pending execution is not part of the public API.

```compile_fail,E0432
use steam_client::PendingUpdate;
```

The caller receives an outcome with cleanup ownership only when safe.

```no_run
use std::time::Duration;
use steam_client::{CloudFile, Executor, PublishedFile, UpdateOutcome};
fn submit<'runtime, 'storage>(item: PublishedFile<'runtime, 'storage>, file: CloudFile<'runtime, 'storage>, executor: &mut Executor<'runtime>) {
    match item.update_with(file, executor, Duration::from_secs(30)) {
        UpdateOutcome::NotSubmitted { file, .. } | UpdateOutcome::Complete { file, .. } => { let _ = file.delete(); }
        UpdateOutcome::Uncertain { filename, error } => { eprintln!("Retained {filename}: {error}"); }
    }
}
```
