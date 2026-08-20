mod class;
mod extends;
mod object;
mod serializable;
mod writer;

pub use class::{Class, ClassBuilder, ClassFlags, Field, FieldBuilder, FieldKind};
pub use extends::{Extends, ExtendsLayout};
pub use object::{Object, ObjectBuilder};
pub use serializable::{JavaObject, JavaSerializable, JavaWriteable};
pub use writer::JavaWriter;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::proto::object::Object;
    use anyhow::Result;
    use once_cell::sync::Lazy;
    use std::io;
    use std::io::Write;

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
        fn write_object(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
            w.write_int(self.i)?;
            w.write_string(&self.message)?;
            Ok(())
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
        fn write_object(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
            w.write_string(&self.city)?;
            Ok(())
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
        fn write_object(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
            w.write_int(self.id)?;

            let obj = Object::<Address, ()>::builder(Address::class())
                .this(&self.address)
                .build();
            obj.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_string() -> Result<()> {
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
    fn test_serialize() -> Result<()> {
        init();

        let mut b: Vec<u8> = vec![];
        let mut w = JavaWriter::new(&mut b)?;

        let demo = Demo {
            i: 42,
            message: "helloWorld".to_string(),
        };

        let obj = Object::<Demo, ()>::builder(Demo::class())
            .this(&demo)
            .build();

        obj.write_to(&mut w)?;

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

        let obj = Object::<Order, ()>::builder(Order::class())
            .this(&order)
            .build();

        obj.write_to(&mut w)?;

        info!("java serialize nested: {}", hex::encode(&b));

        assert_eq!(
            "aced000573720011636f6d2e6578616d706c652e4f72646572267b25a101681cb402000249000269644c0007616464726573737400154c636f6d2f6578616d706c652f416464726573733b78700000000773720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200014c0004636974797400124c6a6176612f6c616e672f537472696e673b78707400024e59",
            hex::encode(&b)
        );

        Ok(())
    }

    struct CustomPojo {
        username: String,
        password: String,
    }

    impl JavaObject for CustomPojo {
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

    impl JavaSerializable for CustomPojo {
        fn write_object(&self, w: &mut JavaWriter<&mut dyn Write>) -> io::Result<()> {
            self.username.write_to(w)?;
            let enc_password = format!("ENCRYPT_{}", &self.password);
            enc_password.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_custom_pojo() -> Result<()> {
        init();

        let cp = CustomPojo {
            username: "fake_username".to_string(),
            password: "fake_password".to_string(),
        };

        let b = cp.to_bytes()?;

        let actual = hex::encode(&b);
        let expect = "aced000573720016636f6d2e6578616d706c652e437573746f6d506f6a6fd3281f2fbb086f580300014c0008757365726e616d657400124c6a6176612f6c616e672f537472696e673b787074000d66616b655f757365726e616d65740015454e43525950545f66616b655f70617373776f726478";

        assert_eq!(expect, &actual);

        Ok(())
    }
}
