use once_cell::sync::Lazy;
use serde_java::*;
use std::io;

#[derive(Debug)]
struct User {
    id: i64,
    name: String,
    age: i32,
    addresses: Vec<Address>,
    ext1: ExtInfo,
    ext2: ExtInfo,
}

#[derive(Debug)]
struct Address {
    country: String,
    city: String,
    street: String,
}

#[derive(Debug, Clone, PartialEq)]
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

impl JavaObject for ExtInfo {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("com.example.ExtInfo", 8520976260072537200)
                .field(Field::builder("id").int())
                .field(Field::builder("key").string())
                .field(Field::builder("value").string())
                .build()
        });
        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for ExtInfo {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.id.write_to(w)?;
        self.key.write_to(w)?;
        self.value.write_to(w)?;
        Ok(())
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

impl JavaObject for Address {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("com.example.Address", -4433675896693646393)
                .field(Field::builder("city").string())
                .field(Field::builder("country").string())
                .field(Field::builder("street").string())
                .build()
        });
        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for Address {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.city.write_to(w)?;
        self.country.write_to(w)?;
        self.street.write_to(w)?;
        Ok(())
    }
}

impl JavaObject for User {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("com.example.User", 4956385333250593913)
                .field(Field::builder("age").int())
                .field(Field::builder("id").long())
                .field(Field::builder("addresses").array(Address::class().signature()))
                .field(Field::builder("ext1").object("Lcom/example/ExtInfo;"))
                .field(Field::builder("ext2").object("Lcom/example/ExtInfo;"))
                .field(Field::builder("name").string())
                .build()
        });

        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for User {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.age.write_to(w)?;
        self.id.write_to(w)?;
        self.addresses.write_to(w)?;
        self.ext1.write_to(w)?;
        self.ext2.write_to(w)?;
        self.name.write_to(w)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
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

    let b = user.to_bytes()?;

    println!("{:?}: {}", user, hex::encode(&b));

    assert_eq!(
        "aced000573720010636f6d2e6578616d706c652e5573657244c89e33565b94790200064900036167654a000269645b00096164647265737365737400165b4c636f6d2f6578616d706c652f416464726573733b4c0004657874317400154c636f6d2f6578616d706c652f457874496e666f3b4c00046578743271007e00024c00046e616d657400124c6a6176612f6c616e672f537472696e673b787000000012000000000000007b757200165b4c636f6d2e6578616d706c652e416464726573733b68c376a74c450c5f02000078700000000273720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200034c00046369747971007e00034c0007636f756e74727971007e00034c000673747265657471007e000378707400085368616e676861697400054368696e6174000b446f6e6766616e672052647371007e00077400074265696a696e6771007e000a74000a4368616e67616e20526473720013636f6d2e6578616d706c652e457874496e666f764096c33128fc7002000349000269644c00036b657971007e00034c000576616c756571007e00037870000003097400046b373737740004763737377371007e000f000003787400046b383838740004763838387400044a61636b",
        hex::encode(&b),
    );

    Ok(())
}
