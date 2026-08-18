#![cfg(feature = "derive")]

use serde_java::*;

// ---- 复用 src/proto/mod.rs 测试里那条来自真实 ObjectOutputStream 的 fixture ----

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

// ---- 嵌套对象，同样对已知 fixture ----

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

// ---- 排序：声明顺序刻意打乱，断言 class() 里的字段顺序 ----

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
    // 基本类型在前（beta:I, yak:J 按名字排），然后对象（alpha, zebra 按名字排）
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
        vec!["Z", "B", "B", "C", "S", "I", "J", "F", "D", "Ljava/lang/String;"],
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
    // 只验证能跑通、不 panic
    assert!(!s.to_bytes().unwrap().is_empty());
}

// ---- rename ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Renamed", serial_version_uid = 3)]
struct Renamed {
    // 改名后应排到 "beta" 之前（对象组按 Java 名排序，不是 Rust 名）
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
        zzz: Address { city: "NY".to_string() },
        beta: Address { city: "LA".to_string() },
    };
    let hex = hex::encode(r.to_bytes().unwrap());
    // "aaa" 的 modified-UTF-8 是 616161，长度前缀 0003
    assert!(hex.contains("0003616161"), "missing renamed field: {hex}");
    // Rust 侧字段名不应出现在流里
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
    // fields() 也只产出一个值，与 class() 对齐
    assert_eq!(1, s.fields().len());
}

// ---- signature ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Declared", serial_version_uid = 5)]
struct Declared {
    // Java 侧声明为接口类型，实际写入的对象仍是 com.example.Address
    #[java(signature = "Ljava/util/Map;")]
    lookup: Address,
    #[java(signature = "[Ljava/lang/Object;")]
    raw: Address,
}

#[test]
fn test_derive_signature_override() {
    // 只检查 schema 侧。故意不序列化 Declared：`raw` 的描述符声明成
    // [Ljava/lang/Object; 而值侧仍写一个 com.example.Address 对象，这样的流
    // JVM 是会拒的 —— 本测试要验的只是 signature 有没有原样落进描述符。
    let class = Declared::class();
    let sigs: Vec<String> = class.fields().iter().map(|f| f.kind().to_string()).collect();
    // 顺序：都是对象组，按 Java 名排 -> lookup, raw
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
    // id 是基本类型排最前；home / note 按 Java 名排序
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
    // 字段值区在 classdesc 之后：int 0x00000001，然后两个 TC_NULL (0x70)
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

// ---- 基本类型数组 ----

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
    // 全是对象组（数组不是 primitive），按 Java 名排序：
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

// ---- 对象数组：与手写实现对拍 ----

#[derive(JavaSerialize)]
#[java(class = "com.example.Team", serial_version_uid = 8)]
struct DerivedTeam {
    size: i32,
    members: Vec<Address>,
}

// 手写等价物：Java 类名与 SUID 必须与 DerivedTeam 完全一致，字段顺序按
// ObjectStreamField#compareTo（size 是基本类型排前，members 是数组排后）。
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
    fn fields(&self) -> Vec<FieldValue<'_>> {
        vec![FieldValue::String(&self.city)]
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
    fn fields(&self) -> Vec<FieldValue<'_>> {
        vec![
            FieldValue::Int(self.size),
            FieldValue::Array(
                // 与 examples/example.rs 里原先硬编码的魔数同源
                Class::class_of_array(HandAddress::class(), 7549007861314292831),
                self.members
                    .iter()
                    .map(|a| (HandAddress::class(), a as &dyn JavaSerializable))
                    .collect(),
            ),
        ]
    }
}

#[test]
fn test_derive_object_array_matches_handwritten() {
    let derived = DerivedTeam {
        size: 2,
        members: vec![
            Address { city: "Shanghai".to_string() },
            Address { city: "Beijing".to_string() },
        ],
    };
    let hand = HandTeam {
        size: 2,
        members: vec![
            HandAddress { city: "Shanghai".to_string() },
            HandAddress { city: "Beijing".to_string() },
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

// ---- 借用形式的基本类型切片 ----

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

// ---- 原始标识符（raw identifier）字段名 ----

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
    // 去掉 `r#` 前缀后按 Java 名排序：final, type（两者都是基本类型）
    assert_eq!(vec!["final", "type"], names);
}
