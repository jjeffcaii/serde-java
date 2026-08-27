use super::throwable::{CLASS_OF_THROWABLE, Throwable};
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, Extends, ExtendsLayout, JavaObject, JavaSerializable, ObjectWriter, Reference,
};
use std::io;

pub(crate) static CLASS_OF_EXCEPTION: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Exception", -3387516993124229948)
        .super_class(Clone::clone(&CLASS_OF_THROWABLE))
        .build()
});

#[derive(Default)]
struct ExceptionInner {}

impl JavaObject for ExceptionInner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_EXCEPTION)
    }
}

impl JavaSerializable for ExceptionInner {
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

pub struct Exception<C>(ExtendsLayout<ExceptionInner, Throwable<C>>);

impl<C> Exception<C> {
    pub fn new(parent: Throwable<C>) -> Self {
        Exception(ExceptionInner::default().extends(parent))
    }

    pub fn set_cause(&mut self, cause: Reference<C>) {
        self.0.parent_mut().set_cause(cause)
    }
}

impl<C> JavaObject for Exception<C> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_EXCEPTION)
    }
}

impl<C> JavaSerializable for Exception<C>
where
    C: JavaSerializable + JavaObject + 'static,
{
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_object(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::stack_trace_element::{FormatFlags, StackTraceElement};
    use serde_java::{JavaWriteableExt, Pointer};
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_exception() -> io::Result<()> {
        init();

        // ===== com/example/ExceptionDemo.java =====
        //
        // package com.example;
        //
        // import org.apache.commons.codec.binary.Hex;
        // import org.apache.commons.io.FileUtils;
        //
        // public class ExceptionDemo {
        //
        //   public static void main(String[] args) throws Exception {
        // 	   Exception ex = new Exception("fake");
        //
        // 	   byte[] raw = SerializationUtils.serialize(ex);
        // 	   System.out.println("result: " + Hex.encodeHexString(raw));
        //   }
        //
        // }

        let stack = StackTraceElement::builder(
            "com.example.ExceptionDemo",
            "main",
            "ExceptionDemo.java",
            9,
        )
        .format(FormatFlags::BUILTIN_CLASS_LOADER)
        .class_loader_name("app")
        .build();

        let th = Throwable::<Pointer>::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();
        let ex = Reference::new(Exception::<Pointer>::new(th));
        ex.borrow_mut()
            .set_cause(Reference::new(Pointer::from(ex.key())));

        let raw = ex.to_bytes()?;

        assert_eq!(
            "aced0005737200136a6176612e6c616e672e457863657074696f6ed0fd1f3e1a3b1cc4020000787200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000674000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00034c000e6465636c6172696e67436c61737371007e00034c000866696c654e616d6571007e00034c000a6d6574686f644e616d6571007e00034c000a6d6f64756c654e616d6571007e00034c000d6d6f64756c6556657273696f6e71007e000378700100000009740003617070740019636f6d2e6578616d706c652e457863657074696f6e44656d6f740012457863657074696f6e44656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
    }
}
