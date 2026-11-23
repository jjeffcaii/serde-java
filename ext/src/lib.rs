//! Pre-built `JavaObject`/`JavaSerializable` implementations for common JDK types.

#[cfg(test)]
#[macro_use]
extern crate log;

mod jdk;

#[cfg(test)]
pub(crate) mod testkit;

#[doc(hidden)]
pub mod __private {
    pub use crate::jdk::*;
}

pub mod java;
