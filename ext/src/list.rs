use serde_java::__private::Lazy;
use serde_java::{
    Class, ClassFlags, Field, JavaObject, JavaSerializable, JavaWriteable, ObjectWriter,
};
use std::fmt::Formatter;
use std::{fmt, io};

static CLASS_OF_ARRAY_LIST: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.util.ArrayList", 8683452581122892189)
        .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
        .field(Field::builder("size").int())
        .build()
});

pub struct ArrayList<T>(pub Vec<T>);

impl<T> fmt::Debug for ArrayList<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<T> From<Vec<T>> for ArrayList<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> JavaObject for ArrayList<T> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_ARRAY_LIST)
    }
}

impl<T> JavaSerializable for ArrayList<T>
where
    T: JavaWriteable,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        (self.0.len() as i32).write_to(w)?;
        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)?;

        (self.0.len() as i32).write_to(w)?;

        for next in &self.0 {
            next.write_to(w)?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_java::JavaSerialize;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_array_list() -> io::Result<()> {
        init();

        let l = ArrayList::from(vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
            "qux".to_string(),
            "gob".to_string(),
        ]);

        let b = l.to_bytes()?;

        l.to_file("/Users/jeffsky/Desktop/rust_arraylist.dat")?;

        assert_eq!(
            "aced0005737200136a6176612e7574696c2e41727261794c6973747881d21d99c7619d03000149000473697a65787000000005770400000005740003666f6f74000362617274000362617a740003717578740003676f6278",
            hex::encode(b)
        );

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(
        class = "com.example.ListDemo",
        serial_version_uid = 3153513349080412905
    )]
    struct ListDemo {
        id: i32,

        #[java(signature = "Ljava/util/List;")]
        names: ArrayList<String>,
    }

    #[test]
    fn test_list_demo() -> io::Result<()> {
        init();

        let l = ListDemo {
            id: 0xffff,
            names: ArrayList(vec![
                "foo".to_string(),
                "bar".to_string(),
                "qux".to_string(),
            ]),
        };

        let b = l.to_bytes()?;

        l.to_file("/Users/jeffsky/Desktop/rust.dat")?;

        assert_eq!(
            "aced000573720014636f6d2e6578616d706c652e4c69737444656d6f2bc387aed663f2e902000249000269644c00056e616d65737400104c6a6176612f7574696c2f4c6973743b78700000ffff737200136a6176612e7574696c2e41727261794c6973747881d21d99c7619d03000149000473697a65787000000003770400000003740003666f6f74000362617274000371757878",
            hex::encode(&b)
        );

        Ok(())
    }
}
