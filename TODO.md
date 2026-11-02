# steam-workshop

## Review checkpoints

Each numbered step is a separate reviewable change. Finish and report its
validation before moving to the next step. No automatic commits. Preserve the
existing Cargo/mise scaffold. Run formatting and relevant compiler/lint checks
for code changes; add focused tests where they verify meaningful failure behavior.

## Backlog

- [ ] 12. Deferred (user accepts untested Windows/Linux). Platform verification: add Windows and Linux build checks, and record native-library/architecture requirements. Perform live Steam smoke tests where those systems are available; do not label build-only platforms runtime-verified.

## Scope

A game-independent Rust CLI for Steam Workshop targeting Windows, macOS, and Linux.
Verify macOS ARM64 first; distinguish intended support from tested support.
Use mise for toolchains. Work in small reviewable steps; do not commit automatically.

Commands:

- `steam-workshop probe-steam` (implemented)
- `steam-workshop inspect APP_ID ITEM_ID` (implemented)
- `steam-workshop update-file APP_ID ITEM_ID FILE` (implemented; live publication and TTS loading verified)
- `steam-workshop download APP_ID ITEM_ID OUTPUT` (implemented)

Keep Steam session initialization and callback handling, Steam Cloud operations,
and Workshop operations in focused internal modules. Expose only the commands
needed now. Do not build a plugin framework or general-purpose Steam SDK.

TTS save handling, JSON-to-BSON conversion, and module-to-Workshop-ID mappings
belong in the module repository, outside this executable. Publish opaque bytes.
The CLI uses the already logged-in Steam client and requires Valve's native library.

## Target architecture

This refactor supersedes the earlier module-only layering. Existing evidence below
records earlier implementations, not the final architecture. Finish the following
review checkpoints before the private-item publication acceptance test.

Dependency direction:

```text
CLI / context (composition)
           ↓
workshop (layer 3: application workflows)
           ↓
steam (layer 2: resource contracts and execution)
           ↓
native (layer 1: mechanical SDK wrappers)
           ↓
steamworks / steamworks::sys
```

Utilities are independent leaves. No inner module imports an outer type, including
through crate-root re-exports or impl blocks. Retain the current CLI and one
steam-client crate; do not add crates merely to represent layers.

Layer 1: native runtime/interface ownership and mechanical wrappers only.
RemoteStorage produces named File and PublishedFile wrappers. File exposes exists,
get_size, read, write and delete. PublishedFile exposes get_details and update;
its Update exposes creation, set_file and commit. Shared-content download/read
primitives live here too. Read receives a buffer and returns the native read result;
allocation, create-if-absent policy, cleanup policy and waiting belong above.
Async calls return typed native ApiCall<Result> values. Callback retrieval/release
and memory-safe decoding into native-layer result types also live here. No timeout
loops, application models, higher-level Session or executor references. Private
unsafe construction may encode native preconditions; the outer safe boundary must
enforce them. Never expose safe methods whose memory-safety preconditions are unmet.

Layer 2: SDK lifetime, session exclusivity, semantic Steam resource types and async
execution. Separate shared access to the initialized runtime from exclusive access
to the callback dispatcher. Pass an executor to operations that need execution;
resource objects must not retain an exclusive Session just to stay valid. Pending
operations may retain the executor borrow for their actual execution lifetime.
Use split fields/reborrows, not self-referential owners or general interior mutability.

Types express provenance; borrowing expresses lifetime; arguments supply execution
resources. Introduce a single scoped invariant storage brand if independently
constructed storage scopes must be distinguished. File and PublishedFile share that
brand and can coexist and be passed through ordinary functions/structs inside the
scope. Do not add a redundant session brand: one live SDK session plus runtime
borrows already prevents mixing successive sessions. A storage scope creates one
brand; subsequent borrows must not mint new identities. Compile-pass/fail tests
must establish the actual guarantees rather than assuming matching lifetime names
prove provenance. Native named File is not the same contract as an owned CloudFile.

Layer 2's PublishedFile::update_with consumes the owned file and takes an executor.
It encapsulates create-update, set-file, commit and result validation. Its pending
state retains the file until completion. Distinguish not-submitted, confirmed
completion/failure, and uncertain outcome. Return cleanup ownership only when safe;
uncertain outcomes retain remote content and expose recovery information. Dropping
or timing out must not silently delete a file that the remote operation may use.
Do not retry uncertain publication automatically.

Layer 3: Workshop preflight, temporary naming, inspect/download/update composition,
and application-facing results. Convert inner result models here. No CString,
native handles, callback descriptors, update builders or primitive decoding here.
Use Workshop::new(...) at this layer; do not make the inner Session depend on
Workshop through a convenience constructor. CLI owns local filesystem I/O and
presentation; context owns process re-execution and composes session/executor scope.

Target module responsibilities (create submodules only as each checkpoint needs them):

- native/: runtime, async/{api_call,dispatch,callback}, remote_storage/{file,published_file/update,ugc}, and native result types.
- steam/: session, executor, pending operations, storage scope, owned files, published files and semantic results.
- workshop/: application workflows, staging policy and models.
- context/: env and process boundary, composing the layers.
- utils/: independent buffer, entropy and representation helpers.
- tests/: mirrored production paths, including compile-time contract fixtures.

All lib.rs/mod.rs files contain declarations and exports only. CLI unsafe-code
prohibition remains in Cargo.toml. Preserve user edits and command behavior; do not
commit automatically. Publication stays outside the refactor validation.

## Implemented layout and verification limits

- CLI constructs Workshop explicitly inside steam_client::run and owns local file
  I/O and presentation. Context owns process configuration and re-execution.
- steam/session owns SDK lifetime and singleton access; access splits shared
  interface acquisition from exclusive executor dispatch. Storage scopes brand
  related resources, whose semantic wrappers embed native wrappers directly.
- steam/remote_storage owns complete read/update operations. Private pending updates
  retain their item, file and executor until completion. Public outcomes carry
  cleanup ownership only for unsubmitted or confirmed operations; uncertainty
  retains content and reports its filename.
- native owns SDK wrappers, async callback leases, descriptors and decoding. Utilities
  are independent leaves; Timeout preserves one operation budget across all layers.
- workshop owns preflight, temporary naming, cleanup policy and application models.
  It has no native handles, callback descriptors, pending types or update builders.
- Tests mirror production paths; compile-contract fixtures verify provenance,
  ownership, scoped validity and hidden pending execution. Module roots contain
  declarations and exports only; the CLI forbids unsafe code in Cargo.toml.

Current execution is synchronous and serial. Timeout stops waiting, not the remote
operation; uncertain publication is not retried. Tests of publication completion,
rejection and pending retention use simulated results or zero-timeout rejection;
the Rust CLI's actual publication round trip and TTS loading are now verified. Cloud tests
verify local Steam API behavior, not independent remote synchronization. Cloud
creation is not atomic against other processes; staging uses random filenames.
macOS ARM64 is runtime-verified under Cargo and from the published mise installation.
Windows/Linux verification is deferred by user agreement. The TTS wrapper remains checkpoint 13.

## Historical checkpoint evidence

R2 introduced features/RemoteStorage and CloudFile. RemoteStorage checks the native
pointer with NonNull and retains a Session borrow; CloudFile owns that borrowed
interface plus a nonempty, NUL-free CString filename. File methods own native
exists/write/read/delete calls, buffer/size validation and errors. Staging uses short
wrapper borrows before/after the operation and contains no raw FFI. Its cleanup
outcomes and timeout retention are unchanged. write_new rejects existing files
but is not atomic against other processes; random staging names avoid practical
collisions. The live isolated staging test passed readback, collision preservation,
invalid-name rejection and success/failure cleanup. Nineteen automated tests,
formatting and Clippy passed. Workshop items were not changed.

Temporary Cloud staging is implemented in staging.rs through
Session::with_temporary_cloud_file. It generates a random filename, refuses a
collision, checks FileWrite and deletes staging after successful/failed operations.
Cleanup errors retain the filename and original operation error. Timeouts retain
staging and report its name without retrying. FileWrite failure skips the operation
and attempts cleanup if a file exists. The safe wrapper's stream writer does not
report close errors, so this helper uses checked raw FileWrite inside the client.
A live read-only-to-Workshop test wrote/read/deleted two isolated Cloud files for
app 286160, covering operation success and failure. Cleanup failures and timeout
retention passed simulated tests. Nineteen automated tests, formatting and Clippy
passed. CLI publishing remains unavailable until the commit checkpoint; no Workshop
item was modified. This verifies staging through the local Steam API, not independent
remote Cloud synchronization.

Update preflight follows the refactored commands/<name>.rs layout and call_client
helper. It reads regular files with a bounded read, rejects payloads exceeding
Steam's single-FileWrite limit, and checks the item's consumer app and owner
against the initialized session. A successful preflight explicitly returns
Publishing is not implemented yet (exit 1). No Cloud writes or publishing APIs
are present in this path. Live preflight passed for the known BSON and private
item; missing files, directories and an invalid item were rejected. Metadata
before and after was unchanged. Wrong-app/owner and oversize checks passed in
focused tests. All 16 automated tests, formatting and Clippy passed.

Download resolves the current content handle, waits for UGCDownload, validates the
returned handle/size and reads the complete payload with UGCRead Close. The content
handle remains crate-private. The CLI stages output in a temporary file beside the
destination and persists without overwriting; failures remove temporary output.
Live test item 3826709471 downloaded 60091 bytes, identical to the known BSON:
SHA-256 3cddec1bf0c071f53e16c99deae12f2b2934469043eb1ee2bf24da8e9259ae09.
Repeated download preserved existing output; invalid item returned FileNotFound
and left no output or staging file. Paths containing spaces passed. Replaced an
existing timestamp-based test-directory helper with tempfile after a parallel-test
name collision. Fourteen automated tests and workspace Clippy passed.

Inspect is wired through `steam_client::run` and `Session::inspect`, with a 30-second
wait and typed WorkshopItem metadata. PendingCall is now crate-private. CLI output
formats strings with escaping and labels timestamps as Unix seconds. Native result
decoding checks size, status, text termination/UTF-8, visibility and nonnegative file
size without constructing native enum values from the byte buffer.
Live CLI lookup of item 3826709471 returned title test, app 286160, Private visibility
and 60091 bytes. Invalid item u64::MAX returned FileNotFound and exit 1 with no partial
metadata. Four decoder tests plus the eight existing automated tests passed;
formatting and workspace Clippy passed. No Workshop content was changed.

The `steam_client::run` process boundary is implemented. Its parent branch
re-executes the current command with the requested app variables and inherits
standard streams. Its child branch validates configuration and creates the session
before invoking the command closure. It returns an integer exit code, or a typed
error for setup, launch/wait, abnormal termination, initialization or command failure.
`Session::connect` is internal; its existing tests now live in src/tests/session.
Four isolated subprocess tests passed for single execution, output/status forwarding,
arguments containing spaces, cwd preservation, unchanged parent environment and
rejection of inconsistent child configuration without recursion. The process tests
use an internal fixture with no Steam initialization. A separate explicitly run
live test now verifies `run` using app 286160 for success and command failure,
and app u32::MAX for initialization failure. It verifies one closure execution
(or none on initialization failure), propagated output/status, SDK shutdown before
return and session-lock release by reconnecting. Probe-steam also passed with all
app/child variables unset. The inspect command is now wired to `run`.

Inspection of the installed TTS gameplay assembly found metadata queries through
SteamUGC but content loading through RemoteStorage.UGCDownload/UGCRead, followed
by direct save decoding. No modern DownloadItem/GetItemInstallInfo folder-loading
path was found in that assembly; its uploader also uses the legacy API. Continue
with the verified legacy content flow. Modern folder uploads are not verified for
TTS and are not required for the current implementation plan.

Callback waiting is implemented by `PendingCall`, which exclusively borrows the
session and consumes itself when waited on. Raw handles and decoders remain
internal. Manual dispatch checks callback IDs and buffer sizes, frees each fetched
callback on all exit paths, and reports transport, result, Steam and timeout errors.
A timeout does not cancel or retry the remote operation. This is the sole callback
consumer; do not mix it with `Client::run_callbacks` or concurrent pending calls.
Four focused tests cover completion, failure and deadlines. An explicitly run,
read-only native test fetched private item 3826709471 successfully through the
legacy API. Transport failure and timeout are simulated, not induced against Steam.
The public lookup now uses this waiter internally.

Session lifetime uses `steamworks` 0.13.1 internally. `Session::connect` verifies
the requested app and logged-in user; a private mutex guard prevents overlapping
sessions, and the private Client drops before the guard to shut Steam down.
Use `Client::init`, not `init_app`: the latter mutates the process environment.
Existing direct session tests supply app variables explicitly. The `run` boundary
now encapsulates that setup for command callers; session construction is internal. Initialization writes
no app-ID files and does not change cwd.
Live lifecycle tests passed for app 286160: user access, duplicate-session rejection,
shutdown/reopen, app mismatch and recovery. Missing app configuration exercises
initialization failure and lock release. Steam-stopped and logged-out cases have
not been tested live. Tests are ignored by default because they require a specific
Steam environment; both were explicitly run separately during this checkpoint.

`steamworks-sys` 0.13.0 supplies the raw legacy RemoteStorage APIs, manual callback
APIs, and matching native redistributables. It is a transitive dependency, accessed through `steamworks::sys` with the
`raw-bindings` feature. Its bundled macOS library contains
both ARM64 and x86_64. The ARM64 `probe-steam` command built and ran through Cargo,
reporting Steam running. This checks library loading and a status query only;
Publishing and downloading remain subsequent checkpoints.
Cargo supplies the development library search path. Standalone packaging remains
step 11; Windows/Linux runtime support remains unverified.

The temporary KP-Steam macOS port built with .NET 10 and Steamworks.NET
2025.163.0, using the ARM64 Steam library bundled with TTS. It successfully
updated private Workshop item 3826709471 for app 286160. Downloaded BSON matched
the uploaded bytes. The owner confirmed the changed top-level Note loaded in TTS.
TTS overwrites SaveName with the Workshop listing title when caching a module,
so SaveName is not a suitable content-update marker.

Useful local references (temporary paths are not durable dependencies):

- Original KP-Steam source: `/tmp/kpsteam-review`
- Tested macOS port and read-only probes: `/tmp/kpsteam-macos`
- Matching Steamworks.NET source: `/tmp/steamworks-review`
- Test JSON, BSON, and verification report: `/Users/wangyinong/Files/workspace/assets/tabtop-assets/workshop-test/`

Initial acceptance test: inspect the private test item without modifying it.
Uploads require an explicit test step; avoid touching production Workshop items.

Ownership checkpoint: CloudFile::new consumes RemoteStorage and returns a non-cloneable CloudFile
only after successful creation, rejects collisions, and handles partial-write
cleanup internally. CloudFile::delete consumes ownership and returns the file on
failure; retain explicitly relinquishes the local owner. TemporaryCloudFile owns
the staging lifecycle and reborrows its session for operations. CloudOperation
separates completed results from uncertain outcomes; cleanup no longer infers
remote completion from an error variant. No lookup API was added.

Validation: 20 automatic tests passed, formatting and strict Clippy passed.
Both isolated live Cloud tests passed: payload round trip, collision protection,
simulated partial-write cleanup, deletion ownership recovery, and staging cleanup
after success/failure. Uncertain-outcome retention and cleanup failures are tested
without inducing remote failures. No Workshop item was published.

R4 completed: callback dispatch now lives in features/callbacks.rs. RemoteStorage
implements published_file_details and download_content in focused feature modules;
these own native calls, decoding, callback waits and result validation. Session's
inspect/download methods compose those interfaces without raw SDK calls, pending
the Workshop facade checkpoint. Mirrored decoder/callback tests moved with them.
Validation: 20 automated tests, strict Clippy and formatting passed. The live
callback test and CLI inspection passed; a read-only download matched the known
60,091-byte BSON exactly (SHA-256 3cddec1bf0c071f53e16c99deae12f2b2934469043eb1ee2bf24da8e9259ae09).
No Workshop content changed.

R5 completed: Session::workshop returns a Workshop borrowing the session.
Inspection, download, update preflight, staging, and models now live under workshop/;
Session retains initialization, identity and feature accessors. CLI command modules
use the facade. Native upload limits are exported from the feature layer, so
Workshop orchestration has no SDK dependency. Tests mirror the moved modules.
Validation: 20 automated tests, formatting and strict Clippy passed. Live checks
passed for inspection, byte-identical download, overwrite protection, invalid-item
output cleanup, valid and rejected update preflight, and temporary staging cleanup.
No publication was attempted.

R6 completed: context/{env,run}.rs owns child-process configuration and dispatch;
its environment types are scoped to context. Root exports preserve run/RunError.
Subprocess tests live under tests/context/run and use the updated exact fixture
names. CloudFile now uses the feature layer's shared upload-limit constant.
The audit found no obsolete Session workflow forwarders; context, Workshop and
utilities have no raw SDK calls. Native model decoding remains under features;
session.rs owns SDK lifetime, identity, pipe access and app-independent probing.
All test-module paths mirror their production paths. No dependencies were added.
Validation: 20 automated tests, formatting and strict Clippy passed. Live process
boundary tests verified success, command failure, initialization failure and SDK
shutdown; session lifetime tests verified exclusivity and reconnect. Probe-steam
passed with app/child variables unset. Next checkpoint: 9, upload implementation;
its live publication acceptance remains a separate checkpoint.

Step 9 completed: Workshop::update_file composes preflight, temporary Cloud file
creation, PublishedUpdate creation/file selection/consuming commit, and callback
completion. The CLI now invokes publication with a five-minute shared operation
budget and reports the legal-agreement requirement when returned by Steam.
PublishedUpdate owns its native update handle and storage/session lifetime.
Native decoding distinguishes confirmed Steam results from unreadable responses.
Confirmed success/failure permits staging cleanup; timeout, transport failure,
malformed response or wrong item ID retains staging and reports the filename.
No retries are performed. Setup rejection before submission allows cleanup.

Validation: 24 automated tests, formatting and strict Clippy passed. Focused tests
cover legal status, native failure decoding and uncertain-outcome classification,
with existing staging tests covering cleanup and retention. Live isolated staging
cleanup passed; CLI rejected missing input and a nonexistent item before staging
or commit. No Workshop publication was executed. Step 10 remains required to verify
the actual legacy commit and TTS content round trip.
Reference: https://partner.steamgames.com/doc/api/ISteamRemoteStorage#CommitPublishedFileUpdate

L1 completed: details, visibility, download and commit result definitions now belong
to features/results.rs. Native decoding impls remain beside their native layouts;
they do not reference Workshop models. Workshop inspection converts inner details
and visibility into its public models. Download/commit results remain internal and
retain their existing behavior. Session/executor dependencies are unchanged in this
checkpoint and remain work for subsequent steps.
Validation: 26 automatic tests passed, including malformed native results, Steam
failures, legal status, Unicode and outer conversion of all fields/visibility
variants. Formatting, strict Clippy and the model-dependency audit passed. No live
publication or other Steam mutations were performed. Next checkpoint: L2.

L2 completed: native/runtime.rs owns the SDK Client and native identity, pipe and
status access. Session retains initialization policy, app/login validation and the
singleton mutex guard; runtime shutdown precedes guard release. Runtime is explicitly
thread-confined so removing the Session borrow does not permit cross-thread dispatch.
RemoteStorage and PendingCall now borrow Runtime, and feature modules no longer
reference Session. Staging callbacks receive reborrowed storage instead of accessing
the outer session through CloudFile. This replaces with_session; splitting lifetime
access from exclusive execution still belongs to L5. Native runtime has no imports
from outer modules; Session has no direct steamworks calls.
Validation: 26 automated tests, formatting and strict Clippy passed. Six explicitly
run live tests passed across process initialization/failure, session exclusivity and
reconnect, callback dispatch, owned Cloud files and staging cleanup. App-independent
probe passed with app/child environment variables unset. No Workshop publication.
Next checkpoint: L3.

L3 completed: native/remote_storage now contains mechanical RemoteStorage, File,
PublishedFile, PublishedFile::Update, and Ugc wrappers. File reads accept a
caller-provided buffer and return the native count; size conversions prevent slice
length truncation at the FFI boundary. Native wrappers do not allocate buffers,
wait for completion, apply collision policy, or decide cleanup. Higher feature
callers retain those responsibilities. All storage SDK invocations moved into the
native wrappers; callback dispatch remains for L4.
Derived native wrappers carry the runtime lifetime. The internal unsafe acquisition
constructor requires SDK validity for that lifetime; the safe feature owner enforces
it through its retained runtime borrow. This adapter remains until lifetime access
and execution are separated in L5. Async entry points still return native handles
for now; typed ApiCall descriptors are the next checkpoint.
Validation: 26 automatic tests, formatting and strict Clippy passed. Three isolated
live Cloud/staging tests passed, as did read-only inspection and byte-identical
60,091-byte download. No Workshop publication was executed. Next checkpoint: L4.

L4 completed: native async entry points now return ApiCall<ResultType>, binding
handles to native callback IDs, layouts and decoders inside the native layer.
Native Dispatch/Callback wrappers own callback retrieval and release; callback
leases prevent another retrieval through the same dispatcher until dropped.
A single nonblocking typed poll handles matching/retrieval; polling intervals and
timeouts remain in the higher waiter. Native ApiError is distinct from waiting
and allocation errors. Native result models and decoding moved to native/results;
outer conversion maps native errors without creating inward dependencies.
Validation: 28 automatic tests, formatting and strict Clippy passed, including
handle/descriptor matching, retrieval failures, malformed results and deadlines.
Live typed waiting and a native callback-release error-path test passed, followed
by inspection and byte-identical 60,091-byte download. Native dependency and mirrored
test-path audits passed. No Workshop publication. Next checkpoint: L5.

L5 completed: Session::execution splits shared storage lifetime access from the
session's exclusive dispatch token. The new steam::Executor owns native dispatch
access; async operations receive it explicitly and pending calls retain its mutable
borrow until completion. Storage and CloudFile no longer retain mutable Runtime
borrows. Native storage acquisition now safely borrows Runtime. Staging passes
storage and executor directly, removing CloudFile::with_storage and reborrow helpers.
The existing feature adapters remain for L6–L8 migration.
Validation: 28 automatic tests and three compile-contract checks passed, proving
resource coexistence/helper use, rejection of overlapping executors, and rejection
of resources outliving Session. Formatting, strict Clippy and diff checks passed.
Seven live tests passed across session/process lifecycle, callback completion and
release, owned Cloud files and staging cleanup. Read-only download matched the
known 60,091-byte BSON exactly. No Workshop publication. Next checkpoint: L6.

Execution restructuring completed: Session::access returns shared Access and an
independent Executor without acquiring RemoteStorage. Access owns interface access;
Executor owns dispatch orchestration and deadlines; PendingCall holds the exclusive
execution borrow and delegates waiting. Native async/{api_call,dispatch,callback}
separates typed completion recognition/decoding, manual dispatch, and callback leases.
Session, access, executor, pending call and execution errors now live under steam/;
mirrored tests moved with them. Existing feature adapters and Session::workshop
remain scheduled for L6–L8 migration. Execution remains serial, without multiplexing.
Validation: 28 automatic tests and three compile-contract checks passed, along with
strict Clippy and formatting. Four live session/process/callback tests passed,
including callback release after malformed completion. Read-only download remained
byte-identical (60,091 bytes). No Workshop publication. Next checkpoint remains L6.

Accessor migration completed: removed Session::remote_storage and migrated the
remaining Cloud and staging tests to Access::remote_storage. All 28 automatic tests,
three compile-contract checks, three isolated live Cloud/staging tests, formatting
and strict Clippy passed. No Workshop publication.

L6 completed: Access::remote_storage now takes a scoped closure, minting one
invariant storage identity per invocation. RemoteStorage copies retain that identity;
CloudFile and PublishedFile carry it without owning execution access. The semantic
storage wrappers moved from features/ to steam/remote_storage/. PublishedFile owns
the item reference and metadata operation; PublishedUpdate now takes a PublishedFile
and accepts only a CloudFile with the same brand. Staging passes the typed file,
removing string-to-CString conversion from Workshop submission. Full consuming
update/pending-file ownership remains L8; read composition and removal of the
Session::workshop dependency remain L7.
Validation: 28 automatic tests and eight compile-contract checks passed. Contracts
cover same-source helper/struct composition, mixed-source rejection, CloudFile and
PublishedFile scope escape rejection, consuming deletion, and executor/session
lifetimes. Four live tests passed for Cloud creation/collision, failed deletion
ownership recovery and retry, rejected writes, staging cleanup, and metadata waiting.
Read-only download matched the known 60,091-byte BSON exactly. Formatting, strict
Clippy and diff checks passed. No Workshop publication. Next checkpoint: L7.

Native embedding refactor completed: native File owns its CString; CloudFile and
PublishedFile embed their corresponding native wrappers and carry the storage brand
separately. PublishedUpdate consumes the embedded native published-file wrapper and
preserves its brand. Resource operations no longer reconstruct native wrappers.
Validation: 28 automatic tests, eight compile-contract checks, four live Cloud/metadata/
staging tests, formatting, strict Clippy and diff checks passed. No publication.

L8 completed ahead of L7 at the user's request: PublishedFile::update_with consumes
CloudFile, owns native update creation/selection/commit, and returns either a
PendingUpdate or UpdateRejected with cleanup ownership. PendingUpdate retains the
file and exclusive executor borrow; confirmed completion returns the file, while
uncertain completion returns its retained filename and error. Dropping pending
updates retains remote content. Workshop now owns only temporary naming, preflight,
and cleanup/reporting policy. Removed TemporaryCloudFile, CloudOperation, the generic
staging callback, and the exposed semantic update builder.
Native Ugc construction validates handle sentinels; native ReadAction translates the
read flag. Layer 2 no longer references sys symbols.
Validation: 26 automatic tests and 11 compile-contract checks passed. Four isolated
live tests cover cleanup diagnostics, simulated rejected/confirmed/uncertain updates,
and unpolled pending-drop/zero-timeout retention. No native publication was invoked;
submission errors and completion results are injected in test-only code. Read-only
UGC download matched the known 60,091-byte BSON. Strict Clippy, formatting, diff and
raw-symbol/obsolete-abstraction audits passed. Next checkpoint remains L7, then L9.

Update boundary correction completed: PendingUpdate is private to layer 2 and owns
PublishedFile as well as CloudFile. Native PublishedFile::update borrows its wrapper.
The public update_with(file, executor, timeout) performs submission, waiting and
validation in one call, returning NotSubmitted, Complete or Uncertain. Workshop
only applies cleanup/reporting policy. No public pending or submission-error types
remain. Zero timeout is rejected before any native update submission.
Validation: 26 automatic tests, 12 compile-contract checks, five isolated live tests,
strict Clippy, formatting and diff checks passed. Live checks used simulated results
or zero-timeout rejection; no Workshop publication. L7 remains next.

L7 completed: PublishedFile::download now resolves metadata and downloads its UGC
content entirely within layer 2 under one timeout budget. A private semantic Ugc
embeds the native wrapper, validates completion and reads the bytes. Workshop
converts metadata at its boundary and no longer stores or handles content handles.
Removed Session::workshop; CLI commands construct Workshop::new(session) explicitly.
The Steam layer no longer imports Workshop or references sys symbols.
Validation: 26 automatic tests, 12 compile-contract checks, formatting, strict Clippy
and diff checks passed. Live CLI inspection and byte-identical 60,091-byte download
passed, including a path with spaces, overwrite protection, and invalid-item cleanup
with no leftover output/staging file. No Workshop publication. Next checkpoint: L9.

Timeout refactor completed: utils::Timeout owns the original start and duration,
with check(self) -> Option<Self> and remaining(). CLI operation entry points construct
one budget; Workshop, semantic operations and callback waiting pass it unchanged.
Removed nested timer creation and remaining-duration arithmetic. Read submission
checks expiration before starting native calls; update expiration still preserves
not-submitted versus uncertain ownership semantics. Only polling sleep reads the
remaining duration.
Validation: 28 automatic tests, 12 compile-contract checks, five isolated live
ownership/metadata tests and a byte-identical 60,091-byte download passed. Tests
verify expired/zero budgets and preservation of start/duration through copies and
checks. Formatting, strict Clippy and boilerplate/diff audits passed. No publication.
Next checkpoint remains L9.

L9 completed: audited all production imports, impls and re-exports for inward-only
dependencies, checked mirrored tests/module roots and CLI unsafe prohibition, and
verified that obsolete feature adapters and public pending/submission interfaces
are gone. Existing code passed without requiring further production changes.
Validation: 28 automated tests, 12 compile-contract checks, strict Clippy, formatting
and diff checks passed. Eleven live tests passed for process/session lifecycle,
callback completion/release, owned Cloud files, simulated publication ownership,
timeout/drop retention and cleanup diagnostics. CLI probe, inspection, byte-identical
60,091-byte download, overwrite protection, invalid-item cleanup, missing-input and
invalid-item update preflight rejection passed. No native Workshop publication was
invoked. The refactor checkpoints are complete; checkpoint 10 remains a separate
private-item publication acceptance step.

Checkpoint 10 publication/download passed: the Rust CLI updated private item
3826709471 (app 286160) with a BSON changing only its top-level Note to marker
`Rust CLI upload test - 2026-11-02 - edd24741`. Commit returned success and the
subsequent download matched all 60,089 uploaded bytes (SHA-256
927eae5e559a67439a0a440dac85c2e83580394fbab338ba89944338276cc8ab).
The item remains Private; updated timestamp is 1793596535. In-game TTS loading
was confirmed by the user. Original download/metadata, upload BSON/JSON, command logs,
round-trip download and verification.json are preserved under
`/Users/wangyinong/Files/workspace/assets/tabtop-assets/workshop-test/rust-cli-2026-11-02-edd24741/`.
No production item was changed; no commit was created.

Checkpoint 11 completed: published v0.1.0 from existing pushed commit
3e5f43cfb388bb39c02b15e21492997751868033, with a macOS ARM64 archive
containing the executable and sibling libsteam_api.dylib, plus SHA-256 checksum.
scripts/package-macos.py builds and packages the release. Installed the remote
GitHub release through mise and verified --version, probe-steam and inspection of
private item 3826709471 outside the repository. No global mise configuration change
or source commit was made. README documents the mise install command. Windows and
Linux remain untested and are not packaged in this release; the user accepts this.
Release: https://github.com/isonovio/steam-workshop/releases/tag/v0.1.0

Version bump task completed: mise run bump major|minor|patch opens $EDITOR for
Markdown notes, then updates the root Cargo.toml, its Cargo.lock entry, and
CHANGELOG.md. Verified all bump modes, offline locked Cargo metadata, editor
failure/empty-note cancellation, invalid input, and mise argument forwarding.
The working version remains 0.1.0; no commit or publication was performed.

Release automation implemented: .github/workflows/release.yaml checks main pushes
and manual runs, skips published Cargo versions, validates/builds macOS ARM64,
and uses the version's CHANGELOG.md section as release notes. scripts/publish.py
resumes matching drafts, uploads archive/checksum before publishing, and rejects
version downgrades and conflicting tags/drafts. Published releases stay unchanged.
Validation: actionlint, Ruff F/E9/PLW1510 checks, Rust formatting, strict Clippy,
28 automated tests and 12 doctests passed; publication/recovery/skip/conflict paths
were checked with simulated GitHub responses. The exact mise use @latest command
passed in a temporary directory and stored latest in mise.toml. No new release
was published. Workflow activation requires committing and pushing to main.
