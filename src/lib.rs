//! This crate provides high and low level abstractions for
//! reading and editing ssh_config files non-destructively.
//!
//! [SSHConfig](crate::SSHConfig) `parse` -> `to_string()` is byte-identical,
//! editing key K only changes the bytes of K's line.
//!
//! The semantic layer is accessible throught a lossy high-level abstraction with
//! [SSHConfig](crate::SSHConfig), [HostSettings](crate::HostSettings),
//! [Field](crate::Field) and [FieldKey](crate::FieldKey).
//!
//! ```rust
//! use ssh_config_edit::SSHConfig;
//!
//! fn main() {
//!     todo!()
//! }
//!
//! ```
//!
//!
//! The document layer is accessible throught a lossless low-level abstraction with
//! [SSHConfig](crate::SSHConfig), [Section](crate::Section) and [Line](crate::Line).
//!
//! ```rust
//! use ssh_config_edit::SSHConfig;
//!
//! fn main() {
//!     todo!()
//! }
//!
//! ```

mod error;
pub mod field_keys;
mod lexer;
pub mod line;
pub mod section;
pub mod settings;
pub mod ssh_config;
