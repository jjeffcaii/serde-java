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
