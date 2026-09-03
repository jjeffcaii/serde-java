use super::misc::write_boxed;
use serde_java::__private::once_cell::sync::Lazy;
use serde_java::{
    Class, ClassFlags, Field, JavaObject, JavaSerializable, JavaWriteable, Layout, ObjectWriter,
};
use std::collections::HashMap as StdHashMap;
use std::hash::Hash;
use std::io;
use std::ops::Deref;

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

#[derive(Clone)]
pub struct HashMapOwned<K, V> {
    capacity: i32,
    load_factor: f32,
    threshold: i32,
    inner: StdHashMap<K, V>,
}

impl<K, V> HashMapOwned<K, V> {
    pub fn builder(origin: StdHashMap<K, V>) -> HashMapOwnedBuilder<K, V> {
        HashMapOwnedBuilder {
            load_factor: DEFAULT_LOAD_FACTOR,
            init_capacity: None,
            inner: origin,
        }
    }
}

impl<K, V> Into<StdHashMap<K, V>> for HashMapOwned<K, V> {
    fn into(self) -> StdHashMap<K, V> {
        self.inner
    }
}

impl<K, V> Deref for HashMapOwned<K, V> {
    type Target = StdHashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V> From<StdHashMap<K, V>> for HashMapOwned<K, V> {
    fn from(value: StdHashMap<K, V>) -> Self {
        Self::builder(value).build()
    }
}

impl<K, V> JavaObject for HashMapOwned<K, V> {
    fn class() -> Class {
        Clone::clone(&CLASS_OF_HASH_MAP)
    }
}

impl<K, V> JavaSerializable for HashMapOwned<K, V>
where
    K: 'static + JavaWriteable + Eq + Hash,
    V: 'static + JavaWriteable,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.load_factor.write_to(w)?;
        self.threshold.write_to(w)?;

        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let buckets = self.capacity;
        let size = self.inner.len() as i32;

        self.default_write_object(w)?;

        buckets.write_to(w)?;
        size.write_to(w)?;

        for (k, v) in &self.inner {
            write_boxed(w, k)?;
            write_boxed(w, v)?;
        }

        Ok(())
    }
}

pub struct HashMapOwnedBuilder<K, V> {
    load_factor: f32,
    init_capacity: Option<usize>,
    inner: StdHashMap<K, V>,
}

impl<K, V> HashMapOwnedBuilder<K, V> {
    pub fn load_factory(mut self, load_factor: f32) -> Self {
        self.load_factor = load_factor;
        self
    }

    pub fn init_capacity(mut self, init_capacity: usize) -> Self {
        self.init_capacity = Some(init_capacity);
        self
    }

    #[inline]
    pub fn build(self) -> HashMapOwned<K, V> {
        let Self {
            load_factor,
            init_capacity,
            inner,
        } = self;

        let (threshold, capacity) = match init_capacity {
            None => {
                if inner.is_empty() {
                    (0i32, DEFAULT_INIT_CAPACITY)
                } else {
                    (
                        ((DEFAULT_INIT_CAPACITY as f32) * DEFAULT_LOAD_FACTOR) as i32,
                        DEFAULT_INIT_CAPACITY,
                    )
                }
            }
            Some(n) => {
                let capacity = n.min(MAXIMUM_CAPACITY);
                if inner.is_empty() {
                    (0, capacity)
                } else {
                    (table_size_for(capacity) as i32, capacity)
                }
            }
        };

        HashMapOwned {
            capacity: capacity as i32,
            load_factor,
            threshold,
            inner,
        }
    }
}

pub struct HashMap<'a, K, V> {
    load_factor: f32,
    threshold: i32,
    capacity: i32,
    inner: &'a StdHashMap<K, V>,
}

impl<'a, K, V> From<&'a StdHashMap<K, V>> for HashMap<'a, K, V> {
    fn from(value: &'a StdHashMap<K, V>) -> Self {
        Self::builder(value).build()
    }
}

impl<'a, K, V> Deref for HashMap<'a, K, V> {
    type Target = StdHashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

pub struct HashMapBuilder<'a, K, V> {
    load_factor: f32,
    init_capacity: Option<usize>,
    inner: &'a StdHashMap<K, V>,
}

impl<'a, K, V> HashMapBuilder<'a, K, V> {
    pub fn load_factory(mut self, load_factor: f32) -> Self {
        self.load_factor = load_factor;
        self
    }

    pub fn init_capacity(mut self, init_capacity: usize) -> Self {
        self.init_capacity = Some(init_capacity);
        self
    }

    #[inline]
    pub fn build(self) -> HashMap<'a, K, V> {
        let Self {
            load_factor,
            init_capacity,
            inner,
        } = self;

        let (threshold, capacity) = match init_capacity {
            None => {
                if inner.is_empty() {
                    (0i32, DEFAULT_INIT_CAPACITY)
                } else {
                    (
                        ((DEFAULT_INIT_CAPACITY as f32) * DEFAULT_LOAD_FACTOR) as i32,
                        DEFAULT_INIT_CAPACITY,
                    )
                }
            }
            Some(n) => {
                let capacity = n.min(MAXIMUM_CAPACITY);
                if inner.is_empty() {
                    (0, capacity)
                } else {
                    (table_size_for(capacity) as i32, capacity)
                }
            }
        };

        HashMap {
            capacity: capacity as i32,
            load_factor,
            threshold,
            inner,
        }
    }
}

impl<'a, K, V> HashMap<'a, K, V> {
    #[inline]
    pub fn builder(origin: &'a StdHashMap<K, V>) -> HashMapBuilder<'a, K, V> {
        HashMapBuilder {
            load_factor: DEFAULT_LOAD_FACTOR,
            init_capacity: None,
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
    K: 'static + JavaWriteable + Eq + Hash,
    V: 'static + JavaWriteable,
{
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        #[cfg(test)]
        {
            info!("load_factor: {:?}", self.load_factor);
            info!("threshold: {:?}", self.threshold);
        }

        self.load_factor.write_to(w)?;
        self.threshold.write_to(w)?;

        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let buckets = self.capacity;
        let size = self.inner.len() as i32;

        self.default_write_object(w)?;

        buckets.write_to(w)?;
        size.write_to(w)?;

        #[cfg(test)]
        {
            info!("buckets: {:?}", buckets);
            info!("size: {:?}", size);
        }

        for (k, v) in self.inner {
            write_boxed(w, k)?;
            write_boxed(w, v)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_java::{JavaSerialize, JavaWriteableExt};

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_table() {
        assert_eq!(32, table_size_for(20));
        assert_eq!(16, table_size_for(16));
    }

    #[test]
    fn test_hashmap() -> io::Result<()> {
        init();

        // empty
        {
            let empty = StdHashMap::<String, String>::new();
            let raw = HashMap::layout(&empty).to_bytes()?;

            assert_eq!(
                "aced0005737200116a6176612e7574696c2e486173684d61700507dac1c31660d103000246000a6c6f6164466163746f724900097468726573686f6c6478703f400000000000007708000000100000000078",
                hex::encode(&raw)
            );
        }

        {
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
        }

        Ok(())
    }

    #[derive(Debug, JavaSerialize)]
    #[java(class="com.example.HashMapDemo",serial_version_uid=-5707132217429457092)]
    struct HashMapDemo {
        #[java(signature = "Ljava/util/Map;", with = "crate::java::util::HashMap")]
        exts: Option<StdHashMap<String, String>>,
        #[java(signature = "Ljava/util/Map;", with = "crate::java::util::HashMap")]
        scores: StdHashMap<i32, f64>,
    }

    #[test]
    fn test_hashmap_in_fields() -> io::Result<()> {
        init();

        let exts = {
            let mut m = StdHashMap::<String, String>::new();
            m.insert("hello".to_string(), "world".to_string());
            m
        };
        let scores = {
            let mut m = StdHashMap::<i32, f64>::new();
            m.insert(1, 3.14);
            m
        };

        let input = HashMapDemo {
            exts: Some(exts),
            scores,
        };

        info!("input: {:?}", input);

        let raw = input.to_bytes()?;

        assert_eq!(
            "aced000573720017636f6d2e6578616d706c652e486173684d617044656d6fb0cc317c65ee073c0200024c00046578747374000f4c6a6176612f7574696c2f4d61703b4c000673636f72657371007e00017870737200116a6176612e7574696c2e486173684d61700507dac1c31660d103000246000a6c6f6164466163746f724900097468726573686f6c6478703f4000000000000c7708000000100000000174000568656c6c6f740005776f726c64787371007e00033f4000000000000c77080000001000000001737200116a6176612e6c616e672e496e746567657212e2a0a4f781873802000149000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b020000787000000001737200106a6176612e6c616e672e446f75626c6580b3c24a296bfb0402000144000576616c75657871007e000940091eb851eb851f78",
            hex::encode(&raw),
        );

        Ok(())
    }

    #[test]
    fn test_map_in_map() -> io::Result<()> {
        init();

        let origin = {
            let mut origin = StdHashMap::<String, HashMapOwned<String, i32>>::new();

            let mut a = StdHashMap::<String, i32>::new();
            a.insert("bar".to_owned(), 111);

            origin.insert("foo".to_owned(), HashMapOwned::from(a));

            origin
        };

        let m = HashMapOwned::from(origin);

        let raw = m.to_bytes()?;

        assert_eq!(
            "aced0005737200116a6176612e7574696c2e486173684d61700507dac1c31660d103000246000a6c6f6164466163746f724900097468726573686f6c6478703f4000000000000c77080000001000000001740003666f6f7371007e00003f4000000000000c77080000001000000001740003626172737200116a6176612e6c616e672e496e746567657212e2a0a4f781873802000149000576616c7565787200106a6176612e6c616e672e4e756d62657286ac951d0b94e08b02000078700000006f7878",
            hex::encode(&raw),
        );

        Ok(())
    }
}
