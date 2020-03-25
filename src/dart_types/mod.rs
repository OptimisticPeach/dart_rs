//!
//! Wrapper types for Dart types. These provide similar
//! functionality to the equivalent types in Dart, while
//! trying to be idiomatic in Rust.
//!

use crate::dart_handle::{DartHandle, UnverifiedDartHandle};
use std::thread::LocalKey;

pub mod boolean;
pub mod d_string;
pub mod double;
pub mod dynamic;
pub mod integer;
pub mod list;

///
/// Trait which describes types of objects in terms of
/// `Dart` type variables.
///
pub trait DartType: DartHandle {
    ///
    /// The dart `Type` instance which describes what
    /// this instance is.
    ///
    /// See [`Double`](double::Double)'s implementation
    /// for how to implement this.
    ///
    const THIS: &'static LocalKey<UnverifiedDartHandle>;
}
