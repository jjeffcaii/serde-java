use super::stack_trace_element::StackTraceElement;

use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, ClassFlags, Field, JavaObject, JavaSerializable, JavaWriteable, ObjectWriter, Reference,
};
use std::io;

pub(crate) static CLASS_OF_THROWABLE: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Throwable", -3042686055658047285)
        .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
        .field(Field::builder("cause").object("Ljava/lang/Throwable;"))
        .field(Field::builder("detailMessage").string())
        .field(Field::builder("stackTrace").array(StackTraceElement::class().signature()))
        .field(Field::builder("suppressedExceptions").object("Ljava/util/List;"))
        .build()
});

pub struct Throwable<C> {
    inner: Inner,
    cause: Option<Reference<C>>,
}

pub struct Inner {
    detail_message: Option<String>,
    stack_trace: Vec<StackTraceElement>,
    suppressed_exceptions: (), // TODO: how to support suppressed_exceptions?
}

impl<C> Throwable<C> {
    pub fn builder<'a>() -> ThrowableBuilder<'a> {
        ThrowableBuilder::default()
    }

    pub fn set_cause(&mut self, cause: Reference<C>) {
        self.cause = Some(cause);
    }

    pub fn detail_message(&self) -> Option<&str> {
        self.inner.detail_message.as_deref()
    }

    pub fn stack_trace(&self) -> &[StackTraceElement] {
        &self.inner.stack_trace
    }
}

impl<C> JavaObject for Throwable<C> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_THROWABLE)
    }
}

impl<C> JavaSerializable for Throwable<C>
where
    C: JavaSerializable + JavaObject + 'static,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        match &self.cause {
            Some(cause) => cause.write_to(w)?,
            None => ().write_to(w)?,
        }

        match &self.inner.detail_message {
            Some(s) => s.write_to(w)?,
            None => ().write_to(w)?,
        }

        self.inner.stack_trace.write_to(w)?;

        {
            use crate::jdk::EmptyList;
            // use serde_java::Layout;

            let _ = &self.inner.suppressed_exceptions;

            // TODO: handle suppressed_exceptions
            // match &self.suppressed_exceptions {
            //     None => EmptyList.write_to(w)?, // use EmptyList by default in java
            //     Some(ex) => ArrayList::layout(&ex).write_to(w)?,
            // }
            EmptyList.write_to(w)?;
        }

        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct ThrowableBuilder<'a> {
    detail_message: Option<&'a str>,
    stack_trace: Vec<StackTraceElement>,
}

impl<'a> ThrowableBuilder<'a> {
    pub fn detail_message(mut self, detail_message: &'a str) -> Self {
        self.detail_message.replace(detail_message);
        self
    }

    pub fn stack_trace<T>(mut self, stack_trace: T) -> Self
    where
        T: Into<StackTraceElement>,
    {
        self.stack_trace.push(stack_trace.into());
        self
    }

    pub fn build<C>(self) -> Throwable<C> {
        let Self {
            detail_message,
            stack_trace,
        } = self;

        let detail_message = detail_message.map(|s| s.to_owned());
        let inner = Inner {
            detail_message,
            stack_trace,
            suppressed_exceptions: (),
        };

        Throwable {
            inner,
            cause: None as Option<Reference<C>>,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::jdk::FormatFlags;
    use serde_java::{JavaWriteableExt, Pointer};
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_stack_trace_element() -> io::Result<()> {
        init();

        let st = StackTraceElement::builder(
            "fakeDeclaringClass",
            "fakeMethodName",
            "fakeFileName.java",
            123,
        )
        .build();

        info!("stack: {:?}", &st);

        let raw = st.to_bytes()?;

        assert_eq!(
            "aced00057372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d657400124c6a6176612f6c616e672f537472696e673b4c000e6465636c6172696e67436c61737371007e00014c000866696c654e616d6571007e00014c000a6d6574686f644e616d6571007e00014c000a6d6f64756c654e616d6571007e00014c000d6d6f64756c6556657273696f6e71007e00017870000000007b7074001266616b654465636c6172696e67436c61737374001166616b6546696c654e616d652e6a61766174000e66616b654d6574686f644e616d657070",
            hex::encode(&raw)
        );
        Ok(())
    }

    #[test]
    fn test_java_lang_throwable() -> io::Result<()> {
        init();

        // ===== com/example/ThrowableDemo.java =====
        //
        // package com.example;
        //
        // import org.apache.commons.codec.binary.Hex;
        // import org.apache.commons.lang3.SerializationUtils;
        //
        // public class ThrowableDemo {
        //
        //   public static void main(String[] args) {
        // 	   Throwable th = new Throwable("fake");
        // 	   byte[] raw = SerializationUtils.serialize(th);
        // 	   System.out.println("result: " + Hex.encodeHexString(raw));
        //   }
        //
        // }

        let stack = StackTraceElement::builder(
            "com.example.ThrowableDemo",
            "main",
            "ThrowableDemo.java",
            9,
        )
        .format(FormatFlags::BUILTIN_CLASS_LOADER)
        .class_loader_name("app")
        .build();

        let th = Throwable::<Pointer>::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();

        let th = Reference::new(th);

        // the field of cause is yourself
        th.borrow_mut()
            .set_cause(Reference::new(Pointer::from(th.key())));

        let raw = th.to_bytes()?;

        assert_eq!(
            "aced0005737200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000574000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00024c000e6465636c6172696e67436c61737371007e00024c000866696c654e616d6571007e00024c000a6d6574686f644e616d6571007e00024c000a6d6f64756c654e616d6571007e00024c000d6d6f64756c6556657273696f6e71007e000278700100000009740003617070740019636f6d2e6578616d706c652e5468726f7761626c6544656d6f7400125468726f7761626c6544656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
    }
}
