//! Special macroquad-friendly collections.
//! The datatypes from this module may help to organize game code.

/// Global persistent storage. Nice for some global game configs available everywhere.
/// Yes, singletones available right here, with a nice API and some safety. 
pub mod storage;

/// A vector, but coroutines-friendly and with multiple borrows made (relatively) safe.
pub mod magic_vec;
