use serde_java::__private::Lazy;
use serde_java::{Class, Field, JavaObject, JavaSerializable, JavaWriteable, ObjectWriter};
use std::{fmt, io};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Boolean(pub bool);

impl fmt::Debug for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Into<bool> for Boolean {
    fn into(self) -> bool {
        self.0
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl JavaObject for Boolean {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("java.lang.Boolean", -3665804199014368530)
                .field(Field::builder("value").boolean())
                .build()
        });
        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for Boolean {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_boolean() -> io::Result<()> {
        init();

        // true
        {
            let b = Boolean::from(true);

            info!("{}: {}", Boolean::class().name(), &b);

            let raw = b.to_bytes()?;
            assert_eq!(
                "aced0005737200116a6176612e6c616e672e426f6f6c65616ecd207280d59cfaee0200015a000576616c7565787001",
                hex::encode(&raw)
            );
        }

        // false
        {
            let b = Boolean::from(false);

            info!("{}: {}", Boolean::class().name(), &b);

            let raw = b.to_bytes()?;
            assert_eq!(
                "aced0005737200116a6176612e6c616e672e426f6f6c65616ecd207280d59cfaee0200015a000576616c7565787000",
                hex::encode(&raw)
            );
        }

        Ok(())
    }
}
