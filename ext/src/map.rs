use serde_java::__private::Lazy;
use serde_java::{
    Class, ClassFlags, Field, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter,
};
use std::collections::HashMap as StdHashMap;
use std::hash::Hash;
use std::io;

const DEFAULT_LOAD_FACTOR: f32 = 0.75;
const DEFAULT_INIT_CAPACITY: usize = 16;
const MAXIMUM_CAPACITY: usize = 1 << 30;

static CLASS_OF_HASH_MAP: Lazy<Class> = Lazy::new(|| {
    Class::builder("java.util.HashMap", 362498820763181265)
        .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
        .field(Field::builder("loadFactor").float())
        .field(Field::builder("threshold").int())
        .build()
});

pub struct HashMap<'a, K, V> {
    load_factor: f32,
    threshold: i32,
    inner: &'a StdHashMap<K, V>,
}

impl<'a, K, V> HashMap<'a, K, V> {
    #[inline]
    fn capacity(&self) -> i32 {
        if self.threshold > 0 {
            self.threshold
        } else {
            DEFAULT_INIT_CAPACITY as i32
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct HashMapBuilder<'a, K, V> {
    load_factor: f32,
    init_capacity: usize,
    inner: &'a StdHashMap<K, V>,
}

impl<'a, K, V> HashMapBuilder<'a, K, V> {
    pub fn load_factory(mut self, load_factor: f32) -> Self {
        self.load_factor = load_factor;
        self
    }

    pub fn init_capacity(mut self, init_capacity: usize) -> Self {
        self.init_capacity = init_capacity;
        self
    }

    pub fn build(self) -> HashMap<'a, K, V> {
        let Self {
            load_factor,
            mut init_capacity,
            inner,
        } = self;

        if init_capacity > MAXIMUM_CAPACITY {
            init_capacity = MAXIMUM_CAPACITY;
        }

        let threshold = if init_capacity == 0 {
            0
        } else {
            table_size_for(init_capacity) as i32
        };

        HashMap {
            load_factor,
            threshold,
            inner,
        }
    }
}

#[inline]
fn table_size_for(capacity: usize) -> usize {
    let cap = capacity as i32;
    // Java's `>>>` uses only the low 5 bits of the shift distance for `int`.
    let shift = cap.wrapping_sub(1).leading_zeros() & 31;
    let n = ((-1i32 as u32) >> shift) as i32;
    if n < 0 {
        1
    } else if (n as usize) >= MAXIMUM_CAPACITY {
        MAXIMUM_CAPACITY
    } else {
        (n as usize) + 1
    }
}

impl<'a, K, V> HashMap<'a, K, V> {
    pub fn builder(origin: &'a StdHashMap<K, V>) -> HashMapBuilder<'a, K, V> {
        HashMapBuilder {
            load_factor: DEFAULT_LOAD_FACTOR,
            init_capacity: 0,
            inner: origin,
        }
    }
}

impl<'a, K, V> JavaObject for HashMap<'a, K, V> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_HASH_MAP)
    }
}

impl<'a, K, V> JavaSerializable for HashMap<'a, K, V>
where
    K: JavaWriteable + Eq + Hash,
    V: JavaWriteable,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let threshold = if self.threshold > 0 {
            self.threshold
        } else {
            ((DEFAULT_INIT_CAPACITY as f32) * self.load_factor) as i32
        };

        self.load_factor.write_to(w)?;
        threshold.write_to(w)?;

        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let buckets = self.capacity();
        let size = self.len() as i32;

        self.default_write_object(w)?;

        buckets.write_to(w)?;
        size.write_to(w)?;

        for (k, v) in self.inner {
            k.write_to(w)?;
            v.write_to(w)?;
        }

        Ok(())
    }
}

impl<'a, K, V> Layout<'a> for HashMap<'a, K, V>
where
    K: JavaWriteable + Eq + Hash + 'a,
    V: JavaWriteable,
{
    type Input = StdHashMap<K, V>;
    type Output = HashMap<'a, K, V>;

    fn layout(input: &'a Self::Input) -> Self::Output {
        HashMap::builder(input).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_java::JavaSerialize;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_hashmap() -> io::Result<()> {
        init();

        let origin = {
            let mut m = StdHashMap::<String, String>::new();
            m.insert("hello".to_string(), "world".to_string());
            m
        };

        let m = HashMap::builder(&origin).build();

        let raw = m.to_bytes()?;

        assert_eq!(
            "aced0005737200116a6176612e7574696c2e486173684d61700507dac1c31660d103000246000a6c6f6164466163746f724900097468726573686f6c6478703f4000000000000c7708000000100000000174000568656c6c6f740005776f726c6478",
            hex::encode(&raw)
        );

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(class="com.example.HashMapDemo",serial_version_uid=-5707132217429457092)]
    struct HashMapDemo {
        #[java(signature = "Ljava/util/Map;", with = "crate::HashMap")]
        exts: Option<std::collections::HashMap<String, String>>,
    }

    #[test]
    fn test_hashmap_field() -> io::Result<()> {
        init();

        let exts = {
            let mut m = std::collections::HashMap::<String, String>::new();
            m.insert("hello".to_string(), "world".to_string());
            m
        };

        let input = HashMapDemo { exts: Some(exts) };

        info!("input: {:?}", input);

        let raw = input.to_bytes()?;

        assert_eq!(
            "aced000573720017636f6d2e6578616d706c652e486173684d617044656d6fb0cc317c65ee073c0200014c00046578747374000f4c6a6176612f7574696c2f4d61703b7870737200116a6176612e7574696c2e486173684d61700507dac1c31660d103000246000a6c6f6164466163746f724900097468726573686f6c6478703f4000000000000c7708000000100000000174000568656c6c6f740005776f726c6478",
            hex::encode(&raw),
        );

        Ok(())
    }
}
