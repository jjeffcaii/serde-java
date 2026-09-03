use super::object::Object;
use super::serializable::{JavaObject, JavaSerializable, JavaWriteable};
use super::writer::ObjectWriter;
use crate::ClassFlags;
use std::fmt::{Debug, Formatter};
use std::{fmt, io};

pub trait Extends<P: Sized>: Sized {
    fn extends(self, parent: P) -> ExtendsLayout<Self, P>;
}

impl<T, P> Extends<P> for T {
    fn extends(self, parent: P) -> ExtendsLayout<Self, P> {
        ExtendsLayout::new(self, parent)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

    pub fn this_mut(&mut self) -> &mut T {
        &mut self.this
    }

    pub fn parent_mut(&mut self) -> &mut P {
        &mut self.parent
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
        let obj = Object::<T, P>::builder(T::class(), &self.this)
            .extends(P::class(), &self.parent)
            .build();

        obj.write_to(w)?;

        Ok(())
    }
}

/// Lets a `Xxx(ExtendsLayout<Inner, Parent>)` wrapper type delegate its own
/// `JavaSerializable` to this impl, so it can be nested (e.g. inside `Reference<T>`)
/// rather than only usable as a top-level `JavaWriteable`.
impl<T, P> JavaSerializable for ExtendsLayout<T, P>
where
    T: JavaSerializable + JavaObject,
    P: JavaSerializable + JavaObject,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.this.write_fields(w)
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.parent.write_object(w)?;
        if P::class().flags().contains(ClassFlags::WRITE_METHOD) {
            w.end()?;
        }
        self.this.write_object(w)
    }
}

// C extends B extends A
impl<C, B, A> JavaWriteable for ExtendsLayout<C, ExtendsLayout<B, A>>
where
    C: JavaSerializable + JavaObject,
    B: JavaSerializable + JavaObject,
    A: JavaSerializable + JavaObject,
{
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let c_class = C::class();
        let c = &self.this;

        let b_class = B::class();
        let b = &self.parent.this;

        let a_class = A::class();
        let a = &self.parent.parent;

        let old = w.set_block_data_mode(true);

        let h = w.begin_object(&c_class)?;

        // A
        {
            w.with_dyn(|w| a.write_object(w))?;
            if a_class.flags().contains(ClassFlags::WRITE_METHOD) {
                w.end()?;
            }
        }

        // B
        w.with_dyn(|w| b.write_object(w))?;
        if b_class.flags().contains(ClassFlags::WRITE_METHOD) {
            w.end()?;
        }

        // C
        w.with_dyn(|w| c.write_object(w))?;
        if c_class.flags().contains(ClassFlags::WRITE_METHOD) {
            w.end()?;
        }

        w.set_block_data_mode(old);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Class, Field, JavaWriteableExt, Writer};
    use once_cell::sync::Lazy;

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
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            w.write(self.id)?;
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
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
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

    struct A {
        a: String,
    }

    impl JavaObject for A {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.A", 5993405657316421481)
                    .field(Field::builder("a").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for A {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.a.write_to(w)?;
            Ok(())
        }
    }

    struct B {
        b: String,
    }

    impl JavaObject for B {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.B", 7601936770627721045)
                    .super_class(A::class())
                    .field(Field::builder("b").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for B {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.b.write_to(w)?;
            Ok(())
        }
    }

    struct C {
        c: String,
    }

    impl JavaObject for C {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.C", 1714748359461003911)
                    .super_class(B::class())
                    .field(Field::builder("c").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for C {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.c.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_c_extends_b_extends_a() -> io::Result<()> {
        init();

        let a = A {
            a: "fake-a".to_string(),
        };
        let b = B {
            b: "fake-b".to_string(),
        };
        let c = C {
            c: "fake-c".to_string(),
        };

        let c_b_a = c.extends(b.extends(a));

        let raw = c_b_a.to_bytes()?;

        assert_eq!(
            "aced00057372000d636f6d2e6578616d706c652e4317cc028c3cdb62870200014c0001637400124c6a6176612f6c616e672f537472696e673b7872000d636f6d2e6578616d706c652e42697f8137523ed7550200014c00016271007e00017872000d636f6d2e6578616d706c652e41532cdab0df296f690200014c00016171007e0001787074000666616b652d6174000666616b652d6274000666616b652d63",
            hex::encode(&raw)
        );

        Ok(())
    }
}
