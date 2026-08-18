use once_cell::sync::Lazy;
use serde_java::*;

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
    fn fields(&self) -> Vec<FieldValue<'_>> {
        vec![
            FieldValue::from(self.id),
            FieldValue::from(&self.key),
            FieldValue::from(&self.value),
        ]
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
    fn fields(&self) -> Vec<FieldValue<'_>> {
        vec![
            FieldValue::String(&self.city),
            FieldValue::String(&self.country),
            FieldValue::String(&self.street),
        ]
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
    fn fields(&self) -> Vec<FieldValue<'_>> {
        vec![
            FieldValue::from(self.age),
            FieldValue::from(self.id),
            FieldValue::Array(
                Class::class_of_array(Address::class(), 7549007861314292831),
                self.addresses
                    .iter()
                    .map(|a| (Address::class(), a as &dyn JavaSerializable))
                    .collect(),
            ),
            FieldValue::Object(ExtInfo::class(), &self.ext1),
            FieldValue::Object(ExtInfo::class(), &self.ext2),
            FieldValue::from(&self.name),
        ]
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

    println!("{:?}: {}", &user, hex::encode(&user.to_bytes()?));

    Ok(())
}
