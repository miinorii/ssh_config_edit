//! This crate provides high and low level abstractions for
//! reading and editing ssh_config files non-destructively.
//!
//! [`SSHConfig`] `parse` -> `to_string()` is byte-identical,
//! editing key K only changes the bytes of K's line.
//!
//! The semantic layer is accessible through a lossy high-level abstraction with
//! [`SSHConfig`], [`HostSettings`], [`Field`] and [`FieldKey`].
//!
//! ```rust
//! use ssh_config_edit::{FieldKey, SSHConfig};
//!
//! # fn main() -> Result<(), ssh_config_edit::Error> {
//! let mut config = SSHConfig::parse("Host dev\n\tUser me\n")?;
//!
//! let mut settings = config.raw_host_settings("dev").expect("dev is declared");
//! settings.replace(FieldKey::Port, "2222")?;
//! config.set_host_settings(&settings)?;
//!
//! assert_eq!(config.to_string(), "Host dev\n\tUser me\n\tPort 2222\n");
//! # Ok(())
//! # }
//! ```
//!
//! The document layer is accessible through a lossless low-level abstraction with
//! [`SSHConfig`], [`Section`] and [`Line`].
//!
//! ```rust
//! use ssh_config_edit::{FieldKey, Line, SSHConfig, Section, Selector, LineEnding, Indent};
//!
//! # fn main() -> Result<(), ssh_config_edit::Error> {
//! let mut config = SSHConfig::parse("# managed\nHost dev\n    Port=22\n")?;
//!
//! let section = config.section_mut("dev").expect("dev is declared");
//! if let Some(port) = section.get_one_mut(&FieldKey::Port) {
//!     port.set_value("2222")?;
//! }
//! section.push(Line::comment("tuned")?);
//!
//! // the '=' separator, the 4 space indent and the comment all survive
//! assert_eq!(
//!     config.to_string(),
//!     "# managed\nHost dev\n    Port=2222\n    # tuned\n"
//! );
//!
//! // build a whole new section and append it.
//! // keys created this way get their canonical spelling, `FieldKey::Hostname`
//! // writes "Hostname". Parsed keys keep whatever the file had.
//! let mut prod = Section::new(Selector::new(FieldKey::Host, "prod")?)
//!     .with_indent(Indent::new("    ")?)
//!     .with_ending(LineEnding::Lf);
//! prod.push(Line::directive(FieldKey::Hostname, "10.0.0.1")?);
//! prod.push(Line::comment("gateway")?);
//! prod.push(Line::blank());
//! config.push_section(prod);
//!
//! assert_eq!(
//!     config.to_string(),
//!     "# managed\nHost dev\n    Port=2222\n    # tuned\n\
//!      Host prod\n    Hostname 10.0.0.1\n    # gateway\n\n"
//! );
//! # Ok(())
//! # }
//! ```

#![deny(rustdoc::broken_intra_doc_links)]

mod decor;
mod error;
mod field_keys;
mod lexer;
mod line;
mod section;
mod settings;
mod ssh_config;

pub use decor::{Indent, LineEnding};
pub use error::{Error, ParseErrorKind, Result};
pub use field_keys::{FieldKey, SelectorKind};
pub use line::{Directive, Line, LineKind, Selector};
pub use section::Section;
pub use settings::{Field, HostSettings};
pub use ssh_config::SSHConfig;
