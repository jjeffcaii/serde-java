use serde_java::__private::Lazy;
use serde_java::{
    Class, Extends, ExtendsLayout, Field, JavaObject, JavaSerializable, JavaWriteable, ObjectWriter,
};
use std::{fmt, io};

static CLASS_OF_NUMBER: Lazy<Class> =
    Lazy::new(|| Class::builder("java.lang.Number", -8742448824652078965).build());

#[derive(Default)]
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
    ($name:ident, $inner:ident, $class:ident, $prim:ty, $java_class:expr, $suid:expr, $field:ident) => {
        static $class: Lazy<Class> = Lazy::new(|| {
            Class::builder($java_class, $suid)
                .super_class(Clone::clone(&CLASS_OF_NUMBER))
                .field(Field::builder("value").$field())
                .build()
        });

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
    byte
);

define_number!(
    Short,
    ShortInner,
    CLASS_OF_SHORT,
    i16,
    "java.lang.Short",
    7515723908773894738,
    short
);

define_number!(
    Integer,
    IntegerInner,
    CLASS_OF_INTEGER,
    i32,
    "java.lang.Integer",
    1360826667806852920,
    int
);

define_number!(
    Long,
    LongInner,
    CLASS_OF_LONG,
    i64,
    "java.lang.Long",
    4290774380558885855,
    long
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

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_byte() -> io::Result<()> {
        init();

        let b = Byte::from(0x01);

        info!("{}: {}", Byte::class().name(), b);

        let raw = b.to_bytes()?;

        assert_eq!(
            "aced00057372000e6a6176612e6c616e672e427974659c4e6084ee50f51c02000142000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b020000787001",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[test]
    fn test_java_lang_short() -> io::Result<()> {
        init();

        let s = Short::from(0x0102);

        info!("{}: {}", Short::class().name(), s);

        let raw = s.to_bytes()?;

        assert_eq!(
            "aced00057372000f6a6176612e6c616e672e53686f7274684d37133460da5202000153000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b02000078700102",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[test]
    fn test_java_lang_integer() -> io::Result<()> {
        init();

        let i = Integer::from(0x01020304);

        info!("{}: {}", Integer::class().name(), i);

        let b = i.to_bytes()?;

        assert_eq!(
            "aced0005737200116a6176612e6c616e672e496e746567657212e2a0a4f781873802000149000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b020000787001020304",
            hex::encode(&b)
        );

        Ok(())
    }

    #[test]
    fn test_java_lang_long() -> io::Result<()> {
        init();

        let l = Long::from(0x0102030405060708);

        info!("{}: {}", Long::class().name(), l);

        let b = l.to_bytes()?;

        assert_eq!(
            "aced00057372000e6a6176612e6c616e672e4c6f6e673b8be490cc8f23df0200014a000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b02000078700102030405060708",
            hex::encode(&b)
        );

        Ok(())
    }

    #[test]
    // 3.14 is the value the JVM fixture below was captured with, not an attempt at PI.
    #[allow(clippy::approx_constant)]
    fn test_java_lang_float() -> io::Result<()> {
        init();

        let f = Float::from(3.14);

        info!("{}: {}", Float::class().name(), f);

        let b = f.to_bytes()?;

        assert_eq!(
            "aced00057372000f6a6176612e6c616e672e466c6f6174daedc9a2db3cf0ec02000146000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b02000078704048f5c3",
            hex::encode(&b)
        );

        Ok(())
    }

    #[test]
    fn test_java_lang_double() -> io::Result<()> {
        init();

        let f = Double::from(std::f64::consts::PI);

        info!("{}: {}", Double::class().name(), f);

        let b = f.to_bytes()?;

        assert_eq!(
            "aced0005737200106a6176612e6c616e672e446f75626c6580b3c24a296bfb0402000144000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b0200007870400921fb54442d18",
            hex::encode(&b)
        );

        Ok(())
    }
}
