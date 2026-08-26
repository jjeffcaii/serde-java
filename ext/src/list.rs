use super::misc::write_boxed;
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, ClassFlags, Field, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter,
};
use std::io::Write;
use std::{fmt, io};

static CLASS_OF_LINKED_LIST: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.util.LinkedList", 876323262645176354)
        .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
        .build()
});

pub struct LinkedList<'a, T>(pub &'a [T]);

impl<'a, T> fmt::Debug for LinkedList<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(CLASS_OF_LINKED_LIST.name())?;
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<'a, T> fmt::Display for LinkedList<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, next) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            fmt::Display::fmt(next, f)?;
        }
        f.write_str("]")
    }
}

impl<'a, T> From<&'a [T]> for LinkedList<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self(value)
    }
}

impl<'a, T> JavaObject for LinkedList<'a, T> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_LINKED_LIST)
    }
}

impl<'a, T> JavaSerializable for LinkedList<'a, T>
where
    T: 'static + JavaWriteable,
{
    fn write_fields(&self, _w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)?;

        (self.0.len() as i32).write_to(w)?;

        for next in self.0 {
            write_boxed(w, next)?;
        }

        Ok(())
    }
}

impl<'a, T> Layout<'a> for LinkedList<'a, T> {
    type Input = Vec<T>;
    type Output = LinkedList<'a, T>;

    fn layout(input: &'a Self::Input) -> Self::Output {
        LinkedList(&input)
    }
}

static CLASS_OF_ARRAY_LIST: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.util.ArrayList", 8683452581122892189)
        .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
        .field(Field::builder("size").int())
        .build()
});

pub struct ArrayList<'a, T>(pub &'a [T]);

impl<'a, T> fmt::Debug for ArrayList<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(CLASS_OF_ARRAY_LIST.name())?;
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<'a, T> fmt::Display for ArrayList<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, next) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            fmt::Display::fmt(next, f)?;
        }
        f.write_str("]")
    }
}

impl<'a, T> From<&'a Vec<T>> for ArrayList<'a, T> {
    fn from(value: &'a Vec<T>) -> Self {
        From::from(value.as_slice())
    }
}

impl<'a, T> From<&'a [T]> for ArrayList<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self(value)
    }
}

impl<'a, T> JavaObject for ArrayList<'a, T> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_ARRAY_LIST)
    }
}

impl<'a, T> JavaSerializable for ArrayList<'a, T>
where
    T: 'static + JavaWriteable,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        (self.0.len() as i32).write_to(w)?;
        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)?;

        (self.0.len() as i32).write_to(w)?;

        for next in self.0 {
            write_boxed(w, next)?;
        }

        Ok(())
    }
}

impl<'a, T> Layout<'a> for ArrayList<'a, T> {
    type Input = Vec<T>;
    type Output = ArrayList<'a, T>;

    fn layout(input: &'a Self::Input) -> Self::Output {
        ArrayList(input)
    }
}

static CLASS_OF_EMPTY_LIST: Lazy<Class> =
    Lazy::new(|| Class::builder("java.util.Collections$EmptyList", 8842843931221139166).build());

#[derive(Default)]
pub struct EmptyList;

impl JavaObject for EmptyList {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_EMPTY_LIST)
    }
}

impl JavaSerializable for EmptyList {
    fn write_fields(&self, _w: &mut ObjectWriter<&mut dyn Write>) -> io::Result<()> {
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
    fn test_empty_list() -> io::Result<()> {
        init();

        let l = EmptyList::default();
        let raw = l.to_bytes()?;

        assert_eq!(
            "aced00057372001f6a6176612e7574696c2e436f6c6c656374696f6e7324456d7074794c6973747ab817b43ca79ede0200007870",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[test]
    fn test_linked_list() -> io::Result<()> {
        init();

        let origin = vec!["foo".to_string(), "bar".to_string(), "qux".to_string()];
        let l = LinkedList::from(&origin[..]);

        info!("debug: {:?}", l);
        info!("display: {}", l);

        let raw = l.to_bytes()?;

        assert_eq!(
            "aced0005737200146a6176612e7574696c2e4c696e6b65644c6973740c29535d4a6088220300007870770400000003740003666f6f74000362617274000371757878",
            hex::encode(raw)
        );

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(class="com.example.LinkedListDemo",serial_version_uid=-2897624825673425478)]
    struct LinkedListDemo {
        id: i32,
        #[java(signature = "Ljava/util/List;", with = "crate::LinkedList")]
        names: Vec<String>,
        #[java(signature = "Ljava/util/List;", with = "crate::LinkedList")]
        scores: Vec<f32>,
    }

    #[test]
    fn test_linked_list_in_fields() -> io::Result<()> {
        init();

        let v = LinkedListDemo {
            id: -1,
            names: vec!["foo".into(), "bar".into(), "qux".into()],
            scores: vec![1.1, 2.2, 3.3],
        };

        info!("{:?}", &v);

        let raw = v.to_bytes()?;

        assert_eq!(
            "aced00057372001a636f6d2e6578616d706c652e4c696e6b65644c69737444656d6fd7c99192c561ddba02000349000269644c00056e616d65737400104c6a6176612f7574696c2f4c6973743b4c000673636f72657371007e00017870ffffffff737200146a6176612e7574696c2e4c696e6b65644c6973740c29535d4a6088220300007870770400000003740003666f6f740003626172740003717578787371007e00037704000000037372000f6a6176612e6c616e672e466c6f6174daedc9a2db3cf0ec02000146000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b02000078703f8ccccd7371007e0009400ccccd7371007e00094053333378",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[test]
    fn test_array_list() -> io::Result<()> {
        init();

        let origin = vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
            "qux".to_string(),
            "gob".to_string(),
        ];
        let l = ArrayList::from(&origin[..]);

        info!("debug: {:?}", l);
        info!("display: {}", l);

        let raw = l.to_bytes()?;

        assert_eq!(
            "aced0005737200136a6176612e7574696c2e41727261794c6973747881d21d99c7619d03000149000473697a65787000000005770400000005740003666f6f74000362617274000362617a740003717578740003676f6278",
            hex::encode(raw)
        );

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(
        class = "com.example.ArrayListDemo",
        serial_version_uid = 3153513349080412905
    )]
    struct ArrayListDemo {
        id: i32,
        #[java(signature = "Ljava/util/List;", with = "crate::ArrayList")]
        names: Option<Vec<String>>,
        #[java(signature = "Ljava/util/List;", with = "crate::ArrayList")]
        scores: Option<Vec<i32>>,
    }

    #[test]
    fn test_array_list_in_fields() -> io::Result<()> {
        init();

        // check null names
        {
            let l = ArrayListDemo {
                id: 0xffff,
                names: None,
                scores: None,
            };

            info!("{:?}", &l);

            let raw = l.to_bytes()?;
            assert_eq!(
                "aced000573720019636f6d2e6578616d706c652e41727261794c69737444656d6f2bc387aed663f2e902000349000269644c00056e616d65737400104c6a6176612f7574696c2f4c6973743b4c000673636f72657371007e000178700000ffff7070",
                hex::encode(&raw)
            );
        }

        // check non-null names
        {
            let l = ArrayListDemo {
                id: 0xffff,
                names: Some(vec!["foo".into(), "bar".into(), "qux".into()]),
                scores: Some(vec![1, 2, 3]),
            };

            info!("{:?}", &l);

            let raw = l.to_bytes()?;
            assert_eq!(
                "aced000573720019636f6d2e6578616d706c652e41727261794c69737444656d6f2bc387aed663f2e902000349000269644c00056e616d65737400104c6a6176612f7574696c2f4c6973743b4c000673636f72657371007e000178700000ffff737200136a6176612e7574696c2e41727261794c6973747881d21d99c7619d03000149000473697a65787000000003770400000003740003666f6f740003626172740003717578787371007e000300000003770400000003737200116a6176612e6c616e672e496e746567657212e2a0a4f781873802000149000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b0200007870000000017371007e0009000000027371007e00090000000378",
                hex::encode(&raw)
            );
        }

        Ok(())
    }
}
