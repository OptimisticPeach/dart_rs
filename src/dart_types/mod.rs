use crate::dart_handle::{DartHandle, UnverifiedDartHandle};
use std::thread::LocalKey;

pub mod boolean;
pub mod d_string;
pub mod double;
pub mod dynamic;
pub mod integer;
pub mod list;

pub trait DartType: DartHandle {
    const THIS: &'static LocalKey<UnverifiedDartHandle>;
}
