use super::stack_trace_element::StackTraceElement;
use super::throwable::{CLASS_OF_THROWABLE, Throwable};
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, Extends, ExtendsLayout, JavaObject, JavaSerializable, ObjectWriter, Reference,
    ReferenceID,
};
use std::io;

static CLASS_OF_ERROR: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Error", 4980196508277280342)
        .super_class(Clone::clone(&CLASS_OF_THROWABLE))
        .build()
});

#[derive(Clone)]
struct Inner;

impl JavaObject for Inner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_ERROR)
    }
}

impl JavaSerializable for Inner {
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct Error<C = ReferenceID>(ExtendsLayout<Inner, Throwable<C>>);

impl<C> Error<C> {
    pub(crate) fn new(parent: Throwable<C>) -> Self {
        Self(Inner.extends(parent))
    }

    pub fn builder<'a>() -> ErrorBuilder<'a> {
        ErrorBuilder {
            detail_message: None,
            stack: vec![],
        }
    }

    pub fn set_cause(&mut self, cause: Reference<C>) {
        self.0.parent_mut().set_cause(cause);
    }
}

impl<C> JavaObject for Error<C> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_ERROR)
    }
}

impl<C> JavaSerializable for Error<C>
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

pub struct ErrorBuilder<'a> {
    detail_message: Option<&'a str>,
    stack: Vec<StackTraceElement>,
}

impl<'a> ErrorBuilder<'a> {
    pub fn detail_message(mut self, message: &'a str) -> Self {
        self.detail_message = Some(message);
        self
    }

    pub fn stack_trace<T>(mut self, element: T) -> Self
    where
        T: Into<StackTraceElement>,
    {
        self.stack.push(element.into());
        self
    }

    pub fn build(self) -> Reference<Error<ReferenceID>> {
        let e = {
            let c: Error<ReferenceID> = self.build_();
            Reference::new(c)
        };

        // bind cause to self
        {
            let cause = Reference::from(e.id());
            e.borrow_mut().set_cause(cause);
        }

        e
    }

    pub fn build_with_cause<C>(self, cause: &Reference<C>) -> Error<C> {
        let mut ex = self.build_::<C>();

        ex.set_cause(Clone::clone(cause));

        ex
    }

    fn build_<C>(self) -> Error<C> {
        let Self {
            detail_message,
            stack,
        } = self;

        let th: Throwable<C> = {
            let mut bu = Throwable::<C>::builder();

            if let Some(s) = detail_message {
                bu = bu.detail_message(s);
            }

            for next in stack {
                bu = bu.stack_trace(next);
            }

            bu.build()
        };

        Error::new(th)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::jdk::exceptions::{FormatFlags, StackTraceElement};
    use serde_java::{JavaWriteableExt, ReferenceID};
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_error() -> io::Result<()> {
        init();

        // ===== com/example/ErrorDemo.java =====
        //
        // package com.example;
        //
        // import org.apache.commons.codec.binary.Hex;
        // import org.apache.commons.io.FileUtils;
        //
        // public class ErrorDemo {
        //
        //   public static void main(String[] args) throws Exception {
        // 	   Error ex = new Error("fake");
        //
        // 	   byte[] raw = SerializationUtils.serialize(ex);
        // 	   System.out.println("result: " + Hex.encodeHexString(raw));
        //   }
        //
        // }

        let stack =
            StackTraceElement::builder("com.example.ErrorDemo", "main", "ErrorDemo.java", 9)
                .format(FormatFlags::BUILTIN_CLASS_LOADER)
                .class_loader_name("app")
                .build();

        let ex = Error::<ReferenceID>::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();

        let raw = ex.to_bytes()?;

        assert_eq!(
            "aced00057372000f6a6176612e6c616e672e4572726f72451d36568b820e56020000787200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000674000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00034c000e6465636c6172696e67436c61737371007e00034c000866696c654e616d6571007e00034c000a6d6574686f644e616d6571007e00034c000a6d6f64756c654e616d6571007e00034c000d6d6f64756c6556657273696f6e71007e000378700100000009740003617070740015636f6d2e6578616d706c652e4572726f7244656d6f74000e4572726f7244656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
    }
}
