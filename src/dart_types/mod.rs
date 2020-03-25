//!
//! Wrapper types for Dart types. These provide similar
//! functionality to the equivalent types in Dart, while
//! trying to be idiomatic in Rust.
//!
//! # Note
//! Documentation will be lacking in these types since
//! they very closely match the dart semantics.
//!
//! If you would like me to document one of them more
//! closely, please open an issue on this crate's github
//! page.
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
