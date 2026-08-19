use crate::JavaWriter;
use crate::misc::to_signature;
use crate::proto::{Class, Field, JavaObject, JavaSerializable, JavaWriteable};
use once_cell::sync::Lazy;
use std::io::Write;
use std::sync::Arc;

static CLASS_OF_STACK_TRACE_ELEMENT: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.StackTraceElement", 6992337162326171013)
        .field(Field::builder("format").byte())
        .field(Field::builder("lineNumber").int())
        .field(Field::builder("classLoaderName").string())
        .field(Field::builder("declaringClass").string())
        .field(Field::builder("fileName").string())
        .field(Field::builder("methodName").string())
        .field(Field::builder("moduleName").string())
        .field(Field::builder("moduleVersion").string())
        .build()
});

static CLASS_OF_THROWABLE: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.lang.Throwable", -3042686055658047285)
        .field(Field::builder("detailMessage").string())
        .field(Field::builder("cause").object(to_signature("java.lang.Throwable")))
        .build()
});

pub struct StackTraceElementBuilder<'a> {
    class_loader_name: Option<&'a str>,
    module_name: Option<&'a str>,
    module_version: Option<&'a str>,

    declaring_class: &'a str,
    method_name: &'a str,
    file_name: &'a str,
    line_number: i32,
    format: u8,
}

impl<'a> StackTraceElementBuilder<'a> {
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
            format,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StackTraceElement {
    class_loader_name: Option<String>,
    module_name: Option<String>,
    module_version: Option<String>,
    declaring_class: String,
    method_name: String,
    file_name: String,
    line_number: i32,
    format: u8,
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
            format: 0,
        }
    }
}

impl JavaObject for StackTraceElement {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_STACK_TRACE_ELEMENT)
    }
}

impl JavaSerializable for StackTraceElement {
    fn write_object(&self, w: &mut JavaWriter<&mut dyn Write>) -> std::io::Result<()> {
        // Same order as CLASS_OF_STACK_TRACE_ELEMENT: primitives first, then the strings.
        w.write_byte(self.format)?;
        w.write_int(self.line_number)?;
        write_nullable_string(w, self.class_loader_name.as_deref())?;
        w.write_string(&self.declaring_class)?;
        w.write_string(&self.file_name)?;
        w.write_string(&self.method_name)?;
        write_nullable_string(w, self.module_name.as_deref())?;
        write_nullable_string(w, self.module_version.as_deref())?;
        Ok(())
    }
}

#[inline]
fn write_nullable_string(
    w: &mut JavaWriter<&mut dyn Write>,
    s: Option<&str>,
) -> std::io::Result<()> {
    match s {
        Some(s) => {
            w.write_string(s)?;
        }
        None => w.write_null()?,
    }
    Ok(())
}

#[derive(Debug)]
pub struct Throwable {
    detail_message: String,
    cause: Option<Arc<Throwable>>,
    stack_trace: Vec<StackTraceElement>,
}

impl Throwable {
    pub fn with_message<M>(msg: M) -> Self
    where
        M: Into<String>,
    {
        Self {
            detail_message: msg.into(),
            cause: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn detail_message(&self) -> &str {
        &self.detail_message
    }
}

impl JavaObject for Throwable {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_THROWABLE)
    }
}

impl JavaSerializable for Throwable {
    fn write_object(&self, w: &mut JavaWriter<&mut dyn Write>) -> std::io::Result<()> {
        w.write_string(&self.detail_message)?;
        match &self.cause {
            Some(cause) => cause.write_to(w)?,
            None => w.write_null()?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::proto::JavaWriteable;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_stack_trace_element() -> io::Result<()> {
        init();

        let ste = StackTraceElement::builder(
            "fakeDeclaringClass",
            "fakeMethodName",
            "fakeFileName.java",
            123,
        )
        .build();

        let b = ste.to_bytes()?;

        let actual = hex::encode(&b);
        info!("{:?}: {}", &ste, &actual);

        let expect = "aced00057372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d657400124c6a6176612f6c616e672f537472696e673b4c000e6465636c6172696e67436c61737371007e00014c000866696c654e616d6571007e00014c000a6d6574686f644e616d6571007e00014c000a6d6f64756c654e616d6571007e00014c000d6d6f64756c6556657273696f6e71007e00017870000000007b7074001266616b654465636c6172696e67436c61737374001166616b6546696c654e616d652e6a61766174000e66616b654d6574686f644e616d657070";

        assert_eq!(expect, &actual);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_throwable() -> io::Result<()> {
        init();

        let th = Throwable::with_message("fakeThrowable");

        let b = th.to_bytes()?;

        let expect = "aced0005737200136a6176612e6c616e672e5468726f7761626c65d5c635273977b8cb0300044c000563617573657400154c6a6176612f6c616e672f5468726f7761626c653b4c000d64657461696c4d6573736167657400124c6a6176612f6c616e672f537472696e673b5b000a737461636b547261636574001e5b4c6a6176612f6c616e672f537461636b5472616365456c656d656e743b4c001473757070726573736564457863657074696f6e737400104c6a6176612f7574696c2f4c6973743b787071007e000574000d66616b655468726f7761626c657572001e5b4c6a6176612e6c616e672e537461636b5472616365456c656d656e743b02462a3c3cfd22390200007870000000017372001b6a6176612e6c616e672e537461636b5472616365456c656d656e746109c59a2636dd85020008420006666f726d617449000a6c696e654e756d6265724c000f636c6173734c6f616465724e616d6571007e00024c000e6465636c6172696e67436c61737371007e00024c000866696c654e616d6571007e00024c000a6d6574686f644e616d6571007e00024c000a6d6f64756c654e616d6571007e00024c000d6d6f64756c6556657273696f6e71007e000278700100000017740003617070740010636f6d2e6578616d706c652e4675636b7400094675636b2e6a6176617400046d61696e70707372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede020000787078";

        let actual = hex::encode(&b);

        info!("{:?}: {}", &th, &actual);

        assert_eq!(expect, &actual);

        Ok(())
    }
}
