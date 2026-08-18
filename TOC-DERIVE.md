# Derive for Java

设计目标: 使用过程宏, 对目标`struct`自动实现`JavaObject`以及`JavaSerializable`

可以使用的基础第三方库:
- syn
- quote
- proc-macro2


## Example

以下过程宏定义:

```rust

#[derive(JavaSerialize, Debug, PartialEq)]
#[java(class = "com.example.Address", serial_version_uid = 5678)]
struct Address {
    country: String,
    city: String,
    street: String,
}

#[derive(JavaSerialize, Debug, PartialEq)]
#[java(class = "com.example.User", serial_version_uid = 1234)]
struct User {
    id: i32,
    name: String,
    #[java(rename = "address")]
    address_alias: Address,
}


```

将会自动实现`JavaSerialize`和`JavaObject`

```rust

impl JavaObject for Address {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("com.example.Address", 5678)
                .field(Field::builder("city").string())
                .field(Field::builder("country").string())
                .field(Field::builder("street").string())
                .build()
        });

        Clone::clone(&CLASS)
    }
}

impl JavaObject for User {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("com.example.User", 1234)
                .field(Field::builder("id").long())
                .field(Field::builder("address").object(Address::class().signature()))
                .field(Field::builder("name").string())
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

impl JavaSerializable for User {
    fn fields(&self) -> Vec<FieldValue<'_>> {
        vec![
            FieldValue::from(self.id),
            FieldValue::Object(Address::class(), &self.address),
            FieldValue::from(&self.name),
        ]
    }
}


```