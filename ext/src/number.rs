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

static CLASS_OF_SHORT: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Short", 7515723908773894738)
        .super_class(Clone::clone(&CLASS_OF_NUMBER))
        .field(Field::builder("value").short())
        .build()
});

static CLASS_OF_INTEGER: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Integer", 1360826667806852920)
        .super_class(Clone::clone(&CLASS_OF_NUMBER))
        .field(Field::builder("value").int())
        .build()
});

static CLASS_OF_LONG: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Long", 4290774380558885855)
        .super_class(Clone::clone(&CLASS_OF_NUMBER))
        .field(Field::builder("value").long())
        .build()
});

pub struct Short(ExtendsLayout<ShortInner, Number>);

impl JavaObject for Short {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_SHORT)
    }
}

impl JavaWriteable for Short {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

impl fmt::Display for Short {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &self.0.this().0;
        fmt::Display::fmt(&this, f)
    }
}

impl fmt::Debug for Short {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &self.0.this().0;
        fmt::Debug::fmt(this, f)
    }
}

impl Into<i16> for Short {
    fn into(self) -> i16 {
        let (inner, _) = self.0.into();
        inner.0
    }
}

impl From<i16> for Short {
    fn from(value: i16) -> Self {
        Self(ShortInner(value).extends(Number::default()))
    }
}

struct ShortInner(i16);

impl JavaObject for ShortInner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_SHORT)
    }
}

impl JavaSerializable for ShortInner {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

pub struct Integer(ExtendsLayout<IntegerInner, Number>);

impl JavaObject for Integer {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_INTEGER)
    }
}

impl JavaWriteable for Integer {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &self.0.this().0;
        fmt::Display::fmt(&this, f)
    }
}

impl fmt::Debug for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &self.0.this().0;
        fmt::Debug::fmt(this, f)
    }
}

impl Into<i32> for Integer {
    fn into(self) -> i32 {
        let (inner, _) = self.0.into();
        inner.0
    }
}

impl From<i32> for Integer {
    fn from(value: i32) -> Self {
        Self(IntegerInner(value).extends(Number::default()))
    }
}

struct IntegerInner(i32);

impl JavaObject for IntegerInner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_INTEGER)
    }
}

impl JavaSerializable for IntegerInner {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

pub struct Long(ExtendsLayout<LongInner, Number>);

impl JavaObject for Long {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_LONG)
    }
}

impl JavaWriteable for Long {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

impl fmt::Display for Long {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &self.0.this().0;
        fmt::Display::fmt(&this, f)
    }
}

impl fmt::Debug for Long {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = &self.0.this().0;
        fmt::Debug::fmt(this, f)
    }
}

impl Into<i64> for Long {
    fn into(self) -> i64 {
        let (inner, _) = self.0.into();
        inner.0
    }
}

impl From<i64> for Long {
    fn from(value: i64) -> Self {
        Self(LongInner(value).extends(Number::default()))
    }
}

struct LongInner(i64);

impl JavaObject for LongInner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_LONG)
    }
}

impl JavaSerializable for LongInner {
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
    fn test_java_lang_short() -> io::Result<()> {
        init();

        let s = Short::from(0x0102);

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

        let b = i.to_bytes()?;

        assert_eq!(
            "aced0005737200116a6176612e6c616e672e496e746567657212e2a0a4f781873802000149000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b020000787001020304",
            hex::encode(&b)
        );

        Ok(())
    }

    #[test]
    fn test_java_long_long() -> io::Result<()> {
        init();

        let l = Long::from(0x0102030405060708);

        let b = l.to_bytes()?;

        assert_eq!(
            "aced00057372000e6a6176612e6c616e672e4c6f6e673b8be490cc8f23df0200014a000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b02000078700102030405060708",
            hex::encode(&b)
        );

        Ok(())
    }
}
