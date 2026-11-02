use crate::Session;

pub struct Workshop<'session> {
    pub(super) session: &'session mut Session,
}

impl<'session> Workshop<'session> {
    pub fn new(session: &'session mut Session) -> Self {
        Self { session }
    }
}
