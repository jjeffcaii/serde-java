//! The same `com.example.User` stream as `example.rs`, written with
//! `#[derive(JavaSerialize)]` instead of hand-written impls.
//!
//! Both examples target `example.java` and produce byte-identical output. Note that the fields
//! are declared here in the Java class's declaration order — the derive sorts them the way
//! `ObjectStreamClass` does (primitives first, then objects/arrays, each group alphabetical),
//! which is the ordering `example.rs` has to spell out by hand.

use serde_java::*;
use std::io;

#[derive(Debug, JavaSerialize)]
#[java(class = "com.example.User", serial_version_uid = 4956385333250593913)]
struct User {
    id: i64,
    name: String,
    age: i32,
    addresses: Vec<Address>,
    ext1: ExtInfo,
    ext2: ExtInfo,
}

#[derive(Debug, JavaSerialize)]
#[java(class = "com.example.Address", serial_version_uid = -4433675896693646393)]
struct Address {
    country: String,
    city: String,
    street: String,
}

#[derive(Debug, Clone, PartialEq, JavaSerialize)]
#[java(
    class = "com.example.ExtInfo",
    serial_version_uid = 8520976260072537200
)]
struct ExtInfo {
    id: i32,
    key: String,
    value: String,
}

impl ExtInfo {
    fn new<K, V>(id: i32, key: K, value: V) -> ExtInfo
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            id,
            key: key.into(),
            value: value.into(),
        }
    }
}

impl Address {
    fn new<C, T, R>(country: C, city: T, street: R) -> Self
    where
        C: Into<String>,
        T: Into<String>,
        R: Into<String>,
    {
        Self {
            country: country.into(),
            city: city.into(),
            street: street.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, JavaSerialize)]
#[java(class = "com.example.Bang", serial_version_uid = 1)]
struct Bang {
    id: i64,
    names: Vec<String>,
}

impl JavaFieldsDeserializable for Bang {
    type Fields = (i64, Vec<String>);

    fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read,
    {
        Ok((r.read()?, r.read_all()?))
    }
}

impl JavaDeserializable for Bang {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read,
    {
        let (id, names) = Self::read_fields(r)?;
        Ok(Self { id, names })
    }
}

fn main() -> Result<()> {
    let bang = Bang {
        id: 1,
        names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };
    let raw = bang.to_bytes()?;
    let parsed = Bang::from_bytes(&raw)?;
    assert_eq!(parsed, bang);

    let user = User {
        id: 123,
        name: "Jack".to_string(),
        age: 18,
        addresses: vec![
            Address::new("China", "Shanghai", "Dongfang Rd"),
            Address::new("China", "Beijing", "Changan Rd"),
        ],
        ext1: ExtInfo::new(777, "k777", "v777"),
        ext2: ExtInfo::new(888, "k888", "v888"),
    };

    println!("{:?}: {}", user, hex::encode(&user.to_bytes()?));

    // TODO: implement deserialize with derive

    Ok(())
}
