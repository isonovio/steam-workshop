use std::marker::PhantomData;
use std::mem::MaybeUninit;

use steamworks::sys;

use crate::native::Runtime;

use super::callback::Callback;

pub(crate) struct Dispatch<'runtime> {
    pipe: sys::HSteamPipe,
    _runtime: PhantomData<&'runtime Runtime>,
}

impl<'runtime> Dispatch<'runtime> {
    /// No other dispatcher may access this runtime until this dispatcher is dropped.
    pub(crate) unsafe fn new(runtime: &'runtime Runtime) -> Self {
        Self {
            pipe: runtime.pipe(),
            _runtime: PhantomData,
        }
    }

    pub(crate) fn run_frame(&mut self) {
        unsafe { sys::SteamAPI_ManualDispatch_RunFrame(self.pipe) }
    }

    pub(crate) fn next(&mut self) -> Option<Callback<'_>> {
        let mut message = MaybeUninit::uninit();
        let retrieved = unsafe {
            sys::SteamAPI_ManualDispatch_GetNextCallback(self.pipe, message.as_mut_ptr())
        };
        retrieved.then(|| unsafe { Callback::new(self.pipe, message.assume_init()) })
    }
}
