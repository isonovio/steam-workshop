mod runtime;

pub use runtime::is_running;
pub(crate) use runtime::{InitializationError, Runtime};
