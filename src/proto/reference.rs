use super::{JavaObject, JavaSerializable, JavaWriteable, Object, ObjectWriter};
use crate::Class;
use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::io;
use std::ops::Deref;
use std::rc::Rc;

pub struct Reference<T>(pub(crate) Rc<RefCell<T>>);

impl<T> fmt::Debug for Reference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reference")
            .field("key", &self.key())
            .finish()
    }
}

impl<T> fmt::Display for Reference<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reference({})", self.key())
    }
}

impl<T> Reference<T> {
    #[inline]
    pub fn new(t: T) -> Self {
        Self(Rc::new(RefCell::new(t)))
    }

    pub fn id(&self) -> ReferenceID {
        let key = self.key();
        ReferenceID(key)
    }

    #[inline]
    pub(crate) fn key(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}

impl<T> From<T> for Reference<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for Reference<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for Reference<T> {
    fn clone(&self) -> Self {
        let inner = Clone::clone(&self.0);
        Reference(inner)
    }
}

pub struct ReferenceID(usize);

impl JavaObject for ReferenceID {
    fn class() -> Class {
        unreachable!()
    }
}

impl JavaSerializable for ReferenceID {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        unreachable!()
    }
}

impl<T> JavaWriteable for Reference<T>
where
    T: JavaSerializable + JavaObject + 'static,
{
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let borrowed = &*self.0.borrow();

        // 1. write reference key directly for T:Key
        if let Some(k) = (borrowed as &dyn Any).downcast_ref::<ReferenceID>() {
            match w.object_handles.get(&k.0) {
                None => panic!("cannot found ref"),
                Some(h) => {
                    return w.write_reference(*h);
                }
            }
        }

        // 2. write automatically
        let key = self.key();
        let class = T::class();
        let obj = Object::<T, ()>::builder(class, &borrowed).key(key).build();

        obj.write_to(w)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Class, Field, JavaObject, JavaSerializable, JavaWriteable, JavaWriteableExt, ObjectWriter,
        Writer,
    };
    use anyhow::Result;
    use once_cell::sync::Lazy;
    use std::io;

    struct RefDemo {
        id: i64,
        link: Option<Reference<RefDemo>>,
        data: String,
    }

    impl JavaObject for RefDemo {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.RefDemo", 8952467648642045509)
                    .field(Field::builder("id").long())
                    .field(Field::builder("data").string())
                    .field(Field::builder("link").object("Lcom/example/RefDemo;"))
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for RefDemo {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.id.write_to(w)?;
            self.data.write_to(w)?;

            match &self.link {
                None => {
                    w.write(())?;
                }
                Some(link) => {
                    link.write_to(w)?;
                }
            }

            Ok(())
        }
    }

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_reference() -> Result<()> {
        init();

        // ==== the origin Java codes ====
        //
        // package com.example;
        //
        // import java.io.IOException;
        // import lombok.Data;
        // import org.apache.commons.codec.binary.Hex;
        // import org.apache.commons.lang3.SerializationUtils;
        //
        // @Data
        // public class RefDemo implements java.io.Serializable {
        //
        //   private static final long serialVersionUID = 8952467648642045509L;
        //
        //   private long id;
        //   private RefDemo link;
        //   private String data;
        //
        //   public static void main(String[] args) throws IOException {
        // 	   RefDemo root = new RefDemo();
        // 	   root.setId(0x0102030405060708L);
        // 	   root.setData("foobar");
        // 	   root.setLink(root);
        //
        // 	   byte[] raw = SerializationUtils.serialize(root);
        //
        // 	   System.out.println(Hex.encodeHexString(raw));
        //   }
        //
        // }

        let r = {
            let origin = RefDemo {
                id: 0x0102030405060708,
                link: None,
                data: "foobar".to_string(),
            };

            Reference::new(origin)
        };
        // write self-reference
        r.borrow_mut().link.replace(Clone::clone(&r));

        info!("reference: {:?}", r);
        info!("reference: {}", r);

        let raw = r.to_bytes()?;

        assert_eq!(
            "aced000573720013636f6d2e6578616d706c652e52656644656d6f7c3d8de4ec7c56450200034a000269644c0004646174617400124c6a6176612f6c616e672f537472696e673b4c00046c696e6b7400154c636f6d2f6578616d706c652f52656644656d6f3b78700102030405060708740006666f6f62617271007e0003",
            hex::encode(&raw)
        );

        Ok(())
    }
}
