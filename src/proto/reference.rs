use super::{JavaObject, JavaSerializable, JavaWriteable, ObjectView, ObjectWriter};
use crate::Class;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::any::Any;
use std::fmt;
use std::io;
use std::ops::Deref;
use std::sync::Arc;

pub struct Reference<T>(pub(crate) Arc<RwLock<T>>);

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
        Self(Arc::new(RwLock::new(t)))
    }

    pub fn id(&self) -> ReferenceID {
        let key = self.key();
        ReferenceID(key)
    }

    #[inline]
    pub(crate) fn key(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    #[inline]
    pub fn borrow(&self) -> RwLockReadGuard<'_, T> {
        self.0.read()
    }

    #[inline]
    pub fn borrow_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write()
    }
}

impl<T> From<T> for Reference<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for Reference<T> {
    type Target = Arc<RwLock<T>>;

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
        // Self-referential graphs (see `test_reference`) recurse back into this same
        // lock while the outer read guard below is still held. parking_lot supports
        // that as long as no writer is queued in between, which never happens here
        // since serialization is single-threaded and only ever takes read locks.
        let borrowed = &*self.0.read();

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
        let obj = ObjectView::<T, ()>::builder(class, borrowed)
            .key(key)
            .build();

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

    #[test]
    fn test_reference_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Reference<RefDemo>>();
    }

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_reference() -> Result<()> {
        init();

        let r = {
            let origin = RefDemo {
                id: 72623859790382856,
                link: None,
                data: "foobar".to_string(),
            };

            Reference::new(origin)
        };
        // write self-reference
        r.borrow_mut().link.replace(Clone::clone(&r));

        info!("reference: {:?}", r);
        info!("reference: {}", r);

        // the self-referenced JSON in fastjson:
        // ```java
        //  RefDemo d = new RefDemo();
        // 	d.setId(0x0102030405060708L);
        // 	d.setData("foobar");
        // 	d.setLink(d);
        //  String result = JSON.toJSONString(d, Feature.ReferenceDetection)
        // ```
        let json = r#"{"id":72623859790382856,"data":"foobar","link":{"$ref":".."}}"#;

        let actual =
            crate::testkit::serialize(RefDemo::class().name(), &["testdata/RefDemo.java"], json)?;

        let b = hex::encode(&actual);

        info!("binary: {}", &b);

        let raw = r.to_bytes()?;

        assert_eq!(actual, raw);

        Ok(())
    }
}
