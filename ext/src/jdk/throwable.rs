use super::list::{ArrayList, EmptyList};
use serde_java::__private::bitflags::bitflags;
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::util::compute_signature;
use serde_java::{
    Class, ClassFlags, Extends, ExtendsLayout, Field, JavaObject, JavaSerializable, JavaSerialize,
    JavaWriteable, Layout, ObjectWriter, Reference,
};
use std::io;

bitflags! {
    #[derive(Default,Clone,Copy)]
    pub struct FormatFlags: u8 {
        const BUILTIN_CLASS_LOADER = 0x01;
        const JDK_NON_UPGRADEABLE_MODULE = 0x01 << 1;
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, JavaSerialize)]
#[java(
    class = "java.lang.StackTraceElement",
    serial_version_uid = 6992337162326171013
)]
pub struct StackTraceElement {
    #[java(rename = "classLoaderName")]
    class_loader_name: Option<String>,
    #[java(rename = "moduleName")]
    module_name: Option<String>,
    #[java(rename = "moduleVersion")]
    module_version: Option<String>,
    #[java(rename = "declaringClass")]
    declaring_class: String,
    #[java(rename = "methodName")]
    method_name: String,
    #[java(rename = "fileName")]
    file_name: String,
    #[java(rename = "lineNumber")]
    line_number: i32,
    #[java(rename = "format")]
    format: u8,
}

pub struct StackTraceElementBuilder<'a> {
    class_loader_name: Option<&'a str>,
    module_name: Option<&'a str>,
    module_version: Option<&'a str>,

    declaring_class: &'a str,
    method_name: &'a str,
    file_name: &'a str,
    line_number: i32,
    format: FormatFlags,
}

impl<'a> StackTraceElementBuilder<'a> {
    pub fn format(mut self, format: FormatFlags) -> Self {
        self.format = format;
        self
    }

    pub fn class_loader_name(mut self, class_loader_name: &'a str) -> Self {
        self.class_loader_name.replace(class_loader_name);
        self
    }

    pub fn build(self) -> StackTraceElement {
        let Self {
            class_loader_name,
            module_name,
            module_version,
            declaring_class,
            method_name,
            file_name,
            line_number,
            format,
        } = self;
        StackTraceElement {
            class_loader_name: class_loader_name.map(|s| s.to_owned()),
            module_name: module_name.map(|s| s.to_owned()),
            module_version: module_version.map(|s| s.to_owned()),
            declaring_class: declaring_class.to_owned(),
            method_name: method_name.to_owned(),
            file_name: file_name.to_owned(),
            line_number,
            format: format.bits(),
        }
    }
}

impl<'a> Into<StackTraceElement> for StackTraceElementBuilder<'a> {
    fn into(self) -> StackTraceElement {
        self.build()
    }
}

impl StackTraceElement {
    pub fn builder<'a>(
        declaring_class: &'a str,
        method_name: &'a str,
        file_name: &'a str,
        line_number: i32,
    ) -> StackTraceElementBuilder<'a> {
        StackTraceElementBuilder {
            class_loader_name: None,
            module_name: None,
            module_version: None,
            declaring_class,
            method_name,
            file_name,
            line_number,
            format: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Throwable {
    detail_message: Option<String>,
    cause: Option<Reference<Throwable>>,
    stack_trace: Vec<StackTraceElement>,
    suppressed_exceptions: Option<Vec<Reference<Throwable>>>,
}

#[derive(Default)]
pub struct ThrowableBuilder<'a> {
    detail_message: Option<&'a str>,
    cause: Option<&'a Reference<Throwable>>,
    stack_trace: Vec<StackTraceElement>,
    suppressed_exceptions: Option<Vec<Reference<Throwable>>>,
}

impl<'a> ThrowableBuilder<'a> {
    pub fn suppressed_exception(mut self, th: Reference<Throwable>) -> Self {
        self.suppressed_exceptions.get_or_insert_default().push(th);
        self
    }

    pub fn detail_message(mut self, detail_message: &'a str) -> Self {
        self.detail_message.replace(detail_message);
        self
    }

    pub fn cause(mut self, cause: &'a Reference<Throwable>) -> Self {
        self.cause.replace(cause);
        self
    }

    pub fn stack_trace<T>(mut self, stack_trace: T) -> Self
    where
        T: Into<StackTraceElement>,
    {
        self.stack_trace.push(stack_trace.into());
        self
    }

    pub fn build(self) -> Reference<Throwable> {
        let Self {
            detail_message,
            cause,
            stack_trace,
            suppressed_exceptions,
        } = self;

        let detail_message = detail_message.map(|s| s.to_owned());

        let th = Reference::from(Throwable {
            detail_message,
            cause: cause.cloned(),
            stack_trace,
            suppressed_exceptions,
        });

        if cause.is_none() {
            th.borrow_mut().cause.replace(Clone::clone(&th));
        }

        th
    }
}

impl Throwable {
    pub fn builder<'a>() -> ThrowableBuilder<'a> {
        ThrowableBuilder::default()
    }

    pub fn detail_message(&self) -> Option<&str> {
        self.detail_message.as_deref()
    }

    pub fn stack_trace(&self) -> &[StackTraceElement] {
        &self.stack_trace
    }

    pub fn cause(&self) -> Option<&Reference<Throwable>> {
        self.cause.as_ref()
    }
}

impl JavaObject for Throwable {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            let name = "java.lang.Throwable";
            let signature = compute_signature(name);
            Class::builder(name, -3042686055658047285)
                .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
                .field(Field::builder("cause").object(signature))
                .field(Field::builder("detailMessage").string())
                .field(Field::builder("stackTrace").array(StackTraceElement::class().signature()))
                .field(Field::builder("suppressedExceptions").object("Ljava/util/List;"))
                .build()
        });

        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for Throwable {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        match &self.cause {
            Some(cause) => cause.write_to(w)?,
            None => ().write_to(w)?,
        }

        match &self.detail_message {
            Some(s) => s.write_to(w)?,
            None => ().write_to(w)?,
        }

        self.stack_trace.write_to(w)?;

        match &self.suppressed_exceptions {
            None => EmptyList.write_to(w)?, // use EmptyList by default in java
            Some(ex) => ArrayList::layout(&ex).write_to(w)?,
        }

        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)?;
        Ok(())
    }
}

static CLASS_OF_EXCEPTION: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Exception", -3387516993124229948)
        .super_class(Throwable::class())
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

pub struct Exception(ExtendsLayout<ExceptionInner, Reference<Throwable>>);

impl Exception {
    pub fn new(parent: Reference<Throwable>) -> Self {
        Exception(ExceptionInner::default().extends(parent))
    }
}

impl JavaObject for Exception {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_EXCEPTION)
    }
}

impl JavaWriteable for Exception {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

static CLASS_OF_ERROR: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Error", 4980196508277280342)
        .super_class(Throwable::class())
        .build()
});

#[derive(Default)]
struct ErrorInner;

impl JavaObject for ErrorInner {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_ERROR)
    }
}

impl JavaSerializable for ErrorInner {
    fn write_fields(&self, _: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }
}

pub struct Error(ExtendsLayout<ErrorInner, Reference<Throwable>>);

impl Error {
    pub fn new(parent: Reference<Throwable>) -> Self {
        Self(ErrorInner::default().extends(parent))
    }
}

impl JavaObject for Error {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_ERROR)
    }
}

impl JavaWriteable for Error {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.0.write_to(w)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

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

        let th = Throwable::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();

        info!("throwable: {:?}", &th);

        let raw = th.to_bytes()?;

        assert_eq!(
            "aced0005737200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000574000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00024c000e6465636c6172696e67436c61737371007e00024c000866696c654e616d6571007e00024c000a6d6574686f644e616d6571007e00024c000a6d6f64756c654e616d6571007e00024c000d6d6f64756c6556657273696f6e71007e000278700100000009740003617070740019636f6d2e6578616d706c652e5468726f7761626c6544656d6f7400125468726f7761626c6544656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
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

        let th = Throwable::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();

        let ex = Exception::new(th);

        let raw = ex.to_bytes()?;

        assert_eq!(
            "aced0005737200136a6176612e6c616e672e457863657074696f6ed0fd1f3e1a3b1cc4020000787200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000674000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00034c000e6465636c6172696e67436c61737371007e00034c000866696c654e616d6571007e00034c000a6d6574686f644e616d6571007e00034c000a6d6f64756c654e616d6571007e00034c000d6d6f64756c6556657273696f6e71007e000378700100000009740003617070740019636f6d2e6578616d706c652e457863657074696f6e44656d6f740012457863657074696f6e44656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
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

        let th = Throwable::builder()
            .detail_message("fake")
            .stack_trace(stack)
            .build();

        let ex = Error::new(th);

        let raw = ex.to_bytes()?;

        assert_eq!(
            "aced00057372000f6a6176612e6c616e672e4572726f72451d36568b820e56020000787200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000674000466616b657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00034c000e6465636c6172696e67436c61737371007e00034c000866696c654e616d6571007e00034c000a6d6574686f644e616d6571007e00034c000a6d6f64756c654e616d6571007e00034c000d6d6f64756c6556657273696f6e71007e000378700100000009740003617070740015636f6d2e6578616d706c652e4572726f7244656d6f74000e4572726f7244656d6f2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078",
            hex::encode(&raw)
        );

        Ok(())
    }
}
