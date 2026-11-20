use once_cell::sync::Lazy;
use serde_java::*;
use std::io;

#[derive(Debug, Eq, PartialEq)]
struct User {
    id: i64,
    name: String,
    age: i32,
    addresses: Vec<Address>,
    ext1: ExtInfo,
    ext2: ExtInfo,
}

#[derive(Debug, Eq, PartialEq)]
struct Address {
    country: String,
    city: String,
    street: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl JavaFieldsDeserializable for ExtInfo {
    type Fields = (i32, String, String);

    fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read,
    {
        let id: i32 = r.read()?;
        let key: String = r.read()?;
        let value: String = r.read()?;
        Ok((id, key, value))
    }
}

impl JavaDeserializable for ExtInfo {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read,
    {
        let (id, key, value) = Self::read_fields(r)?;
        Ok(Self { id, key, value })
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

impl JavaFieldsDeserializable for Address {
    type Fields = (String, String, String);

    fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read,
    {
        let city = r.read()?;
        let country = r.read()?;
        let street = r.read()?;
        Ok((city, country, street))
    }
}

impl JavaDeserializable for Address {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read,
    {
        let (city, country, street) = Self::read_fields(r)?;
        Ok(Self {
            city,
            country,
            street,
        })
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

impl JavaFieldsDeserializable for User {
    type Fields = (i32, i64, Vec<Address>, ExtInfo, ExtInfo, String);

    fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read,
    {
        let age: i32 = r.read()?;
        let id: i64 = r.read()?;
        let addresses: Vec<Address> = r.read_all()?;
        let ext1: ExtInfo = r.read()?;
        let ext2: ExtInfo = r.read()?;
        let name: String = r.read()?;

        Ok((age, id, addresses, ext1, ext2, name))
    }
}

impl JavaDeserializable for User {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read,
    {
        let (age, id, addresses, ext1, ext2, name) = Self::read_fields(r)?;

        Ok(Self {
            age,
            id,
            addresses,
            ext1,
            ext2,
            name,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let _x = minus(1, 10);
    pretty_env_logger::try_init().ok();
    let origin = User {
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

    let raw = origin.to_bytes()?;

    println!("{:?}: {}", origin, hex::encode(&raw));

    assert_eq!(
        "aced000573720010636f6d2e6578616d706c652e5573657244c89e33565b94790200064900036167654a000269645b00096164647265737365737400165b4c636f6d2f6578616d706c652f416464726573733b4c0004657874317400154c636f6d2f6578616d706c652f457874496e666f3b4c00046578743271007e00024c00046e616d657400124c6a6176612f6c616e672f537472696e673b787000000012000000000000007b757200165b4c636f6d2e6578616d706c652e416464726573733b68c376a74c450c5f02000078700000000273720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200034c00046369747971007e00034c0007636f756e74727971007e00034c000673747265657471007e000378707400085368616e676861697400054368696e6174000b446f6e6766616e672052647371007e00077400074265696a696e6771007e000a74000a4368616e67616e20526473720013636f6d2e6578616d706c652e457874496e666f764096c33128fc7002000349000269644c00036b657971007e00034c000576616c756571007e00037870000003097400046b373737740004763737377371007e000f000003787400046b383838740004763838387400044a61636b",
        hex::encode(&raw),
    );

    let parsed = User::from_bytes(&raw)?;

    assert_eq!(origin, parsed);

    Ok(())
}

fn minus(x: u8, y: u8) -> u8 {
    x - y
}
