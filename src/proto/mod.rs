mod class;
mod deserializable;
mod extends;
mod flags;
mod object;
mod reader;
mod reference;
mod serializable;
mod writer;

/// Declares which Java class a Rust type maps to — the Rust-side equivalent of that class's
/// `Class`/`ObjectStreamClass` descriptor.
///
/// This is the minimum information needed for a Rust type to be written to the serialization
/// stream as a Java object: a [`Class`] recording the fully-qualified Java class name, its
/// `serialVersionUID`, class flags ([`crate::ClassFlags`]), and its field list sorted the way the
/// JVM sorts it (all primitive fields first, then all object/array fields, each group ordered
/// alphabetically by name — *not* the Rust struct's declaration order). Writing the object header
/// (`TC_OBJECT` + classdesc) reads straight from this descriptor.
///
/// Usually not hand-written: `#[derive(JavaSerialize)]` combined with
/// `#[java(class = "...", serial_version_uid = ...)]` generates this impl automatically, with
/// field sorting resolved at macro-expansion time. A hand-written impl is only needed for cases
/// the derive doesn't cover yet (the `WRITE_METHOD` flag, superclass chains, custom array
/// classes, ...), and should cache the built `Class` in a `once_cell::sync::Lazy` static so
/// `class()` doesn't rebuild it on every call.
///
/// # Hand-written example
///
/// ```
/// use once_cell::sync::Lazy;
/// use serde_java::*;
/// use std::io;
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl JavaObject for Point {
///     fn class() -> Class {
///         // Cache with Lazy: built once on first call, cheap Arc clone afterwards.
///         static CLASS: Lazy<Class> = Lazy::new(|| {
///             Class::builder("com.example.Point", 1)
///                 .field(Field::builder("x").int())
///                 .field(Field::builder("y").int())
///                 .build()
///         });
///         Clone::clone(&CLASS)
///     }
/// }
///
/// impl JavaSerializable for Point {
///     fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
///         self.x.write_to(w)?;
///         self.y.write_to(w)?;
///         Ok(())
///     }
/// }
///
/// let bytes = Point { x: 1, y: 2 }.to_bytes().unwrap();
/// assert!(bytes.starts_with(&[0xAC, 0xED, 0x00, 0x05])); // stream magic
/// ```
pub trait JavaObject: Sized {
    /// Returns the Java `Class` descriptor for this type (name, serialVersionUID, field list, etc).
    ///
    /// May be called multiple times (once per object of this type written to a stream), so
    /// implementations should cache the built value in a `once_cell::sync::Lazy` and
    /// `Clone::clone` it out (`Class` is internally an `Arc`, so cloning is cheap).
    fn class() -> Class;
}

pub use class::{Class, ClassBuilder, ClassFlags, Field, FieldBuilder, FieldKind};
pub use deserializable::{
    JavaDeserializable, JavaFieldsDeserializable, JavaReadable, JavaReadableExt, Reader,
};
pub use extends::{Extends, ExtendsLayout};
pub(crate) use object::ObjectView;
pub use reader::ObjectReader;
pub use reference::{Reference, ReferenceID};
pub use serializable::{JavaSerializable, JavaWriteable, JavaWriteableExt};
pub use writer::{ArrayWriter, ObjectWriter, Writer};

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use once_cell::sync::Lazy;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[derive(Debug, Eq, PartialEq)]
    struct Demo {
        i: i32,
        message: String,
    }

    impl JavaObject for Demo {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Demo", 5151422842377556126)
                    .field(Field::builder("i").int())
                    .field(Field::builder("message").string())
                    .build()
            });

            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Demo {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.i.write_to(w)?;
            self.message.write_to(w)?;
            Ok(())
        }
    }

    impl JavaFieldsDeserializable for Demo {
        type Fields = (i32, String);

        fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
        where
            R: io::Read,
        {
            Ok((r.read()?, r.read()?))
        }
    }

    impl JavaDeserializable for Demo {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let (i, message) = Self::read_fields(r)?;
            Ok(Self { i, message })
        }
    }

    struct Address {
        city: String,
    }

    impl JavaObject for Address {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Address", -4433675896693646393)
                    .field(Field::builder("city").string())
                    .build()
            });

            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Address {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            w.write(&self.city)?;
            Ok(())
        }
    }

    impl JavaFieldsDeserializable for Address {
        type Fields = String;

        fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
        where
            R: io::Read,
        {
            let city = r.read()?;
            Ok(city)
        }
    }

    impl JavaDeserializable for Address {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let city = Self::read_fields(r)?;
            Ok(Self { city })
        }
    }

    struct Order {
        id: i32,
        address: Address,
    }

    impl JavaObject for Order {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Order", 2772851369020234932)
                    .field(Field::builder("id").int())
                    .field(Field::builder("address").object("Lcom/example/Address;"))
                    .build()
            });

            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Order {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.id.write_to(w)?;
            self.address.write_to(w)?;
            Ok(())
        }
    }

    impl JavaFieldsDeserializable for Order {
        type Fields = (i32, Address);

        fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
        where
            R: io::Read,
        {
            let id = r.read()?;
            let address = r.read()?;
            Ok((id, address))
        }
    }

    impl JavaDeserializable for Order {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let (id, address) = Self::read_fields(r)?;
            Ok(Self { id, address })
        }
    }

    #[test]
    fn test_string() -> Result<()> {
        init();

        fn check(input: &str, expect: &str) -> Result<()> {
            let mut b: Vec<u8> = vec![];
            let mut w = ObjectWriter::new(&mut b)?;

            w.write(input)?;

            let actual = hex::encode(&b);

            info!("{}: {}", input, actual);

            assert_eq!(expect, &actual);

            let mut r = ObjectReader::new(&b[..])?;
            let output: String = r.read()?;

            assert_eq!(input, output);

            Ok(())
        }

        // simple
        check("Hello World!", "aced000574000c48656c6c6f20576f726c6421")?;

        // with CJK
        check("Hello 世界!", "aced000574000d48656c6c6f20e4b896e7958c21")?;

        // with emoji
        check(
            "I♥️みかみゆあ!",
            "aced000574001749e299a5efb88fe381bfe3818be381bfe38286e3818221",
        )?;

        Ok(())
    }

    #[test]
    fn test_serialize() -> Result<()> {
        init();

        let origin = Demo {
            i: 42,
            message: "helloWorld".to_string(),
        };

        let raw = origin.to_bytes()?;

        assert_eq!(
            "aced000573720010636f6d2e6578616d706c652e44656d6f477d87c81fbf509e020002490001694c00076d6573736167657400124c6a6176612f6c616e672f537472696e673b78700000002a74000a68656c6c6f576f726c64",
            hex::encode(&raw)
        );

        let parsed = Demo::from_bytes(&raw)?;

        assert_eq!(origin, parsed);

        Ok(())
    }

    #[test]
    fn test_serialize_nested() -> io::Result<()> {
        init();

        let order = Order {
            id: 7,
            address: Address {
                city: "NY".to_string(),
            },
        };

        let raw = order.to_bytes()?;

        assert_eq!(
            "aced000573720011636f6d2e6578616d706c652e4f72646572267b25a101681cb402000249000269644c0007616464726573737400154c636f6d2f6578616d706c652f416464726573733b78700000000773720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200014c0004636974797400124c6a6176612f6c616e672f537472696e673b78707400024e59",
            hex::encode(&raw)
        );

        Ok(())
    }

    struct CustomPojo1 {
        username: String,
        password: String, // transient
        admin: bool,      // transient
        enabled: bool,    // transient
    }

    impl JavaObject for CustomPojo1 {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.CustomPojo", -3231298442776514728)
                    .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
                    .field(Field::builder("username").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for CustomPojo1 {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.username.write_to(w)?;
            Ok(())
        }

        fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.default_write_object(w)?;

            self.admin.write_to(w)?;
            self.enabled.write_to(w)?;
            let enc_password = format!("ENCRYPT_{}", self.password);
            enc_password.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_custom_pojo() -> Result<()> {
        init();

        let input = CustomPojo1 {
            username: "fake_username".to_string(),
            password: "fake_password".to_string(),
            admin: true,
            enabled: false,
        };

        let raw = input.to_bytes()?;

        assert_eq!(
            "aced000573720016636f6d2e6578616d706c652e437573746f6d506f6a6fd3281f2fbb086f580300014c0008757365726e616d657400124c6a6176612f6c616e672f537472696e673b787074000d66616b655f757365726e616d6577020100740015454e43525950545f66616b655f70617373776f726478",
            hex::encode(&raw)
        );

        Ok(())
    }

    struct CustomPojo2 {
        username: String,
        password: String, // transient
        admin: bool,      // transient
        enabled: bool,    // transient
    }

    impl JavaObject for CustomPojo2 {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.CustomPojo", -3231298442776514728)
                    .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
                    .field(Field::builder("username").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for CustomPojo2 {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.username.write_to(w)?;
            Ok(())
        }

        fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.default_write_object(w)?;

            self.admin.write_to(w)?;
            let enc_password = format!("ENCRYPT_{}", self.password);
            enc_password.write_to(w)?;
            self.enabled.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_custom_pojo2() -> Result<()> {
        init();

        let input = CustomPojo2 {
            username: "fake_username".to_string(),
            password: "fake_password".to_string(),
            admin: true,
            enabled: false,
        };

        let raw = input.to_bytes()?;

        assert_eq!(
            "aced000573720016636f6d2e6578616d706c652e437573746f6d506f6a6fd3281f2fbb086f580300014c0008757365726e616d657400124c6a6176612f6c616e672f537472696e673b787074000d66616b655f757365726e616d65770101740015454e43525950545f66616b655f70617373776f726477010078",
            hex::encode(&raw)
        );

        Ok(())
    }

    struct CustomPojo3 {
        username: String,
        password: String, // transient
        admin: bool,      // transient
        enabled: bool,    // transient
    }

    impl JavaObject for CustomPojo3 {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.CustomPojo", -3231298442776514728)
                    .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
                    .field(Field::builder("username").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for CustomPojo3 {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.username.write_to(w)?;
            Ok(())
        }

        fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.default_write_object(w)?;

            let enc_password = format!("ENCRYPT_{}", self.password);
            enc_password.write_to(w)?;

            self.admin.write_to(w)?;
            self.enabled.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_custom_pojo3() -> Result<()> {
        init();

        let input = CustomPojo3 {
            username: "fake_username".to_string(),
            password: "fake_password".to_string(),
            admin: true,
            enabled: false,
        };

        let raw = input.to_bytes()?;

        assert_eq!(
            "aced000573720016636f6d2e6578616d706c652e437573746f6d506f6a6fd3281f2fbb086f580300014c0008757365726e616d657400124c6a6176612f6c616e672f537472696e673b787074000d66616b655f757365726e616d65740015454e43525950545f66616b655f70617373776f72647702010078",
            hex::encode(&raw)
        );

        Ok(())
    }

    struct CharDemo {
        ch: char,
    }

    impl JavaObject for CharDemo {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.CharDemo", -7957532738628518212)
                    .field(Field::builder("ch").char())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for CharDemo {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            w.write(self.ch)
        }
    }

    #[test]
    fn test_char_demo() -> Result<()> {
        init();

        // normal char 'a'
        {
            let o = CharDemo { ch: 'a' };
            let raw = o.to_bytes()?;
            assert_eq!(
                "aced000573720014636f6d2e6578616d706c652e4368617244656d6f91912a2291817ebc020001430002636878700061",
                hex::encode(&raw)
            );
        }

        // CJK char, still a single UTF-16 code unit: same classdesc, only the trailing 2-byte
        // value (last 4 hex chars) differs from the 'a' case above.
        {
            let o = CharDemo { ch: '世' };
            let raw = o.to_bytes()?;
            assert_eq!(
                "aced000573720014636f6d2e6578616d706c652e4368617244656d6f91912a2291817ebc020001430002636878700061"
                    .replace("0061", "4e16"),
                hex::encode(&raw)
            );
        }

        // Emoji is outside BMP, will return an error
        let o = CharDemo { ch: '😀' };
        assert!(o.to_bytes().is_err());

        Ok(())
    }

    #[test]
    fn test_char_write() -> Result<()> {
        init();

        fn check(origin: char, expect: &str) -> Result<()> {
            let mut b: Vec<u8> = vec![];
            let mut w = ObjectWriter::new(&mut b)?;
            // A bare `char` is a raw 2-byte value with no framing of its own, so it must be
            // written outside block-data mode, same as `default_write_object` does for declared
            // fields; a fresh `ObjectWriter` otherwise starts in block-data mode and would
            // silently buffer the bytes instead of emitting them.
            w.set_block_data_mode(false);

            w.write(origin)?;

            let actual = hex::encode(&b);

            info!("{:?}: {}", origin, actual);

            assert_eq!(expect, &actual);
            Ok(())
        }

        // ASCII
        check('a', "aced00050061")?;

        // CJK, still a single UTF-16 code unit
        check('世', "aced00054e16")?;

        // the highest BMP code point
        check('\u{ffff}', "aced0005ffff")?;

        // outside the BMP: cannot be represented by a single Java `char`
        let mut b: Vec<u8> = vec![];
        let mut w = ObjectWriter::new(&mut b)?;
        w.set_block_data_mode(false);
        assert!(w.write('😀').is_err());

        Ok(())
    }
}
