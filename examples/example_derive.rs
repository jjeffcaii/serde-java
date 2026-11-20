//! The same `com.example.User` stream as `example.rs`, written with
//! `#[derive(JavaSerialize, JavaDeserialize)]` instead of hand-written impls.
//! Also demonstrates round trips for nested objects, arrays, and nullable fields.
//!
//! Both examples target `example.java` and produce byte-identical output. Note that the fields
//! are declared here in the Java class's declaration order — the derive sorts them the way
//! `ObjectStreamClass` does (primitives first, then objects/arrays, each group alphabetical),
//! which is the ordering `example.rs` has to spell out by hand.

use serde_java::*;

#[derive(Debug, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "com.example.User", serial_version_uid = 4956385333250593913)]
struct User {
    id: i64,
    name: String,
    age: i32,
    addresses: Vec<Address>,
    ext1: ExtInfo,
    ext2: ExtInfo,
}

#[derive(Debug, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "com.example.Address", serial_version_uid = -4433675896693646393)]
struct Address {
    country: String,
    city: String,
    street: String,
}

#[derive(Debug, Clone, PartialEq, JavaSerialize, JavaDeserialize)]
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

#[derive(Debug, Clone, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "com.example.Bang", serial_version_uid = 1)]
struct Bang {
    id: i64,
    names: Vec<String>,
}

// `rename` determines the Java field name and wire order. `skip` fields are
// omitted from the stream and initialized with Default during deserialization.
#[derive(Debug, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "com.example.Profile", serial_version_uid = 1)]
struct Profile {
    #[java(rename = "displayName")]
    display_name: String,
    nickname: Option<String>,
    address: Option<Address>,
    scores: Vec<i32>,
    #[java(skip)]
    local_cache: String,
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

    // Nested objects and object arrays are deserialized recursively.
    let raw = user.to_bytes()?;
    println!("{:?}: {}", user, hex::encode(&raw));
    let parsed = User::from_bytes(&raw)?;
    assert_eq!(parsed, user);

    let mut profile = Profile {
        display_name: "Jack".to_string(),
        nickname: Some("JJ".to_string()),
        address: Some(Address::new("China", "Shanghai", "Dongfang Rd")),
        scores: vec![90, 85, 100],
        local_cache: "only stored locally".to_string(),
    };
    let raw = profile.to_bytes()?;

    println!("{:?}: {}", &profile, hex::encode(&raw));

    // aced000573720013636f6d2e6578616d706c652e50726f66696c6500000000000000010200044c0007616464726573737400154c636f6d2f6578616d706c652f416464726573733b4c000b646973706c61794e616d657400124c6a6176612f6c616e672f537472696e673b4c00086e69636b6e616d6571007e00025b000673636f7265737400025b49787073720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200034c00046369747971007e00024c0007636f756e74727971007e00024c000673747265657471007e000278707400085368616e676861697400054368696e6174000b446f6e6766616e672052647400044a61636b7400024a4a757200025b494dba602676eab2a50200007870000000030000005a0000005500000064
    // aced000573720013636f6d2e6578616d706c652e50726f66696c6500000000000000010200044c0007616464726573737400154c636f6d2f6578616d706c652f416464726573733b4c000b646973706c61794e616d657400124c6a6176612f6c616e672f537472696e673b4c00086e69636b6e616d6571007e00025b000673636f7265737400025b49787073720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200034c00046369747971007e00024c0007636f756e74727971007e00024c000673747265657471007e000278707400085368616e676861697400054368696e6174000b446f6e6766616e672052647400044a61636b7400024a4a757200025b494dba602676eab2a50200007870000000030000005a0000005500000064

    let parsed = Profile::from_bytes(&raw)?;
    assert!(parsed.local_cache.is_empty());
    profile.local_cache.clear();
    assert_eq!(parsed, profile);

    // Nullable strings and objects also accept Java null.
    profile.nickname = None;
    profile.address = None;
    let parsed = Profile::from_bytes(&profile.to_bytes()?)?;
    assert_eq!(parsed, profile);

    Ok(())
}
