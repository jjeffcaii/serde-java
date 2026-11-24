use super::flags::*;
use crate::astr::AtomString;
use crate::proto::class::TypeCode;
use crate::{Class, ClassFlags, Field, FieldKind};
use byteorder::{BigEndian, ReadBytesExt};
use hashbrown::HashMap;
use std::io::{self, BufRead as _, Read as _};

pub struct ObjectReader<R> {
    r: io::BufReader<R>,
    stab: HashMap<u32, AtomString>,
    ctab: HashMap<u32, Class>,
    next_handle: u32,
    blkmode: bool,
    block_remaining: usize,
}

impl<R> ObjectReader<R>
where
    R: io::Read,
{
    #[inline]
    pub fn new(r: R) -> io::Result<ObjectReader<R>> {
        Self::with_capacity(r, 0)
    }

    #[inline]
    pub fn with_capacity(r: R, capacity: usize) -> io::Result<ObjectReader<R>> {
        let mut r = ObjectReader {
            r: if capacity == 0 {
                io::BufReader::new(r)
            } else {
                io::BufReader::with_capacity(capacity, r)
            },
            stab: Default::default(),
            ctab: Default::default(),
            next_handle: BASE_WIRE_HANDLE,
            blkmode: false,
            block_remaining: 0,
        };

        r.validate()?;

        Ok(r)
    }

    #[inline]
    fn validate(&mut self) -> io::Result<()> {
        if STREAM_MAGIC != self.get_u16()? {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid magic"));
        }

        if STREAM_VERSION != self.get_u16()? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid version",
            ));
        }

        Ok(())
    }

    /// Peeks at the next raw wire byte, including any block header.
    #[inline]
    pub fn peek(&mut self) -> io::Result<u8> {
        let buf = {
            let mut buf = self.r.buffer();
            if buf.is_empty() {
                buf = self.r.fill_buf()?;
            }
            buf
        };

        let first = buf
            .first()
            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;

        Ok(*first)
    }

    #[inline]
    pub(crate) fn alloc_handle(&mut self) -> u32 {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }

    pub fn advance(&mut self, n: usize) -> io::Result<()> {
        self.r.consume(n);
        Ok(())
    }

    /// Switches primitive reads between block payloads and raw stream bytes.
    /// Returns the previous mode. A partially consumed block must be read first.
    pub fn set_block_data_mode(&mut self, enabled: bool) -> io::Result<bool> {
        if !enabled && self.block_remaining != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unread block data",
            ));
        }
        Ok(std::mem::replace(&mut self.blkmode, enabled))
    }

    pub(crate) fn with_block_data_mode<T>(
        &mut self,
        enabled: bool,
        f: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let previous = self.set_block_data_mode(enabled)?;
        let result = f(self);
        let restored = self.set_block_data_mode(previous);
        let value = result?;
        restored?;
        Ok(value)
    }

    fn next_block(&mut self) -> io::Result<bool> {
        while self.block_remaining == 0 {
            let tag = match self.peek() {
                Ok(tag) => tag,
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
                Err(err) => return Err(err),
            };
            self.block_remaining = match tag {
                TC_BLOCKDATA => {
                    self.advance(1)?;
                    self.r.read_u8()? as usize
                }
                TC_BLOCKDATALONG => {
                    self.advance(1)?;
                    usize::try_from(self.r.read_i32::<BigEndian>()?)
                        .map_err(|_| io::ErrorKind::InvalidData)?
                }
                _ => return Ok(false),
            };
        }
        Ok(true)
    }

    pub(crate) fn end_custom_data(&mut self) -> io::Result<()> {
        while self.next_block()? {
            let available = self.r.fill_buf()?.len();
            if available == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            let n = available.min(self.block_remaining);
            self.r.consume(n);
            self.block_remaining -= n;
        }
        if self.peek()? != TC_ENDBLOCKDATA {
            return Err(io::ErrorKind::InvalidData.into());
        }
        self.advance(1)
    }

    #[inline]
    fn get_bytes<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut bytes = [0; N];
        if !self.blkmode {
            self.r.read_exact(&mut bytes)?;
            return Ok(bytes);
        }

        let mut remaining = bytes.as_mut_slice();
        while !remaining.is_empty() {
            if !self.next_block()? {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            let n = remaining.len().min(self.block_remaining);
            let read = self.r.read(&mut remaining[..n]);
            match read {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(n) => {
                    self.block_remaining -= n;
                    remaining = &mut remaining[n..];
                }
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            }
        }
        Ok(bytes)
    }

    #[inline]
    pub(crate) fn get_i8(&mut self) -> io::Result<i8> {
        Ok(i8::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_i16(&mut self) -> io::Result<i16> {
        Ok(i16::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_i64(&mut self) -> io::Result<i64> {
        Ok(i64::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_u8(&mut self) -> io::Result<u8> {
        Ok(u8::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_be_bytes(self.get_bytes()?))
    }

    #[inline]
    pub(crate) fn get_f64(&mut self) -> io::Result<f64> {
        Ok(f64::from_be_bytes(self.get_bytes()?))
    }

    fn get_string16(&mut self) -> io::Result<AtomString> {
        let n = self.get_u16()? as usize;
        let mut buf = vec![0u8; n];
        self.r.read_exact(&mut buf)?;
        Ok(AtomString::from(unsafe {
            std::str::from_utf8_unchecked(&buf).to_string()
        }))
    }

    pub(crate) fn get_string(&mut self) -> io::Result<AtomString> {
        self.with_block_data_mode(false, Self::read_string)
    }

    fn read_string(&mut self) -> io::Result<AtomString> {
        match self.peek()? {
            TC_REFERENCE => {
                self.advance(1)?;
                let h = self.get_u32()?;
                let s = self
                    .stab
                    .get(&h)
                    .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
                Ok(Clone::clone(s))
            }
            TC_STRING => {
                let h = self.alloc_handle();
                self.advance(1)?;
                let n = self.get_u16()? as usize;
                let mut buf = vec![0u8; n];
                self.r.read_exact(&mut buf)?;
                let s = AtomString::from(unsafe { std::str::from_utf8_unchecked(&buf) });
                self.stab.insert(h, Clone::clone(&s));
                Ok(s)
            }
            TC_LONGSTRING => {
                let h = self.alloc_handle();
                self.advance(1)?;
                let n = self.get_u64()? as usize;
                let mut buf = vec![0u8; n];
                self.r.read_exact(&mut buf)?;
                let s = AtomString::from(unsafe { std::str::from_utf8_unchecked(&buf) });
                self.stab.insert(h, Clone::clone(&s));
                Ok(s)
            }
            _other => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    /// Consumes a null marker, leaving non-null values untouched.
    pub fn read_null(&mut self) -> io::Result<bool> {
        self.with_block_data_mode(false, |r| {
            if r.peek()? == TC_NULL {
                r.advance(1)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }

    pub(crate) fn begin_object(&mut self) -> io::Result<Class> {
        match self.peek()? {
            TC_OBJECT => {
                self.advance(1)?;
                self.read_class()
            }
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported type code 0x{other:02x}"),
            )),
        }
    }

    pub(crate) fn begin_array(&mut self) -> io::Result<Class> {
        match self.peek()? {
            TC_ARRAY => {
                self.advance(1)?;
                let class = self.read_class()?;
                Ok(class)
            }
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported type code 0x{other:02x}"),
            )),
        }
    }

    fn read_class(&mut self) -> io::Result<Class> {
        match self.peek()? {
            TC_NULL => Err(io::Error::from(io::ErrorKind::InvalidData)),
            TC_REFERENCE => {
                self.advance(1)?;
                let h = self.get_u32()?;
                let c = self
                    .ctab
                    .get(&h)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidData))?;
                Ok(Clone::clone(c))
            }
            TC_CLASSDESC => {
                self.advance(1)?;

                let h = self.alloc_handle();

                // class name
                let class_name = self.get_string16()?;

                debug!("read class_name {}", class_name.as_ref());

                // SUID
                let suid = self.get_i64()?;

                // flags
                let flags =
                    ClassFlags::from_bits(self.r.read_u8()?).ok_or(io::ErrorKind::InvalidData)?;

                // field amount
                let field_len = self.get_u16()? as usize;
                let mut fields: Vec<(FieldKind, AtomString)> = Vec::with_capacity(field_len);

                for _ in 0..field_len {
                    let next = match self.get_u8()? {
                        b'Z' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Boolean), name)
                        }
                        b'B' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Byte), name)
                        }
                        b'C' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Char), name)
                        }
                        b'S' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Short), name)
                        }
                        b'I' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Int), name)
                        }
                        b'J' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Long), name)
                        }
                        b'F' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Float), name)
                        }
                        b'D' => {
                            let name = self.get_string16()?;
                            (FieldKind::Primitive(TypeCode::Double), name)
                        }
                        b'L' => {
                            // object
                            let name = self.get_string16()?;
                            let sig = self.get_string()?;
                            (FieldKind::Object(sig), name)
                        }
                        b'[' => {
                            // array
                            let name = self.get_string16()?;
                            let sig = self.get_string()?;
                            (FieldKind::Array(sig), name)
                        }
                        other => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("unknown field type code 0x{other:02x}"),
                            ));
                        }
                    };

                    fields.push(next);
                }

                if self.get_u8()? != TC_ENDBLOCKDATA {
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }

                let mut bu = Class::builder(&class_name, suid).flags(flags);
                for (kind, name) in &fields {
                    bu = bu.field(Field::builder(name.as_ref()).build(Clone::clone(kind)));
                }

                let class = if TC_NULL == self.peek()? {
                    self.advance(1)?;
                    bu.build()
                } else {
                    let super_class = self.read_class()?;
                    bu.super_class(super_class).build()
                };

                self.ctab.insert(h, Clone::clone(&class));

                Ok(class)
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::{JavaDeserializable, JavaFieldsDeserializable, ObjectReader, Reader};
    use crate::{
        Class, Extends, Field, JavaObject, JavaSerializable, JavaWriteable, JavaWriteableExt as _,
        ObjectWriter,
    };
    use anyhow::Result;
    use once_cell::sync::Lazy;
    use std::io;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Animal {
        fly: bool,
        swim: bool,
        run: bool,
    }

    impl JavaObject for Animal {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Animal", 86)
                    .field(Field::builder("fly").boolean())
                    .field(Field::builder("swim").boolean())
                    .field(Field::builder("run").boolean())
                    .sorted()
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Animal {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.fly.write_to(w)?;
            self.run.write_to(w)?;
            self.swim.write_to(w)?;
            Ok(())
        }
    }

    impl JavaFieldsDeserializable for Animal {
        type Fields = (bool, bool, bool);

        fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
        where
            R: io::Read,
        {
            let fly: bool = r.read()?;
            let run: bool = r.read()?;
            let swim: bool = r.read()?;

            Ok((fly, run, swim))
        }
    }

    impl JavaDeserializable for Animal {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let (fly, run, swim) = Self::read_fields(r)?;
            Ok(Self { fly, run, swim })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Cat {
        id: i32,
        name: String,
    }

    impl JavaObject for Cat {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Cat", 42)
                    .super_class(Animal::class())
                    .field(Field::builder("id").int())
                    .field(Field::builder("name").string())
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Cat {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.id.write_to(w)?;
            self.name.write_to(w)?;
            Ok(())
        }
    }

    #[test]
    fn test_default_read_object_between_block_data() -> Result<()> {
        use crate::{ClassFlags, JavaReadable};

        #[derive(Debug, PartialEq)]
        struct BlockFixture(i32, i32, String, u8);

        impl JavaObject for BlockFixture {
            fn class() -> Class {
                Class::builder("BlockFixture", 1)
                    .flags(ClassFlags::SERIALIZABLE | ClassFlags::WRITE_METHOD)
                    .field(Field::builder("value").int())
                    .build()
            }
        }

        impl JavaFieldsDeserializable for BlockFixture {
            type Fields = i32;

            fn read_fields<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<i32> {
                r.read()
            }
        }

        impl JavaDeserializable for BlockFixture {
            fn read_object<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<Self> {
                let leading = r.read()?;
                let value = Self::default_read_object(r)?;
                let label = r.read()?;
                let trailing = r.read()?;
                assert_eq!(
                    r.set_block_data_mode(false).unwrap_err().kind(),
                    io::ErrorKind::InvalidData
                );
                Ok(Self(leading, value, label, trailing))
            }
        }

        // OpenJDK ObjectOutputStream: two shorts separated by flush(), defaultWriteObject(),
        // writeObject("custom"), and write(byte[256]) with the first byte set to 42.
        let mut bytes = hex::decode(concat!(
            "aced00057372000c426c6f636b46697874757265000000000000000103000149000576616c75657870",
            "770212347702567877000001740006637573746f6d7a000001002a",
        ))?;
        bytes.extend([0; 255]);
        bytes.extend(hex::decode("787400056166746572")?);

        let mut r = ObjectReader::with_capacity(bytes.as_slice(), 1)?;
        assert_eq!(
            BlockFixture::read_from(&mut r)?,
            BlockFixture(0x12345678, 0x77000001, "custom".into(), 42)
        );
        assert_eq!(Reader::<String>::read(&mut r)?, "after");

        Ok(())
    }

    #[test]
    fn test_animal() -> Result<()> {
        init();

        let animal = Animal {
            fly: false,
            swim: false,
            run: true,
        };
        let raw = animal.to_bytes()?;

        let mut r = ObjectReader::new(raw.as_slice())?;

        let loaded: Animal = r.read()?;

        assert_eq!(animal, loaded);

        Ok(())
    }

    #[test]
    fn test_read_class() -> Result<()> {
        init();

        let tombili = {
            Cat {
                id: 42,
                name: "Tombili".into(),
            }
        }
        .extends({
            Animal {
                fly: false,
                swim: false,
                run: true,
            }
        });

        let raw = tombili.to_bytes()?;

        let class = ObjectReader::new(raw.as_slice())?.begin_object()?;
        info!("class: {:?}", class);
        assert_eq!(Cat::class(), class);

        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Person {
        id: i32,
        name: String,
        gender: u8,
    }

    impl JavaObject for Person {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Person", 1)
                    .field(Field::builder("id").int())
                    .field(Field::builder("name").string())
                    .field(Field::builder("gender").byte())
                    .build()
            });

            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Person {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.id.write_to(w)?;
            self.name.write_to(w)?;
            self.gender.write_to(w)?;
            Ok(())
        }
    }

    impl JavaFieldsDeserializable for Person {
        type Fields = (i32, String, u8);

        fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
        where
            R: io::Read,
        {
            let id = r.get_i32()?;
            let name = r.get_string()?.to_string();
            let gender = r.get_u8()?;

            Ok((id, name, gender))
        }
    }

    impl JavaDeserializable for Person {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let (id, name, gender) = Self::read_fields(r)?;
            Ok(Self { id, name, gender })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Dog {
        id: i32,
        name: String,
        master: Option<Person>,
    }

    impl JavaObject for Dog {
        fn class() -> Class {
            static CLASS: Lazy<Class> = Lazy::new(|| {
                Class::builder("com.example.Dog", 1)
                    .field(Field::builder("id").int())
                    .field(Field::builder("name").string())
                    .field(Field::builder("master").object(Person::class().signature()))
                    .build()
            });
            Clone::clone(&CLASS)
        }
    }

    impl JavaSerializable for Dog {
        fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
            self.id.write_to(w)?;
            self.name.write_to(w)?;
            match &self.master {
                None => ().write_to(w)?,
                Some(master) => master.write_to(w)?,
            }
            Ok(())
        }
    }

    impl JavaFieldsDeserializable for Dog {
        type Fields = (i32, String, Option<Person>);

        fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
        where
            R: io::Read,
        {
            let id = r.read()?;
            let name = r.read()?;
            let master: Person = r.read()?;
            Ok((id, name, Some(master)))
        }
    }

    impl JavaDeserializable for Dog {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let (id, name, master) = Self::read_fields(r)?;

            Ok(Self { id, name, master })
        }
    }

    #[test]
    fn test_read_association() -> io::Result<()> {
        init();

        let master = Person {
            id: 7,
            name: "Tom".to_string(),
            gender: 1,
        };

        let dog = Dog {
            id: 123,
            name: "Doge".into(),
            master: Some(master),
        };

        let raw = dog.to_bytes()?;

        let mut r = ObjectReader::new(raw.as_slice())?;

        let dog2: Dog = r.read()?;

        assert_eq!(dog, dog2);

        Ok(())
    }
}
