use super::object::Object;
use super::serializable::{JavaObject, JavaSerializable, JavaWriteable};
use super::writer::ObjectWriter;
use std::fmt::{Debug, Formatter};
use std::{fmt, io};

pub trait Extends<P: Sized>: Sized {
    fn extends(self, parent: P) -> ExtendsLayout<Self, P>;
}

impl<T, P> Extends<P> for T
where
    T: JavaSerializable + JavaObject,
    P: JavaSerializable + JavaObject,
{
    fn extends(self, parent: P) -> ExtendsLayout<Self, P> {
        ExtendsLayout::new(self, parent)
    }
}

pub struct ExtendsLayout<T, P> {
    this: T,
    parent: P,
}

impl<T, P> ExtendsLayout<T, P> {
    pub(crate) fn new(this: T, parent: P) -> Self {
        Self { this, parent }
    }

    pub fn this(&self) -> &T {
        &self.this
    }

    pub fn parent(&self) -> &P {
        &self.parent
    }
}

impl<T, P> Debug for ExtendsLayout<T, P>
where
    T: fmt::Debug,
    P: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtendsLayout")
            .field("this", &self.this)
            .field("parent", &self.parent)
            .finish()
    }
}

impl<T, P> Into<(T, P)> for ExtendsLayout<T, P> {
    fn into(self) -> (T, P) {
        (self.this, self.parent)
    }
}

impl<T, P> JavaWriteable for ExtendsLayout<T, P>
where
    T: JavaSerializable + JavaObject,
    P: JavaSerializable + JavaObject,
{
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let t_class = T::class();
        let obj = Object::<T, P>::builder(t_class)
            .this(&self.this)
            .extends(&self.parent)
            .build();

        obj.write_to(w)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Class, Field};
    use once_cell::sync::Lazy;
    use std::io::Write;

    struct PojoA {
        id: i32,
    }

    impl JavaObject for PojoA {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.PojoA", -4387630733789281459)
                    .field(Field::builder("id").int())
                    .super_class(PojoB::class())
                    .build()
            });

            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for PojoA {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn Write>) -> io::Result<()> {
            w.write_int(self.id)?;
            Ok(())
        }
    }

    struct PojoB {
        name: String,
    }

    impl JavaObject for PojoB {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.PojoB", 311662489269302022)
                    .field(Field::builder("name").string())
                    .build()
            });

            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for PojoB {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn Write>) -> io::Result<()> {
            self.name.write_to(w)?;
            Ok(())
        }
    }

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_serialize() -> io::Result<()> {
        init();

        let a = PojoA { id: 0xff };
        let b = PojoB {
            name: "helloworld".to_string(),
        };

        let c = a.extends(b);
        let raw = c.to_bytes()?;

        assert_eq!(
            "aced000573720011636f6d2e6578616d706c652e506f6a6f41c31c011822e7434d020001490002696478720011636f6d2e6578616d706c652e506f6a6f4204533f61fab5f3060200014c00046e616d657400124c6a6176612f6c616e672f537472696e673b787074000a68656c6c6f776f726c64000000ff",
            hex::encode(&raw),
        );

        Ok(())
    }
}
