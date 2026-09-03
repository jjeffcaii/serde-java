use super::flags::*;
use crate::astr::AtomString;
use crate::proto::class::TypeCode;
use crate::{Class, ClassFlags, Field, FieldKind, JavaObject};
use byteorder::{BigEndian, ReadBytesExt};
use hashbrown::HashMap;
use std::io::{self, BufRead as _, Read as _};

pub trait Reader<T> {
    fn read(&mut self) -> io::Result<T>;
}

pub trait JavaDeserializable: Sized {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read;
}

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
    fn get_i8(&mut self) -> io::Result<i8> {
        self.r.read_i8()
    }

    #[inline]
    fn get_i16(&mut self) -> io::Result<i16> {
        self.r.read_i16::<BigEndian>()
    }

    #[inline]
    fn get_i32(&mut self) -> io::Result<i32> {
        self.r.read_i32::<BigEndian>()
    }

    #[inline]
    fn get_i64(&mut self) -> io::Result<i64> {
        self.r.read_i64::<BigEndian>()
    }

    #[inline]
    fn get_u8(&mut self) -> io::Result<u8> {
        self.r.read_u8()
    }

    #[inline]
    fn get_u16(&mut self) -> io::Result<u16> {
        self.r.read_u16::<BigEndian>()
    }

    #[inline]
    fn get_u32(&mut self) -> io::Result<u32> {
        self.r.read_u32::<BigEndian>()
    }

    #[inline]
    fn get_u64(&mut self) -> io::Result<u64> {
        self.r.read_u64::<BigEndian>()
    }

    #[inline]
    fn get_f32(&mut self) -> io::Result<f32> {
        self.r.read_f32::<BigEndian>()
    }

    #[inline]
    fn get_f64(&mut self) -> io::Result<f64> {
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

    fn get_string(&mut self) -> io::Result<AtomString> {
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

    pub fn begin_object(&mut self) -> io::Result<Class> {
        match self.peek()? {
            TC_OBJECT => {
                self.advance(1)?;
                self.read_class()
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

impl<W> Reader<bool> for ObjectReader<W>
where
    W: io::Read,
{
    fn read(&mut self) -> io::Result<bool> {
        let i = self.get_u8()?;
        Ok(i == 1)
    }
}

impl<W, T> Reader<T> for ObjectReader<W>
where
    T: JavaObject + JavaDeserializable,
    W: io::Read,
{
    fn read(&mut self) -> io::Result<T> {
        let class = self.begin_object()?;
        let t_class = T::class();
        if class != t_class {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let _h = self.alloc_handle();

        T::read_object(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::reader::{JavaDeserializable, ObjectReader, Reader};
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

    impl JavaDeserializable for Animal {
        fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
        where
            R: io::Read,
        {
            let fly: bool = r.read()?;
            let run: bool = r.read()?;
            let swim: bool = r.read()?;

            Ok(Self { fly, run, swim })
        }
    }

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
}
