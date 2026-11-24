use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, ClassFlags, DeserializeWith, JavaDeserializable, JavaFieldsDeserializable, JavaObject,
    JavaSerializable, JavaWriteable, Layout, ObjectReader, ObjectWriter, Reader,
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

impl From<Date> for u64 {
    fn from(val: Date) -> Self {
        val.0
    }
}

impl From<Date> for SystemTime {
    fn from(val: Date) -> Self {
        let unix_millis = time::Duration::from_millis(val.0);
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

impl JavaFieldsDeserializable for Date {
    type Fields = ();

    fn read_fields<R>(_r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read,
    {
        Ok(())
    }
}

impl JavaDeserializable for Date {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read,
    {
        let _fields = Self::default_read_object(r)?;

        let unix_millis: u64 = r.read()?;
        Ok(Self(unix_millis))
    }
}

impl<'a> Layout<'a> for Date {
    type Input = u64;
    type Output = Date;

    fn layout(input: &'a Self::Input) -> Self::Output {
        Date(*input)
    }
}

impl DeserializeWith for Date {
    type Output = u64;

    fn deserialize_with(&self) -> Self::Output {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;
    use serde_java::{JavaReadableExt, JavaSerialize, JavaWriteableExt};
    use serde_json::json;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_date() -> anyhow::Result<()> {
        init();

        let unix_millis = 1787649130999;

        let origin = Date::from(unix_millis); // 2026-08-25T17:12:10.999+0800

        info!("date: {:?}", origin);

        let actual = origin.to_bytes()?;

        let expect = testkit::serialize(Date::class().name(), &[], &unix_millis.to_string())?;

        assert_eq!(expect, actual);

        let parsed = Date::from_bytes(&actual)?;
        assert_eq!(origin, parsed);

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
            with = "crate::java::util::Date"
        )]
        created_at: Option<u64>,
    }

    #[test]
    fn test_date_in_fields() -> anyhow::Result<()> {
        init();

        let input = 1787649130999;
        let v = DateDemo {
            created_at: Some(input),
        };

        let expect = {
            let json = json!({"createdAt": &input}).to_string();
            testkit::serialize(DateDemo::class().name(), &["testdata/DateDemo.java"], &json)?
        };

        let actual = v.to_bytes()?;

        assert_eq!(expect, actual);

        Ok(())
    }
}
