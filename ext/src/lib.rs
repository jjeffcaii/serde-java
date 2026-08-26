//! Pre-built `JavaObject`/`JavaSerializable` implementations for common JDK types.

#[cfg(test)]
#[macro_use]
extern crate log;

mod jdk;

#[doc(hidden)]
pub mod __private {
    pub use crate::jdk::*;
}

pub mod java;
