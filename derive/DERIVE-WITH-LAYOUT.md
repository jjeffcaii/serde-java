# Derive with custom `serde_java::Layout`

## Goals

1. add an attribute `with`, eg: `#[java(signature="Ljava/util/List;",with="serde_java_ext::ArrayList")]`
2. `with` must implements `Layout` trait
3. if `with="some_module::SomeLayout"` is specified, you should serialize field with
   `some_module::SomeLayout::layout(&origin_field).write_to(w)`

## APIs Design

Here are some Demo APIs, the `ArrayList` has been implemented in `ext`, see from `list.rs`:

```rust

#[derive(Debug, JavaSerialize)]
#[java(
    class = "com.example.ListDemo",
    serial_version_uid = 3153513349080412905
)]
struct ListDemo {
    id: i32,
    #[java(signature = "Ljava/util/List;", with = "serde_java_ext::ArrayList")]
    names: Vec<String>,
}
```

- expand macros:

```rust

struct ListDemo {
    id: i32,
    names: Vec<String>,
}

impl JavaObject for ListDemo {
    fn class() -> Class {
        static CLASS: Lazy<Class> = Lazy::new(|| {
            Class::builder("com.example.ListDemo", 3153513349080412905)
                .field(Field::builder("id").int())
                .field(Field::builder("names").object("Ljava/util/List;"))
                .build()
        });
        Clone::clone(&CLASS)
    }
}

impl JavaSerializable for ListDemo {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn Write>) -> io::Result<()> {
        self.id.write_to(w)?;
        serde_java_ext::ArrayList::layout(&self.names).write_to(w)?;
        Ok(())
    }
}

```
