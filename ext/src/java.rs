pub mod lang {
    pub use crate::jdk::Boolean;
    pub use crate::jdk::Character;
    pub use crate::jdk::{Byte, Double, Float, Integer, Long, Short};
    pub use crate::jdk::{Error, Exception, RuntimeException, StackTraceElement, Throwable};
}

pub mod util {
    pub use crate::jdk::Date;
    pub use crate::jdk::{ArrayList, LinkedList};
    pub use crate::jdk::{HashMap, HashMapBuilder};
}
