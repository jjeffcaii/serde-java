mod class;
mod writer;

pub use class::*;
use std::fs::File;
use std::io;
use std::path::Path;
pub(crate) use writer::*;

pub trait JavaObject {
    fn class() -> Class;
}

pub trait JavaSerializable {
    fn fields(&self) -> Vec<FieldValue<'_>>;

    fn write_object<W: io::Write>(&self, w: &mut JavaWriter<W>) -> io::Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}

pub fn write_to<W, O>(w: &mut JavaWriter<W>, obj: &O) -> io::Result<()>
where
    W: io::Write,
    O: JavaSerializable + JavaObject,
{
    let class = O::class();
    if class.flags().contains(ClassFlags::WRITE_METHOD) {
        w.write_object(&class, &[])?;
        // w.custom_block_begin()?;
        obj.write_object(w)?;
        w.end_block_data()?;
        // w.custom_block_end()?;
    } else {
        w.write_object(&class, &obj.fields())?;
    }

    Ok(())
}

/// JavaSerializableExt extends more features.
pub trait JavaSerializableExt {
    /// serialize object to bytes.
    fn to_bytes(&self) -> io::Result<Vec<u8>>;

    /// Serialize object then write to file.
    fn to_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>;
}

impl<T: JavaSerializable + JavaObject> JavaSerializableExt for T {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut b = Vec::<u8>::new();
        let mut w = JavaWriter::new(&mut b)?;
        write_to(&mut w, self)?;
        Ok(b)
    }

    fn to_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let mut f = File::create(path)?;
        let mut w = JavaWriter::new(&mut f)?;
        write_to(&mut w, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use once_cell::sync::Lazy;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

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
        fn fields(&self) -> Vec<FieldValue<'_>> {
            vec![FieldValue::Int(self.i), FieldValue::String(&self.message)]
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
        fn fields(&self) -> Vec<FieldValue<'_>> {
            vec![FieldValue::String(&self.city)]
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
        fn fields(&self) -> Vec<FieldValue<'_>> {
            vec![
                FieldValue::Int(self.id),
                FieldValue::Object(Address::class(), &self.address),
            ]
        }
    }

    #[test]
    fn test_string() -> io::Result<()> {
        init();

        let mut b: Vec<u8> = vec![];
        let mut w = JavaWriter::new(&mut b)?;

        let origin = "Hello 世界!";

        w.write_string(origin)?;

        let actual = hex::encode(&b);

        info!("{}: {}", origin, &actual);

        let expect_java = "aced000574000d48656c6c6f20e4b896e7958c21";
        assert_eq!(expect_java, &actual);

        Ok(())
    }

    #[test]
    fn test_serialize() -> io::Result<()> {
        init();

        let mut b: Vec<u8> = vec![];
        let mut w = JavaWriter::new(&mut b)?;

        let demo = Demo {
            i: 42,
            message: "helloWorld".to_string(),
        };

        write_to(&mut w, &demo)?;

        info!("java serialize: {}", hex::encode(&b));

        assert_eq!(
            "aced000573720010636f6d2e6578616d706c652e44656d6f477d87c81fbf509e020002490001694c00076d6573736167657400124c6a6176612f6c616e672f537472696e673b78700000002a74000a68656c6c6f576f726c64",
            hex::encode(&b)
        );

        Ok(())
    }

    #[test]
    fn test_serialize_nested() -> io::Result<()> {
        init();

        let mut b: Vec<u8> = vec![];
        let mut w = JavaWriter::new(&mut b)?;

        let order = Order {
            id: 7,
            address: Address {
                city: "NY".to_string(),
            },
        };

        write_to(&mut w, &order)?;

        info!("java serialize nested: {}", hex::encode(&b));

        assert_eq!(
            "aced000573720011636f6d2e6578616d706c652e4f72646572267b25a101681cb402000249000269644c0007616464726573737400154c636f6d2f6578616d706c652f416464726573733b78700000000773720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200014c0004636974797400124c6a6176612f6c616e672f537472696e673b78707400024e59",
            hex::encode(&b)
        );

        Ok(())
    }

    // Writing into an in-memory buffer:
    // let mut buf = Vec::new();
    // let mut w = JavaWriter::new(&mut buf)?;
}
