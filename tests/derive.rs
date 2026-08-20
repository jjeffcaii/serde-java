#![cfg(feature = "derive")]

use serde_java::*;
use std::io;

// ---- Reuses the real-ObjectOutputStream fixture from src/proto/mod.rs's tests ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Demo", serial_version_uid = 5151422842377556126)]
struct Demo {
    i: i32,
    message: String,
}

#[test]
fn test_derive_matches_demo_fixture() {
    let demo = Demo {
        i: 42,
        message: "helloWorld".to_string(),
    };

    assert_eq!(
        "aced000573720010636f6d2e6578616d706c652e44656d6f477d87c81fbf509e020002490001694c00076d6573736167657400124c6a6176612f6c616e672f537472696e673b78700000002a74000a68656c6c6f576f726c64",
        hex::encode(demo.to_bytes().unwrap())
    );
}

// ---- Nested objects, again against a known fixture ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Address", serial_version_uid = -4433675896693646393)]
struct Address {
    city: String,
}

#[derive(JavaSerialize)]
#[java(class = "com.example.Order", serial_version_uid = 2772851369020234932)]
struct Order {
    id: i32,
    address: Address,
}

#[test]
fn test_derive_matches_order_fixture() {
    let order = Order {
        id: 7,
        address: Address {
            city: "NY".to_string(),
        },
    };

    assert_eq!(
        "aced000573720011636f6d2e6578616d706c652e4f72646572267b25a101681cb402000249000269644c0007616464726573737400154c636f6d2f6578616d706c652f416464726573733b78700000000773720013636f6d2e6578616d706c652e41646472657373c2786b43385d1bc70200014c0004636974797400124c6a6176612f6c616e672f537472696e673b78707400024e59",
        hex::encode(order.to_bytes().unwrap())
    );
}

// ---- Ordering: declaration order is deliberately shuffled; assert the order in class() ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Shuffled", serial_version_uid = 1)]
struct Shuffled {
    zebra: String,
    beta: i32,
    alpha: Address,
    yak: i64,
}

#[test]
fn test_derive_field_order() {
    let class = Shuffled::class();
    let names: Vec<&str> = class.fields().iter().map(Field::name).collect();
    // Primitives first (beta:I, yak:J by name), then objects (alpha, zebra by name)
    assert_eq!(vec!["beta", "yak", "alpha", "zebra"], names);
}

#[test]
fn test_derive_all_scalar_types() {
    #[derive(JavaSerialize)]
    #[java(class = "com.example.Scalars", serial_version_uid = 2)]
    struct Scalars {
        a_bool: bool,
        b_i8: i8,
        c_u8: u8,
        d_u16: u16,
        e_i16: i16,
        f_i32: i32,
        g_i64: i64,
        h_f32: f32,
        i_f64: f64,
        j_string: String,
    }

    let sigs: Vec<String> = Scalars::class()
        .fields()
        .iter()
        .map(|f| f.kind().to_string())
        .collect();

    assert_eq!(
        vec![
            "Z",
            "B",
            "B",
            "C",
            "S",
            "I",
            "J",
            "F",
            "D",
            "Ljava/lang/String;"
        ],
        sigs
    );

    let s = Scalars {
        a_bool: true,
        b_i8: -1,
        c_u8: 2,
        d_u16: 3,
        e_i16: 4,
        f_i32: 5,
        g_i64: 6,
        h_f32: 7.0,
        i_f64: 8.0,
        j_string: "x".to_string(),
    };
    // Only checks that it runs without panicking
    assert!(!s.to_bytes().unwrap().is_empty());
}

// ---- rename ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Renamed", serial_version_uid = 3)]
struct Renamed {
    // Once renamed this sorts before "beta": the object group sorts by Java name, not Rust name
    #[java(rename = "aaa")]
    zzz: Address,
    beta: Address,
}

#[test]
fn test_derive_rename() {
    let class = Renamed::class();
    let names: Vec<&str> = class.fields().iter().map(Field::name).collect();
    assert_eq!(vec!["aaa", "beta"], names);
}

#[test]
fn test_derive_rename_affects_wire_field_name() {
    let r = Renamed {
        zzz: Address {
            city: "NY".to_string(),
        },
        beta: Address {
            city: "LA".to_string(),
        },
    };
    let hex = hex::encode(r.to_bytes().unwrap());
    // "aaa" in modified UTF-8 is 616161, with the length prefix 0003
    assert!(hex.contains("0003616161"), "missing renamed field: {hex}");
    // The Rust-side field name must not appear in the stream
    assert!(!hex.contains("7a7a7a"), "rust field name leaked: {hex}");
}

// ---- skip ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Skipped", serial_version_uid = 4)]
struct Skipped {
    kept: i32,
    #[java(skip)]
    ignored: std::collections::HashMap<String, String>,
    #[java(skip)]
    also_ignored: f64,
}

#[test]
fn test_derive_skip() {
    let class = Skipped::class();
    let names: Vec<&str> = class.fields().iter().map(Field::name).collect();
    assert_eq!(vec!["kept"], names);

    let s = Skipped {
        kept: 9,
        ignored: Default::default(),
        also_ignored: 1.5,
    };
    // The value side skips them too: the stream ends with the single int 9
    let hex = hex::encode(s.to_bytes().unwrap());
    assert!(hex.ends_with("00000009"), "unexpected tail: {hex}");
}

// ---- signature ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Declared", serial_version_uid = 5)]
struct Declared {
    // Declared as an interface type on the Java side; the object written is still com.example.Address
    #[java(signature = "Ljava/util/Map;")]
    lookup: Address,
    #[java(signature = "[Ljava/lang/Object;")]
    raw: Address,
}

#[test]
fn test_derive_signature_override() {
    // Schema side only. Declared is deliberately not serialized: `raw` declares the descriptor
    // [Ljava/lang/Object; while the value side still writes a com.example.Address object. That is
    // exactly what a Java field declared `Object[]` holding an `Address[]` looks like, but this
    // test only checks that `signature` lands in the descriptor verbatim.
    let class = Declared::class();
    let sigs: Vec<String> = class
        .fields()
        .iter()
        .map(|f| f.kind().to_string())
        .collect();
    // Order: both are in the object group, sorted by Java name -> lookup, raw
    assert_eq!(vec!["Ljava/util/Map;", "[Ljava/lang/Object;"], sigs);
}

// ---- Option ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Nullable", serial_version_uid = 6)]
struct Nullable {
    id: i32,
    note: Option<String>,
    home: Option<Address>,
}

#[test]
fn test_derive_option_schema_matches_inner() {
    let sigs: Vec<String> = Nullable::class()
        .fields()
        .iter()
        .map(|f| f.kind().to_string())
        .collect();
    // id is a primitive and comes first; home / note sort by Java name
    assert_eq!(
        vec!["I", "Lcom/example/Address;", "Ljava/lang/String;"],
        sigs
    );
}

#[test]
fn test_derive_option_none_writes_null() {
    let n = Nullable {
        id: 1,
        note: None,
        home: None,
    };
    let hex = hex::encode(n.to_bytes().unwrap());
    // The field-value section follows the classdesc: int 0x00000001, then two TC_NULL (0x70)
    assert!(hex.ends_with("000000017070"), "unexpected tail: {hex}");
}

#[test]
fn test_derive_option_some_writes_value() {
    let n = Nullable {
        id: 1,
        note: Some("hi".to_string()),
        home: Some(Address {
            city: "NY".to_string(),
        }),
    };
    let hex = hex::encode(n.to_bytes().unwrap());
    // "hi" -> TC_STRING(0x74) + len 0x0002 + bytes 6869
    assert!(hex.contains("7400026869"), "missing note: {hex}");
    assert!(!hex.ends_with("7070"), "should not be null: {hex}");
}

// ---- Primitive arrays ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Arrays", serial_version_uid = 7)]
struct Arrays {
    bytes: Vec<u8>,
    shorts: Vec<i16>,
    ints: Vec<i32>,
    longs: Vec<i64>,
    floats: Vec<f32>,
    doubles: Vec<f64>,
}

#[test]
fn test_derive_primitive_arrays() {
    let sigs: Vec<String> = Arrays::class()
        .fields()
        .iter()
        .map(|f| f.kind().to_string())
        .collect();
    // All in the object group (an array is not a primitive), sorted by Java name:
    // bytes, doubles, floats, ints, longs, shorts
    assert_eq!(vec!["[B", "[D", "[F", "[I", "[J", "[S"], sigs);

    let a = Arrays {
        bytes: vec![1, 2],
        shorts: vec![3],
        ints: vec![4],
        longs: vec![5],
        floats: vec![6.0],
        doubles: vec![7.0],
    };
    assert!(!a.to_bytes().unwrap().is_empty());
}

// ---- Object arrays: cross-checked against a hand-written impl ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Team", serial_version_uid = 8)]
struct DerivedTeam {
    size: i32,
    members: Vec<Address>,
}

// The hand-written equivalent: the Java class names and SUIDs must match DerivedTeam exactly, and
// the fields follow ObjectStreamField#compareTo (primitive size first, array members after).
struct HandTeam {
    size: i32,
    members: Vec<HandAddress>,
}

struct HandAddress {
    city: String,
}

impl JavaObject for HandAddress {
    fn class() -> Class {
        Class::builder("com.example.Address", -4433675896693646393)
            .field(Field::builder("city").string())
            .build()
    }
}

impl JavaSerializable for HandAddress {
    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_string(&self.city)?;
        Ok(())
    }
}

impl JavaObject for HandTeam {
    fn class() -> Class {
        Class::builder("com.example.Team", 8)
            .field(Field::builder("size").int())
            .field(Field::builder("members").array(HandAddress::class().signature()))
            .build()
    }
}

impl JavaSerializable for HandTeam {
    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_int(self.size)?;

        // Same magic number that used to be hard-coded in examples/example.rs
        w.begin_array(
            &Class::class_of_array(&HandAddress::class()),
            self.members.len(),
        )?;
        for m in &self.members {
            m.write_to(w)?;
        }

        Ok(())
    }
}

#[test]
fn test_derive_object_array_matches_handwritten() {
    let derived = DerivedTeam {
        size: 2,
        members: vec![
            Address {
                city: "Shanghai".to_string(),
            },
            Address {
                city: "Beijing".to_string(),
            },
        ],
    };
    let hand = HandTeam {
        size: 2,
        members: vec![
            HandAddress {
                city: "Shanghai".to_string(),
            },
            HandAddress {
                city: "Beijing".to_string(),
            },
        ],
    };

    assert_eq!(
        hex::encode(hand.to_bytes().unwrap()),
        hex::encode(derived.to_bytes().unwrap())
    );
}

#[test]
fn test_derive_object_array_is_empty_safe() {
    let derived = DerivedTeam {
        size: 0,
        members: vec![],
    };
    assert!(!derived.to_bytes().unwrap().is_empty());
}

// ---- Borrowed primitive slices ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Borrowed", serial_version_uid = 9)]
struct Borrowed<'a> {
    name: &'a str,
    payload: &'a [u8],
}

#[test]
fn test_derive_borrowed_fields() {
    let sigs: Vec<String> = Borrowed::class()
        .fields()
        .iter()
        .map(|f| f.kind().to_string())
        .collect();
    assert_eq!(vec!["Ljava/lang/String;", "[B"], sigs);

    let b = Borrowed {
        name: "hi",
        payload: &[1, 2, 3],
    };
    assert!(!b.to_bytes().unwrap().is_empty());
}

// ---- Raw-identifier field names ----

#[derive(JavaSerialize)]
#[java(class = "com.example.RawIdent", serial_version_uid = 10)]
struct RawIdent {
    r#final: i32,
    r#type: i32,
}

#[test]
fn test_derive_raw_identifier_field_name() {
    let class = RawIdent::class();
    let names: Vec<&str> = class.fields().iter().map(Field::name).collect();
    // Sorted by Java name once the `r#` prefix is stripped: final, type (both primitives)
    assert_eq!(vec!["final", "type"], names);
}
