use serde_java::__private::bitflags::bitflags;
use serde_java::JavaSerialize;

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
