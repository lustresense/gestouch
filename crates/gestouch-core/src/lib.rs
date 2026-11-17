//! Platform-independent configuration, recognizer and diagnostics types.
//! Windows and unsafe FFI deliberately live outside this crate.
pub mod config;
pub mod gesture;
pub mod health;

pub use config::{Action, Config, GestureRule, Trigger};
pub use gesture::{Contact, Direction, Frame, GestureKind, Recognized, Recognizer};
