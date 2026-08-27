use super::exception::{CLASS_OF_EXCEPTION, Exception};
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{Class, ExtendsLayout, JavaObject, JavaSerializable, JavaWriteable, ObjectWriter};
use std::io;

static CLASS_OF_RUNTIME_EXCEPTION: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.RuntimeException", -7034897190745766939)
        .super_class(Clone::clone(&CLASS_OF_EXCEPTION))
        .build()
});

struct RuntimeExceptionInner;

impl JavaObject for RuntimeExceptionInner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_RUNTIME_EXCEPTION)
    }
}

impl JavaSerializable for RuntimeExceptionInner {
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

pub struct RuntimeException(ExtendsLayout<RuntimeExceptionInner, Exception>);

impl JavaObject for RuntimeException {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_RUNTIME_EXCEPTION)
    }
}

impl JavaWriteable for RuntimeException {
    fn write_to(&self, _w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        unimplemented!()
    }
}
