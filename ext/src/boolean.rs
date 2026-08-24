use serde_java::__private::Lazy;
use serde_java::{Class, Field, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter};
use std::{fmt, io};

/// Reference of java.lang.Boolean
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

impl<'a> Layout<'a> for Boolean {
    type Input = bool;
    type Output = Boolean;

    fn layout(input: &'a Self::Input) -> Self::Output {
        Boolean(*input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_java::JavaSerialize;

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

    #[derive(Debug, JavaSerialize)]
    #[java(
        class = "com.example.BooleanDemo",
        serial_version_uid = 3043629198327648182
    )]
    struct BooleanDemo {
        #[java(signature = "Ljava/lang/Boolean;", with = "crate::Boolean")]
        enabled: Option<bool>,
    }

    #[test]
    fn test_java_lang_boolean_field() -> io::Result<()> {
        init();

        // check enabled=null
        {
            let v = BooleanDemo { enabled: None };

            info!("{:?}", &v);

            let raw = v.to_bytes()?;

            assert_eq!(
                "aced000573720017636f6d2e6578616d706c652e426f6f6c65616e44656d6f2a3d24a14a538fb60200014c0007656e61626c65647400134c6a6176612f6c616e672f426f6f6c65616e3b787070",
                hex::encode(&raw)
            );
        }

        // check enabled=true
        {
            let v = BooleanDemo {
                enabled: Some(true),
            };

            info!("{:?}", &v);

            let raw = v.to_bytes()?;

            assert_eq!(
                "aced000573720017636f6d2e6578616d706c652e426f6f6c65616e44656d6f2a3d24a14a538fb60200014c0007656e61626c65647400134c6a6176612f6c616e672f426f6f6c65616e3b7870737200116a6176612e6c616e672e426f6f6c65616ecd207280d59cfaee0200015a000576616c7565787001",
                hex::encode(&raw)
            );
        }

        Ok(())
    }
}
