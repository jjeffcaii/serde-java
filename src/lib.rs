#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::from_over_into)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate smallvec;

mod error;
pub mod ext;
mod misc;
mod proto;
mod suid;

pub use error::Error;
pub use proto::*;

#[cfg(feature = "derive")]
pub use serde_java_derive::JavaSerialize;

/// Implementation details used by the generated code of `#[derive(JavaSerialize)]`.
/// Not a stable API.
#[doc(hidden)]
pub mod __private {
    pub use once_cell::sync::Lazy;
}

pub type Result<T> = std::result::Result<T, Error>;

/// cached string
#[doc(hidden)]
pub mod astr {
    include!(concat!(env!("OUT_DIR"), "/astr.rs"));
}
