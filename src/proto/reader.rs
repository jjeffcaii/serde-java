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

    #[inline]
    pub(crate) fn get_i8(&mut self) -> io::Result<i8> {
        self.r.read_i8()
    }

    #[inline]
    pub(crate) fn get_i16(&mut self) -> io::Result<i16> {
        self.r.read_i16::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_i32(&mut self) -> io::Result<i32> {
        self.r.read_i32::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_i64(&mut self) -> io::Result<i64> {
        self.r.read_i64::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_u8(&mut self) -> io::Result<u8> {
        self.r.read_u8()
    }

    #[inline]
    pub(crate) fn get_u16(&mut self) -> io::Result<u16> {
        self.r.read_u16::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_u32(&mut self) -> io::Result<u32> {
        self.r.read_u32::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_u64(&mut self) -> io::Result<u64> {
        self.r.read_u64::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_f32(&mut self) -> io::Result<f32> {
        self.r.read_f32::<BigEndian>()
    }

    #[inline]
    pub(crate) fn get_f64(&mut self) -> io::Result<f64> {
        self.r.read_f64::<BigEndian>()
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

    pub(crate) fn begin_object(&mut self) -> io::Result<Class> {
        match self.peek()? {
            TC_OBJECT => {
                self.advance(1)?;
                self.read_class()
            }
            other => unimplemented!("unsupported tc 0x{:02x}", other),
        }
    }

    pub(crate) fn begin_array(&mut self) -> io::Result<Class> {
        match self.peek()? {
            TC_ARRAY => {
                self.advance(1)?;
                let class = self.read_class()?;
                Ok(class)
            }
            other => unimplemented!("unsupported tc 0x{:02x}", other),
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
                        other => unimplemented!("unknown type code {:?}", other),
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
