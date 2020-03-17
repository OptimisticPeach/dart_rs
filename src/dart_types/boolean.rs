use crate::dart_handle::{UnverifiedDartHandle, DartHandle};
use std::cell::Cell;
use crate::dart_unwrap;
use std::ops::Deref;
use crate::dart_types::DartType;
use std::thread::LocalKey;

#[derive(Clone, Debug)]
pub struct Boolean {
    handle: UnverifiedDartHandle,
    value: Cell<Option<bool>>,
}

impl Boolean {
    pub fn new(value: bool) -> Self {
        let handle = UnverifiedDartHandle::new_bool(value);
        Self {
            handle,
            value: Cell::new(Some(value))
        }
    }

    #[inline]
    pub fn value(&self) -> bool {
        if let Some(x) = self.value.get() {
            x
        } else {
            let value = dart_unwrap!(self.handle.get_bool());
            self.value.set(Some(value));
            value
        }
    }
}

impl std::ops::Not for Boolean {
    type Output = bool;
    fn not(self) -> bool {
        !self.value()
    }
}

impl From<bool> for Boolean {
    fn from(x: bool) -> Self {
        Self::new(x)
    }
}

thread_local! {
    #[allow(non_upper_case_globals)]
    pub static BoolType: UnverifiedDartHandle = {
        let b = UnverifiedDartHandle::const_false();
        b.get_instance_type().unwrap()
    };
}

impl DartType for Boolean {
    const THIS: &'static LocalKey<UnverifiedDartHandle> = &BoolType;
}

impl Deref for Boolean {
    type Target = UnverifiedDartHandle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl DartHandle for Boolean {
    fn handle(&self) -> dart_sys::Dart_Handle {
        self.handle.handle()
    }
    fn safe_handle(&self) -> UnverifiedDartHandle {
        self.handle
    }
    fn from_handle(handle: UnverifiedDartHandle) -> Result<Self, UnverifiedDartHandle> {
        if handle.is_boolean() {
            Ok(Self {
                handle,
                value: Cell::new(None)
            })
        } else {
            Err(handle)
        }
    }
}
