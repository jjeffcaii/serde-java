use crate::astr::AtomString;
use crate::suid::{self, ClassMetadata};
use bitflags::bitflags;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

bitflags! {
    #[derive(Default,Clone,Copy)]
    pub struct ClassFlags: u8 {
        const WRITE_METHOD = 0x01;
        const SERIALIZABLE = 0x01<<1;
        const EXTERNALIZABLE = 0x01<<2;
        const BLOCK_DATA = 0x01<<3;
        const ENUM = 0x10; // why???
    }
}

#[derive(Eq, PartialEq)]
pub struct FieldName(AtomString);

impl<A> From<A> for FieldName
where
    A: Into<AtomString>,
{
    fn from(value: A) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for FieldName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for FieldName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TypeCode {
    Boolean = b'Z',
    Byte = b'B',
    Char = b'C',
    Short = b'S',
    Int = b'I',
    Long = b'J',
    Float = b'F',
    Double = b'D',
}

impl TypeCode {
    #[inline]
    pub fn signature(self) -> &'static str {
        match self {
            TypeCode::Boolean => "Z",
            TypeCode::Byte => "B",
            TypeCode::Char => "C",
            TypeCode::Short => "S",
            TypeCode::Int => "I",
            TypeCode::Long => "J",
            TypeCode::Float => "F",
            TypeCode::Double => "D",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FieldKind {
    Primitive(/* type_code */ TypeCode),
    Object(/* signature */ AtomString),
    Array(/* signature */ AtomString),
}

impl FieldKind {
    #[inline]
    pub fn is_primitive(&self) -> bool {
        matches!(self, FieldKind::Primitive(_))
    }

    pub fn of_string() -> Self {
        static KIND: Lazy<FieldKind> =
            Lazy::new(|| FieldKind::Object(crate::util::compute_signature("java.lang.String")));
        Clone::clone(&KIND)
    }
}

impl fmt::Display for FieldKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldKind::Primitive(x) => fmt::Display::fmt(x.signature(), f),
            FieldKind::Object(x) => fmt::Display::fmt(x.deref(), f),
            FieldKind::Array(x) => fmt::Display::fmt(x.as_ref(), f),
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct Field(FieldName, FieldKind);

impl Field {
    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn kind(&self) -> &FieldKind {
        &self.1
    }
}

/// Mirrors `java.io.ObjectStreamField#compareTo`: primitive fields come before
/// object/array fields, then alphabetically by name.
impl Ord for Field {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.1.is_primitive(), other.1.is_primitive()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => self.name().cmp(other.name()),
        }
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.0.as_ref(), self.1)
    }
}

pub struct FieldBuilder<'a> {
    name: &'a str,
}

impl<'a> FieldBuilder<'a> {
    pub fn boolean(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Boolean)
    }

    pub fn byte(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Byte)
    }

    pub fn char(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Char)
    }

    pub fn short(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Short)
    }

    pub fn int(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Int)
    }

    pub fn long(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Long)
    }

    pub fn float(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Float)
    }

    pub fn double(self) -> Field {
        let Self { name } = self;
        Field::with_primitive(name, TypeCode::Double)
    }

    pub fn string(self) -> Field {
        let Self { name } = self;
        Field::new(name, FieldKind::of_string())
    }

    pub fn array<C>(self, class_sig: C) -> Field
    where
        C: AsRef<str>,
    {
        let Self { name } = self;
        let array_class_sig = AtomString::from(format!("[{}", class_sig.as_ref()));
        let kind = FieldKind::Array(array_class_sig);
        Field::new(name, kind)
    }

    pub fn object<C>(self, class: C) -> Field
    where
        C: AsRef<str>,
    {
        let Self { name } = self;
        let kind = FieldKind::Object(class.as_ref().into());
        Field::new(name, kind)
    }

    pub fn boolean_array(self) -> Field {
        self.array(TypeCode::Boolean.signature())
    }

    pub fn byte_array(self) -> Field {
        self.array(TypeCode::Byte.signature())
    }

    pub fn short_array(self) -> Field {
        self.array(TypeCode::Short.signature())
    }

    pub fn int_array(self) -> Field {
        self.array(TypeCode::Int.signature())
    }

    pub fn long_array(self) -> Field {
        self.array(TypeCode::Long.signature())
    }

    pub fn float_array(self) -> Field {
        self.array(TypeCode::Float.signature())
    }

    pub fn double_array(self) -> Field {
        self.array(TypeCode::Double.signature())
    }

    pub fn string_array(self) -> Field {
        self.array(FieldKind::of_string().to_string())
    }
}

impl Field {
    pub fn builder(name: &str) -> FieldBuilder<'_> {
        FieldBuilder { name }
    }

    #[inline]
    fn new<S>(name: S, kind: FieldKind) -> Self
    where
        S: AsRef<str>,
    {
        Field(name.as_ref().into(), kind)
    }

    #[inline]
    fn with_primitive<I>(name: I, type_code: TypeCode) -> Self
    where
        I: AsRef<str>,
    {
        let name = FieldName::from(name.as_ref());
        let kind = FieldKind::Primitive(type_code);
        Self(name, kind)
    }
}

pub struct ClassBuilder<'a> {
    name: &'a str,
    signature: Option<&'a str>,
    serial_version_uid: i64,
    flags: ClassFlags,
    fields: Vec<Field>,
    super_class: Option<Class>,
}

struct RawClass {
    pub(crate) name: AtomString,
    pub(crate) signature: AtomString,
    pub(crate) serial_version_uid: i64,
    pub(crate) flags: ClassFlags,
    pub(crate) fields: Vec<Field>,
    pub(crate) super_class: Option<Class>,
}

#[derive(Clone)]
pub struct Class {
    raw: Arc<RawClass>,
}

impl Class {
    #[inline]
    fn class_of_array_(item_class: Class, serial_version_uid: i64) -> Class {
        let array_class_name = format!("[L{};", item_class.name());
        let array_class_sig = format!("[{}", item_class.signature());
        Class::builder(&array_class_name, serial_version_uid)
            .signature(&array_class_sig)
            .build()
    }

    /// Class descriptor for `T[]` where `T` is a non-primitive class.
    ///
    /// Array classes declare no serialVersionUID, so the JVM falls back to
    /// `ObjectStreamClass.computeDefaultSUID` over a synthetic descriptor: the array class
    /// name, modifiers `public final abstract`, and no interfaces, fields, constructors or
    /// methods. Verified against every hard-coded array SUID in this file.
    pub fn class_of_array(item_class: &Class) -> Class {
        let array_class_name = format!("[L{};", item_class.name());
        let meta = ClassMetadata {
            class_name: &array_class_name,
            class_modifiers: suid::PUBLIC | suid::FINAL | suid::ABSTRACT,
            is_interface: false,
            interfaces: vec![],
            fields: vec![],
            has_static_initializer: false,
            constructors: vec![],
            methods: vec![],
        };
        let serial_version_uid = suid::compute_default_suid(&meta);
        Class::class_of_array_(Clone::clone(item_class), serial_version_uid)
    }

    pub fn class_of_boolean_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[Z", 6309297032502205922)
                .signature("[Z")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_char_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[C", -5753798564021173076)
                .signature("[C")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_byte_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[B", -5984413125824719648)
                .signature("[B")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_short_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[S", -1188055269542874886)
                .signature("[S")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_int_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[I", 5600894804908749477)
                .signature("[I")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_long_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[J", 8655923659555304851)
                .signature("[J")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_float_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[F", 836686056779680834)
                .signature("[F")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_double_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[D", 4514449696888150558)
                .signature("[D")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn class_of_string_array() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("[Ljava.lang.String;", -5921575005990323385)
                .signature("[Ljava/lang/String;")
                .build()
        });
        Clone::clone(&CLASS)
    }

    pub fn name(&self) -> &str {
        &self.raw.name
    }

    pub(crate) fn cached_name(&self) -> AtomString {
        Clone::clone(&self.raw.name)
    }

    pub fn signature(&self) -> &str {
        &self.raw.signature
    }

    pub(crate) fn cached_signature(&self) -> AtomString {
        Clone::clone(&self.raw.signature)
    }

    pub fn serial_version_uid(&self) -> i64 {
        self.raw.serial_version_uid
    }

    pub fn flags(&self) -> ClassFlags {
        self.raw.flags
    }

    pub fn fields(&self) -> &[Field] {
        &self.raw.fields
    }

    pub fn super_class(&self) -> Option<&Class> {
        self.raw.super_class.as_ref()
    }

    pub fn builder(class: &str, serial_version_uid: i64) -> ClassBuilder<'_> {
        ClassBuilder {
            name: class,
            signature: None,
            serial_version_uid,
            flags: ClassFlags::SERIALIZABLE,
            fields: vec![],
            super_class: None,
        }
    }
}

impl<'a> ClassBuilder<'a> {
    pub fn signature(mut self, signature: &'a str) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn flags(mut self, flags: ClassFlags) -> ClassBuilder<'a> {
        self.flags = flags;
        self
    }

    pub fn field(mut self, field: Field) -> ClassBuilder<'a> {
        self.fields.push(field);
        self
    }

    pub fn super_class(mut self, super_class: Class) -> ClassBuilder<'a> {
        self.super_class = Some(super_class);
        self
    }

    pub fn sorted(mut self) -> ClassBuilder<'a> {
        self.fields.sort();
        self
    }

    pub fn build(self) -> Class {
        let Self {
            name,
            signature,
            serial_version_uid,
            flags,
            fields,
            super_class,
        } = self;

        let rc = RawClass {
            name: AtomString::from(name),
            signature: match signature {
                Some(s) => AtomString::from(s),
                None => crate::util::compute_signature(name),
            },
            serial_version_uid,
            flags,
            fields,
            super_class,
        };

        Class { raw: Arc::new(rc) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_field() {
        init();

        let f = Field::builder("f").long();
        info!("{}", f);
        assert_eq!("f", f.name());
        assert_eq!("J", &f.kind().to_string());

        let f = Field::builder("f").string();
        info!("{}", f);
        assert_eq!("f", f.name());
        assert_eq!("Ljava/lang/String;", &f.kind().to_string());

        let f = Field::builder("f").int_array();
        info!("{}", f);
        assert_eq!("f", f.name());
        assert_eq!("[I", &f.kind().to_string());

        let f = Field::builder("f").string_array();
        info!("{}", f);
        assert_eq!("f", f.name());
        assert_eq!("[Ljava/lang/String;", &f.kind().to_string());
    }

    #[test]
    fn test_field_sort() {
        init();

        // declaration order of examples/example.java
        let mut fields = [
            Field::builder("id").long(),
            Field::builder("name").string(),
            Field::builder("age").int(),
            Field::builder("addresses").array("Lcom/example/Address;"),
            Field::builder("ext1").object("java.util.Map"),
            Field::builder("ext2").object("java.util.Map"),
        ];
        fields.sort();

        let actual: Vec<&str> = fields.iter().map(Field::name).collect();
        assert_eq!(
            vec!["age", "id", "addresses", "ext1", "ext2", "name"],
            actual
        );
    }

    #[test]
    fn test_class_of_object_array() {
        init();

        // com.example.Address's own SUID comes from examples/example.rs and is irrelevant here:
        // an array class's SUID depends only on the element class's name.
        let item = Class::builder("com.example.Address", -4433675896693646393).build();
        let arr = Class::class_of_array(&item);

        info!("{} => {}", arr.name(), arr.serial_version_uid());

        assert_eq!("[Lcom.example.Address;", arr.name());
        assert_eq!("[Lcom/example/Address;", arr.signature());
        // Expected value: the magic number that used to be hard-coded at examples/example.rs:130
        assert_eq!(7549007861314292831, arr.serial_version_uid());
    }

    #[test]
    fn test_class_of_object_array_matches_builtin_string_array() {
        init();

        // Cross-check against the hard-coded SUID in this file's class_of_string_array()
        let item = Class::builder("java.lang.String", 0).build();
        let arr = Class::class_of_array(&item);

        assert_eq!(
            Class::class_of_string_array().serial_version_uid(),
            arr.serial_version_uid()
        );
    }
}
