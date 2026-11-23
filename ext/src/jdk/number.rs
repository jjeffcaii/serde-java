use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, Extends, ExtendsLayout, Field, JavaObject, JavaSerializable, JavaWriteable, Layout,
    ObjectWriter,
};
use std::{fmt, io};

static CLASS_OF_NUMBER: Lazy<Class> =
    Lazy::new(|| Class::builder("java.lang.Number", -8742448824652078965).build());

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Number;

impl JavaObject for Number {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_NUMBER)
    }
}

impl JavaSerializable for Number {
    fn write_fields(&self, _w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

macro_rules! define_number {
    ($name:ident, $inner:ident, $class:ident, $prim:ty, $java_class:expr, $suid:expr, $field:ident $(, $extra:ident)*) => {
        static $class: Lazy<Class> = Lazy::new(|| {
            Class::builder($java_class, $suid)
                .super_class(Clone::clone(&CLASS_OF_NUMBER))
                .field(Field::builder("value").$field())
                .build()
        });

        #[derive(Copy, Clone, PartialEq $(, $extra)*)]
        pub struct $name(ExtendsLayout<$inner, Number>);

        impl JavaObject for $name {
            fn class() -> Class {
                Clone::clone(&$class)
            }
        }

        impl JavaWriteable for $name {
            fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
                self.0.write_to(w)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let this = &self.0.this().0;
                fmt::Display::fmt(this, f)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let this = &self.0.this().0;
                fmt::Debug::fmt(this, f)
            }
        }

        impl From<$name> for $prim {
            fn from(value: $name) -> Self {
                let (inner, _) = value.0.into();
                inner.0
            }
        }

        impl From<$prim> for $name {
            fn from(value: $prim) -> Self {
                Self($inner(value).extends(Number::default()))
            }
        }

        impl<'a> Layout<'a> for $name {
            type Input = $prim;
            type Output = $name;

            fn layout(input: &'a Self::Input) -> Self::Output {
                let extends = $inner(*input).extends(Number::default());
                Self(extends)
            }
        }

        #[derive(Copy, Clone, PartialEq $(, $extra)*)]
        struct $inner($prim);

        impl JavaObject for $inner {
            fn class() -> Class {
                Clone::clone(&$class)
            }
        }

        impl JavaSerializable for $inner {
            fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
                self.0.write_to(w)
            }
        }
    };
}

define_number!(
    Byte,
    ByteInner,
    CLASS_OF_BYTE,
    i8,
    "java.lang.Byte",
    -7183698231559129828,
    byte,
    Eq,
    Hash
);

define_number!(
    Short,
    ShortInner,
    CLASS_OF_SHORT,
    i16,
    "java.lang.Short",
    7515723908773894738,
    short,
    Eq,
    Hash
);

define_number!(
    Integer,
    IntegerInner,
    CLASS_OF_INTEGER,
    i32,
    "java.lang.Integer",
    1360826667806852920,
    int,
    Eq,
    Hash
);

define_number!(
    Long,
    LongInner,
    CLASS_OF_LONG,
    i64,
    "java.lang.Long",
    4290774380558885855,
    long,
    Eq,
    Hash
);

define_number!(
    Float,
    FloatInner,
    CLASS_OF_FLOAT,
    f32,
    "java.lang.Float",
    -2671257302660747028,
    float
);

define_number!(
    Double,
    DoubleInner,
    CLASS_OF_DOUBLE,
    f64,
    "java.lang.Double",
    -9172774392245257468,
    double
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testkit;

    use serde_java::JavaWriteableExt;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_byte() -> anyhow::Result<()> {
        init();

        let origin = 0x01i8;
        let b = Byte::from(origin);

        info!("{}: {}", Byte::class().name(), b);

        let expect = {
            let json = serde_json::to_string(&origin)?;
            testkit::serialize(Byte::class().name(), &[], &json)?
        };

        let actual = b.to_bytes()?;
        assert_eq!(expect, actual);

        Ok(())
    }

    #[test]
    fn test_java_lang_short() -> anyhow::Result<()> {
        init();

        let input = 0x0102i16;

        let s = Short::from(input);

        info!("{}: {}", Short::class().name(), s);

        let expect = {
            let json = serde_json::to_string(&input)?;
            testkit::serialize(Short::class().name(), &[], &json)?
        };

        let actual = s.to_bytes()?;
        assert_eq!(expect, actual);

        Ok(())
    }

    #[test]
    fn test_java_lang_integer() -> anyhow::Result<()> {
        init();

        let input = 0x01020304i32;

        let i = Integer::from(input);

        info!("{}: {}", Integer::class().name(), i);

        let expect = {
            let json = serde_json::to_string(&input)?;
            testkit::serialize(Integer::class().name(), &[], &json)?
        };

        let actual = i.to_bytes()?;
        assert_eq!(expect, actual);

        Ok(())
    }

    #[test]
    fn test_java_lang_long() -> anyhow::Result<()> {
        init();

        let input = 0x0102030405060708i64;
        let l = Long::from(input);

        info!("{}: {}", Long::class().name(), l);

        let expect = {
            let json = serde_json::to_string(&input)?;
            testkit::serialize(Long::class().name(), &[], &json)?
        };

        let actual = l.to_bytes()?;
        assert_eq!(expect, actual);

        Ok(())
    }

    #[test]
    // 3.14 is the value the JVM fixture below was captured with, not an attempt at PI.
    #[allow(clippy::approx_constant)]
    fn test_java_lang_float() -> anyhow::Result<()> {
        init();

        let input = 3.14f32;
        let f = Float::from(input);

        info!("{}: {}", Float::class().name(), f);

        let expect = {
            let json = serde_json::to_string(&input)?;
            testkit::serialize(Float::class().name(), &[], &json)?
        };

        let actual = f.to_bytes()?;
        assert_eq!(expect, actual);

        Ok(())
    }

    #[test]
    fn test_java_lang_double() -> anyhow::Result<()> {
        init();

        let input = std::f64::consts::PI;
        let f = Double::from(input);

        info!("{}: {}", Double::class().name(), f);

        let expect = {
            let json = serde_json::to_string(&input)?;
            testkit::serialize(Double::class().name(), &[], &json)?
        };

        let actual = f.to_bytes()?;

        assert_eq!(expect, actual);

        Ok(())
    }
}
