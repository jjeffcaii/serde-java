use super::reader::ObjectReader;
use crate::JavaObject;
use crate::astr::AtomString;
use std::io;

pub trait Reader<T> {
    fn read(&mut self) -> io::Result<T>;
}

pub trait JavaFieldsDeserializable: Sized {
    type Fields;

    fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read;
}

pub trait JavaDeserializable: Sized {
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read;
}

pub trait JavaReadable {
    type Item;

    fn read_from<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<Self::Item>;
}

pub trait JavaReadableExt<T> {
    fn from_bytes(bytes: &[u8]) -> io::Result<T>;
}

impl<T: JavaDeserializable + JavaObject> JavaReadable for T {
    type Item = T;

    fn read_from<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<Self::Item> {
        let class = r.begin_object()?;
        let t_class = T::class();
        if class != t_class {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let _h = r.alloc_handle();

        T::read_object(r)
    }
}

impl<T> JavaReadableExt<T> for T
where
    T: JavaReadable<Item = T>,
{
    fn from_bytes(bytes: &[u8]) -> io::Result<T> {
        let mut r = ObjectReader::new(bytes)?;
        T::read_from(&mut r)
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

impl<W> Reader<char> for ObjectReader<W>
where
    W: io::Read,
{
    fn read(&mut self) -> io::Result<char> {
        let u = self.get_u16()?;
        std::char::from_u32(u as u32).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
    }
}

macro_rules! impl_primitive_reader {
    ($ty:ty, $get:ident) => {
        impl<W> Reader<$ty> for ObjectReader<W>
        where
            W: io::Read,
        {
            fn read(&mut self) -> io::Result<$ty> {
                self.$get()
            }
        }
    };
}

impl_primitive_reader!(u8, get_u8);
impl_primitive_reader!(u16, get_u16);
impl_primitive_reader!(u32, get_u32);
impl_primitive_reader!(u64, get_u64);
impl_primitive_reader!(i8, get_i8);
impl_primitive_reader!(i16, get_i16);
impl_primitive_reader!(i32, get_i32);
impl_primitive_reader!(i64, get_i64);
impl_primitive_reader!(f32, get_f32);
impl_primitive_reader!(f64, get_f64);

impl<W> Reader<AtomString> for ObjectReader<W>
where
    W: io::Read,
{
    fn read(&mut self) -> io::Result<AtomString> {
        self.get_string()
    }
}

impl<W> Reader<String> for ObjectReader<W>
where
    W: io::Read,
{
    fn read(&mut self) -> io::Result<String> {
        self.get_string().map(|it| it.to_string())
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
