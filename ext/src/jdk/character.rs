use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{Class, Field, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter};
use std::{fmt, io};

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

impl From<Character> for char {
    fn from(val: Character) -> Self {
        val.0
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
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
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
    use crate::testkit;
    use serde::Serialize;
    use serde_java::{JavaSerialize, JavaWriteableExt};

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_character() -> anyhow::Result<()> {
        init();
        let expect = testkit::serialize(Character::class().name(), &[], r#""a""#)?;
        let actual = Character::from('a').to_bytes()?;
        assert_eq!(expect, actual);

        Ok(())
    }

    #[derive(Debug, Serialize, JavaSerialize)]
    #[java(class="com.example.CharacterDemo",serial_version_uid=-1413488344517812718)]
    struct CharacterDemo {
        #[java(
            signature = "Ljava/lang/Character;",
            with = "crate::java::lang::Character"
        )]
        ch: Option<char>,
    }

    #[test]
    fn test_character_in_fields() -> anyhow::Result<()> {
        init();

        for next in [
            // ch='a'
            CharacterDemo { ch: Some('a') },
            // ch=null
            CharacterDemo { ch: None },
        ] {
            info!("{:?}", next);

            let expect = {
                let json = serde_json::to_string(&next)?;
                testkit::serialize(
                    CharacterDemo::class().name(),
                    &["testdata/CharacterDemo.java"],
                    &json,
                )?
            };
            let actual = next.to_bytes()?;
            assert_eq!(expect, actual);
        }

        Ok(())
    }
}
