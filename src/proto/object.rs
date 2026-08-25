use super::{Class, JavaSerializable, ObjectWriter};
use crate::ClassFlags;
use std::io;

impl JavaSerializable for () {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

pub struct Object<'a, T, P> {
    class: Class,
    instance: Option<&'a T>,
    super_instance: Option<&'a P>,
}

pub struct ObjectBuilder<'a, T, P> {
    inner: Object<'a, T, P>,
}

impl<'a, T, P> ObjectBuilder<'a, T, P> {
    pub fn extends(mut self, parent: &'a P) -> Self {
        self.inner.super_instance.replace(parent);
        self
    }

    pub fn this(mut self, this: &'a T) -> Self {
        self.inner.instance.replace(this);
        self
    }

    pub fn build(self) -> Object<'a, T, P> {
        let Self { inner } = self;
        inner
    }
}

impl<'a, I, P> Object<'a, I, P> {
    pub fn builder(class: Class) -> ObjectBuilder<'a, I, P> {
        ObjectBuilder {
            inner: Object {
                class,
                instance: None,
                super_instance: None,
            },
        }
    }

    pub fn class(&self) -> Class {
        Clone::clone(&self.class)
    }

    pub fn this(&self) -> Option<&'a I> {
        self.instance
    }

    pub fn extends(&self) -> Option<&'a P> {
        self.super_instance
    }

    pub fn write_to<W>(&self, w: &mut ObjectWriter<W>) -> io::Result<()>
    where
        W: io::Write,
        P: JavaSerializable,
        I: JavaSerializable,
    {
        let old = w.set_block_data_mode(true);

        w.begin_object(&self.class)?;

        if let Some(p) = &self.super_instance {
            w.with_dyn(|w| p.write_object(w))?;
        }

        if let Some(i) = &self.instance {
            w.with_dyn(|w| i.write_object(w))?;
        }

        if self.class.flags().contains(ClassFlags::WRITE_METHOD) {
            w.end()?;
        }

        w.set_block_data_mode(old);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, Writer};
    use std::io;
    use std::io::Write;

    struct PojoA {
        id: i32,
    }

    impl JavaSerializable for PojoA {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn Write>) -> io::Result<()> {
            w.write(self.id)?;
            Ok(())
        }
    }

    struct PojoB {
        name: String,
    }

    impl JavaSerializable for PojoB {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn Write>) -> io::Result<()> {
            w.write(&self.name)
        }
    }

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_serialize() -> io::Result<()> {
        init();

        let class_b = Class::builder("com.example.PojoB", 311662489269302022)
            .field(Field::builder("name").string())
            .build();

        let class_a = Class::builder("com.example.PojoA", -4387630733789281459)
            .field(Field::builder("id").int())
            .super_class(Clone::clone(&class_b))
            .build();

        let b = PojoB {
            name: "helloworld".to_string(),
        };

        let a = PojoA { id: 0xff };

        let obj = Object::<PojoA, PojoB>::builder(class_a)
            .this(&a)
            .extends(&b)
            .build();

        let mut raw: Vec<u8> = vec![];

        {
            let mut jw = ObjectWriter::new(&mut raw)?;

            obj.write_to(&mut jw)?;

            info!("encode result: {}", hex::encode(&raw));
        }

        assert_eq!(
            "aced000573720011636f6d2e6578616d706c652e506f6a6f41c31c011822e7434d020001490002696478720011636f6d2e6578616d706c652e506f6a6f4204533f61fab5f3060200014c00046e616d657400124c6a6176612f6c616e672f537472696e673b787074000a68656c6c6f776f726c64000000ff",
            hex::encode(&raw),
        );

        Ok(())
    }
}
