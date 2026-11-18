#![cfg(feature = "derive")]
use serde_java::*;

#[derive(Debug, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "Fixture", serial_version_uid = 1)]
struct Fixture {
    values: Vec<i32>,
    #[java(rename = "name")]
    label: String,
    optional: Option<String>,
    count: i16,
    #[java(skip)]
    cache: String,
}

#[test]
fn test_jvm_fixture() {
    // Generated with: javac tests/fixtures/Fixture.java; java -cp tests/fixtures Fixture
    let bytes = hex::decode("aced000573720007466978747572650000000000000001020004530005636f756e744c00046e616d657400124c6a6176612f6c616e672f537472696e673b4c00086f7074696f6e616c71007e00015b000676616c7565737400025b497870ff857400076669787475726570757200025b494dba602676eab2a502000078700000000300000001fffffffe00000003").unwrap();
    let value = Fixture::from_bytes(&bytes).unwrap();
    assert_eq!(
        value,
        Fixture {
            values: vec![1, -2, 3],
            label: "fixture".into(),
            optional: None,
            count: -123,
            cache: String::new()
        }
    );
    let mut wrong_class = bytes.clone();
    wrong_class[8] = b'X';
    assert_eq!(
        Fixture::from_bytes(&wrong_class).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    assert!(Fixture::from_bytes(&bytes[..bytes.len() - 1]).is_err());
}

#[derive(Debug, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "Nested", serial_version_uid = 2)]
struct Nested {
    child: Option<Fixture>,
    children: Vec<Fixture>,
    bytes: Vec<i8>,
    flags: Vec<bool>,
    shorts: Vec<u16>,
    ints: Vec<u32>,
    longs: Vec<u64>,
    signed_longs: Vec<i64>,
    floats: Vec<f32>,
    doubles: Vec<f64>,
    strings: Vec<String>,
    size: usize,
    offset: isize,
    nullable_array: Option<Vec<u8>>,
}

fn fixture() -> Fixture {
    Fixture {
        values: vec![],
        label: "nested".into(),
        optional: Some("present".into()),
        count: 7,
        cache: String::new(),
    }
}

#[test]
fn test_nested_arrays_and_nullable_fields() {
    let mut value = Nested {
        child: Some(fixture()),
        children: vec![fixture()],
        bytes: vec![-128, 127],
        flags: vec![true, false],
        shorts: vec![0, u16::MAX],
        ints: vec![u32::MAX],
        longs: vec![u64::MAX],
        signed_longs: vec![i64::MIN],
        floats: vec![1.25],
        doubles: vec![-3.5],
        strings: vec!["hello".into(), "world".into()],
        size: 42,
        offset: -42,
        nullable_array: Some(vec![0, 255]),
    };
    assert_eq!(
        Nested::from_bytes(&value.to_bytes().unwrap()).unwrap(),
        value
    );
    value.child = None;
    value.nullable_array = None;
    assert_eq!(
        Nested::from_bytes(&value.to_bytes().unwrap()).unwrap(),
        value
    );
}

#[derive(JavaDeserialize)]
#[java(class = "Manual", serial_version_uid = 1)]
struct Manual {
    #[java(skip)]
    local: usize,
}
impl JavaObject for Manual {
    fn class() -> Class {
        Class::builder("Manual", 1).build()
    }
}

#[test]
fn test_deserialize_without_serialize() {
    let mut bytes = Vec::new();
    let mut w = ObjectWriter::new(&mut bytes).unwrap();
    w.begin_object(&Manual::class()).unwrap();
    assert_eq!(Manual::from_bytes(&bytes).unwrap().local, 0);
}

#[test]
fn test_invalid_array_and_object_tokens() {
    let mut bytes = Vec::new();
    ObjectWriter::new(&mut bytes)
        .unwrap()
        .write_all(&[1_i32])
        .unwrap();
    let mut reader = ObjectReader::new(bytes.as_slice()).unwrap();
    let result: std::io::Result<Vec<String>> = reader.read_all();
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);

    // Negative array lengths must be rejected before allocation.
    let len = bytes.len();
    bytes[len - 8..len - 4].copy_from_slice(&(-1_i32).to_be_bytes());
    let mut reader = ObjectReader::new(bytes.as_slice()).unwrap();
    let result: std::io::Result<Vec<i32>> = reader.read_all();
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);

    for token in [0x70, 0x71, 0x75] {
        // null, reference, array in an object position
        let bytes = [0xac, 0xed, 0, 5, token];
        assert_eq!(
            Fixture::from_bytes(&bytes).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
    }
}

#[derive(Debug, PartialEq, JavaSerialize, JavaDeserialize)]
#[java(class = "Scalars", serial_version_uid = 1)]
struct Scalars {
    boolean: bool,
    character: char,
    byte: i8,
    unsigned_byte: u8,
    short: i16,
    unsigned_short: u16,
    int: i32,
    unsigned_int: u32,
    long: i64,
    unsigned_long: u64,
    float: f32,
    double: f64,
}

#[test]
fn test_scalar_types() {
    let value = Scalars {
        boolean: true,
        character: '界',
        byte: i8::MIN,
        unsigned_byte: u8::MAX,
        short: i16::MIN,
        unsigned_short: u16::MAX,
        int: i32::MIN,
        unsigned_int: u32::MAX,
        long: i64::MIN,
        unsigned_long: u64::MAX,
        float: 1.25,
        double: -2.5,
    };
    assert_eq!(
        Scalars::from_bytes(&value.to_bytes().unwrap()).unwrap(),
        value
    );
}
