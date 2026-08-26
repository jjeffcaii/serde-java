use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, ClassFlags, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter,
};
use std::io;
use std::ops::Add;
use std::time::{self, SystemTime, SystemTimeError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(/* unix_millis */ u64);

impl Date {
    pub fn now() -> Result<Self, SystemTimeError> {
        TryFrom::try_from(SystemTime::now())
    }
}

impl Into<u64> for Date {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<SystemTime> for Date {
    fn into(self) -> SystemTime {
        let unix_millis = time::Duration::from_millis(self.0);
        SystemTime::UNIX_EPOCH.add(unix_millis)
    }
}

impl From<u64> for Date {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<SystemTime> for Date {
    type Error = SystemTimeError;

    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        let unix_millis = {
            let du = value.duration_since(SystemTime::UNIX_EPOCH)?;
            du.as_millis() as u64
        };

        Ok(Self(unix_millis))
    }
}

impl JavaObject for Date {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("java.util.Date", 7523967970034938905)
                .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
                .build()
        });

        Clone::clone(&*CLASS)
    }
}

impl JavaSerializable for Date {
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)?;

        self.0.write_to(w)?;

        Ok(())
    }
}

impl<'a> Layout<'a> for Date {
    type Input = u64;
    type Output = Date;

    fn layout(input: &'a Self::Input) -> Self::Output {
        Date(*input)
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
    fn test_date() -> io::Result<()> {
        init();

        let value = Date::from(1787649130999); // 2026-08-25T17:12:10.999+0800

        info!("date: {:?}", &value);

        let raw = value.to_bytes()?;

        assert_eq!(
            "aced00057372000e6a6176612e7574696c2e44617465686a81014b59741903000078707708000001a0383101f778",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(
        class = "com.example.DateDemo",
        serial_version_uid = 3088539499338478546
    )]
    struct DateDemo {
        #[java(
            rename = "createdAt",
            signature = "Ljava/util/Date;",
            with = "crate::Date"
        )]
        created_at: Option<u64>,
    }

    #[test]
    fn test_date_in_fields() -> io::Result<()> {
        init();

        let v = DateDemo {
            created_at: Some(1787649130999),
        };

        let raw = v.to_bytes()?;

        assert_eq!(
            "aced000573720014636f6d2e6578616d706c652e4461746544656d6f2adcb24f94d533d20200014c00096372656174656441747400104c6a6176612f7574696c2f446174653b78707372000e6a6176612e7574696c2e44617465686a81014b59741903000078707708000001a0383101f778",
            hex::encode(&raw)
        );

        Ok(())
    }
}
