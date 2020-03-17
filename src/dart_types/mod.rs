use crate::dart_handle::{UnverifiedDartHandle, DartHandle};
use std::thread::LocalKey;

pub mod integer;
pub mod double;
pub mod d_string;
pub mod list;
pub mod boolean;

pub trait DartType: DartHandle {
    const THIS: &'static LocalKey<UnverifiedDartHandle>;
}
