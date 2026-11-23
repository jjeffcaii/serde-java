use serde_java::__private::once_cell::sync::Lazy;
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

impl From<Boolean> for bool {
    fn from(val: Boolean) -> Self {
        val.0
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
    use crate::testkit;
    use serde::Serialize;
    use serde_java::{JavaSerialize, JavaWriteableExt};

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_boolean() -> anyhow::Result<()> {
        init();

        // true
        {
            let b = Boolean::from(true);

            info!("{}: {}", Boolean::class().name(), b);

            let expect = testkit::serialize("java.lang.Boolean", &[], "true")?;

            let actual = b.to_bytes()?;
            assert_eq!(expect, actual);
        }

        // false
        {
            let b = Boolean::from(false);
            info!("{}: {}", Boolean::class().name(), b);

            let expect = testkit::serialize("java.lang.Boolean", &[], "false")?;
            let actual = b.to_bytes()?;
            assert_eq!(expect, actual);
        }

        Ok(())
    }

    #[derive(Debug, Serialize, JavaSerialize)]
    #[java(
        class = "com.example.BooleanDemo",
        serial_version_uid = 3043629198327648182
    )]
    struct BooleanDemo {
        #[java(signature = "Ljava/lang/Boolean;", with = "crate::java::lang::Boolean")]
        enabled: Option<bool>,
    }

    #[test]
    fn test_java_lang_boolean_field() -> anyhow::Result<()> {
        init();

        for next in [
            // check enabled=null
            BooleanDemo { enabled: None },
            // check enabled=true
            BooleanDemo {
                enabled: Some(true),
            },
            // check enabled=false
            BooleanDemo {
                enabled: Some(false),
            },
        ] {
            info!("{:?}", next);

            let expect = {
                let json = serde_json::to_string(&next)?;
                testkit::serialize(
                    BooleanDemo::class().name(),
                    &["testdata/BooleanDemo.java"],
                    &json,
                )?
            };

            let actual = next.to_bytes()?;
            assert_eq!(expect, actual);
        }

        Ok(())
    }
}
