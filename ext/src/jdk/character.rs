use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{Class, Field, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter};
use std::fmt;
use std::io::Write;

/// Reference of java.lang.Character
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Character(pub char);

impl fmt::Debug for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Into<char> for Character {
    fn into(self) -> char {
        self.0
    }
}

impl From<char> for Character {
    fn from(value: char) -> Self {
        Self(value)
    }
}

impl JavaObject for Character {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("java.lang.Character", 3786198910865385080)
                .field(Field::builder("value").char())
                .build()
        });
        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for Character {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn Write>) -> std::io::Result<()> {
        self.0.write_to(w)
    }
}

impl<'a> Layout<'a> for Character {
    type Input = char;
    type Output = Character;

    fn layout(input: &'a Self::Input) -> Self::Output {
        Character(*input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_java::JavaSerialize;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_character() -> io::Result<()> {
        init();
        //aced0005737200136a6176612e6c616e672e436861726163746572348b47d96b1a267802000143000576616c756578700061

        let raw = Character::from('a').to_bytes()?;
        assert_eq!(
            "aced0005737200136a6176612e6c616e672e436861726163746572348b47d96b1a267802000143000576616c756578700061",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(class="com.example.CharacterDemo",serial_version_uid=-1413488344517812718)]
    struct CharacterDemo {
        #[java(
            signature = "Ljava/lang/Character;",
            with = "crate::java::lang::Character"
        )]
        ch: Option<char>,
    }

    #[test]
    fn test_character_in_fields() -> io::Result<()> {
        init();

        {
            let d = CharacterDemo { ch: Some('a') };
            info!("{:?}", &d);
            let raw = d.to_bytes()?;
            assert_eq!(
                "aced000573720019636f6d2e6578616d706c652e43686172616374657244656d6fec6247d6f2dc5e120200014c000263687400154c6a6176612f6c616e672f4368617261637465723b7870737200136a6176612e6c616e672e436861726163746572348b47d96b1a267802000143000576616c756578700061",
                hex::encode(&raw)
            );
        }

        // check null
        {
            let d = CharacterDemo { ch: None };
            info!("{:?}", &d);
            let raw = d.to_bytes()?;
            assert_eq!(
                "aced000573720019636f6d2e6578616d706c652e43686172616374657244656d6fec6247d6f2dc5e120200014c000263687400154c6a6176612f6c616e672f4368617261637465723b787070",
                hex::encode(&raw)
            );
        }

        Ok(())
    }
}
