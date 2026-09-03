use super::exception::{CLASS_OF_EXCEPTION, Exception};
use super::stack_trace_element::StackTraceElement;
use super::throwable::Throwable;
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, Extends, ExtendsLayout, JavaObject, JavaSerializable, ObjectWriter, Reference,
    ReferenceID,
};
use std::io;

static CLASS_OF_RUNTIME_EXCEPTION: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.RuntimeException", -7034897190745766939)
        .super_class(Clone::clone(&CLASS_OF_EXCEPTION))
        .build()
});

#[derive(Default)]
struct Inner;

impl JavaObject for Inner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_RUNTIME_EXCEPTION)
    }
}

impl JavaSerializable for Inner {
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

pub struct RuntimeException<C = ReferenceID>(ExtendsLayout<Inner, Exception<C>>);

impl<C> RuntimeException<C> {
    pub(crate) fn new(parent: Exception<C>) -> Self {
        Self(Inner::default().extends(parent))
    }

    pub fn builder<'a>() -> RuntimeExceptionBuilder<'a> {
        RuntimeExceptionBuilder {
            detail_message: None,
            stack: vec![],
        }
    }

    pub fn set_cause(&mut self, cause: Reference<C>) {
        self.0.parent_mut().set_cause(cause)
    }
}

impl<C> JavaObject for RuntimeException<C> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_RUNTIME_EXCEPTION)
    }
}

impl<C> JavaSerializable for RuntimeException<C>
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

pub struct RuntimeExceptionBuilder<'a> {
    detail_message: Option<&'a str>,
    stack: Vec<StackTraceElement>,
}

impl<'a> RuntimeExceptionBuilder<'a> {
    pub fn detail_message(mut self, msg: &'a str) -> Self {
        self.detail_message = Some(msg);
        self
    }

    pub fn stack_trace(mut self, stack: StackTraceElement) -> Self {
        self.stack.push(stack);
        self
    }

    pub fn build(self) -> Reference<RuntimeException<ReferenceID>> {
        let ex = Reference::new(self.build_::<ReferenceID>());

        // bind cause to self
        {
            let cause = Reference::new(ex.id());
            ex.borrow_mut().set_cause(cause);
        }

        ex
    }

    pub fn build_with_cause<C>(self, cause: &Reference<C>) -> RuntimeException<C> {
        let mut ex = self.build_::<C>();

        ex.set_cause(Clone::clone(cause));

        ex
    }

    #[inline]
    fn build_<C>(self) -> RuntimeException<C> {
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

        RuntimeException::new(Exception::new(th))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::stack_trace_element::{FormatFlags, StackTraceElement};
    use serde_java::JavaWriteableExt;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_java_lang_runtime_exception() -> io::Result<()> {
        init();

        // ===== com/example/RuntimeExceptionDemo.java =====
        //
        // package com.example;
        //
        // import org.apache.commons.codec.binary.Hex;
        // import org.apache.commons.lang3.SerializationUtils;
        //
        // public class RuntimeExceptionDemo {
        //
        //   public static void main(String[] args) throws Exception {
        // 	   RuntimeException ex = new RuntimeException("fake");
        //
        // 	   byte[] raw = SerializationUtils.serialize(ex);
        // 	   System.out.println("result: " + Hex.encodeHexString(raw));
        //   }
        //
        // }

        let stack = StackTraceElement::builder(
            "com.example.RuntimeExceptionDemo",
            "main",
            "RuntimeExceptionDemo.java",
            9,
        )
        .format(FormatFlags::BUILTIN_CLASS_LOADER)
        .class_loader_name("app")
        .build();

        let ex = RuntimeException::<ReferenceID>::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();

        let raw = ex.to_bytes()?;

        assert_eq!(
            "aced00057372001a6a6176612e6c616e672e52756e74696d65457863657074696f6e9e5f06470a3483e5020000787200136a6176612e6c616e672e457863657074696f6ed0fd1f3e1a3b1cc4020000787200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000774000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00044c000e6465636c6172696e67436c61737371007e00044c000866696c654e616d6571007e00044c000a6d6574686f644e616d6571007e00044c000a6d6f64756c654e616d6571007e00044c000d6d6f64756c6556657273696f6e71007e000478700100000009740003617070740020636f6d2e6578616d706c652e52756e74696d65457863657074696f6e44656d6f74001952756e74696d65457863657074696f6e44656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[test]
    fn test_java_lang_runtime_exception_cascaded() -> io::Result<()> {
        init();

        // ===== com/example/RuntimeExceptionDemo.java =====
        //
        // package com.example;
        //
        // import org.apache.commons.codec.binary.Hex;
        // import org.apache.commons.lang3.SerializationUtils;
        //
        // public class RuntimeExceptionDemo {
        //
        //   public static void main(String[] args) throws Exception {
        // 	   RuntimeException cause = new RuntimeException("root");
        //
        // 	   RuntimeException ex = new RuntimeException("fake", cause);
        //
        // 	   byte[] raw = SerializationUtils.serialize(ex);
        // 	   System.out.println("result: " + Hex.encodeHexString(raw));
        //   }
        //
        // }

        let stacker = |line: i32| {
            StackTraceElement::builder(
                "com.example.RuntimeExceptionDemo",
                "main",
                "RuntimeExceptionDemo.java",
                line,
            )
            .format(FormatFlags::BUILTIN_CLASS_LOADER)
            .class_loader_name("app")
            .build()
        };

        let root = {
            RuntimeException::<ReferenceID>::builder()
                .detail_message("root")
                .stack_trace(stacker(9))
                .build()
        };

        let ex = RuntimeException::<()>::builder()
            .detail_message("fake")
            .stack_trace(stacker(10))
            .build_with_cause(&root);

        let raw = ex.to_bytes()?;

        assert_eq!(
            "aced00057372001a6a6176612e6c616e672e52756e74696d65457863657074696f6e9e5f06470a3483e5020000787200136a6176612e6c616e672e457863657074696f6ed0fd1f3e1a3b1cc4020000787200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b78707371007e000071007e0008740004726f6f747572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00044c000e6465636c6172696e67436c61737371007e00044c000866696c654e616d6571007e00044c000a6d6574686f644e616d6571007e00044c000a6d6f64756c654e616d6571007e00044c000d6d6f64756c6556657273696f6e71007e000478700100000009740003617070740020636f6d2e6578616d706c652e52756e74696d65457863657074696f6e44656d6f74001952756e74696d65457863657074696f6e44656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede02000078707874000466616b657571007e000a000000017371007e000c010000000a71007e000e71007e000f71007e001071007e0011707071007e001378",
            hex::encode(&raw)
        );

        Ok(())
    }
}
