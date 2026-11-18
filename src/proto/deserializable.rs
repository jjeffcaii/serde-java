use super::reader::ObjectReader;
use crate::astr::AtomString;
use crate::{Class, JavaObject};
use std::io;

pub trait Reader<T> {
    fn read(&mut self) -> io::Result<T>;
}

pub trait ArrayReader<T> {
    fn read_all(&mut self) -> io::Result<Vec<T>>;
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

impl<R> Reader<bool> for ObjectReader<R>
where
    R: io::Read,
{
    fn read(&mut self) -> io::Result<bool> {
        let i = self.get_u8()?;
        Ok(i == 1)
    }
}

impl<R> Reader<char> for ObjectReader<R>
where
    R: io::Read,
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

impl<R> Reader<AtomString> for ObjectReader<R>
where
    R: io::Read,
{
    fn read(&mut self) -> io::Result<AtomString> {
        self.get_string()
    }
}

impl<R> Reader<String> for ObjectReader<R>
where
    R: io::Read,
{
    fn read(&mut self) -> io::Result<String> {
        self.get_string().map(|it| it.to_string())
    }
}

impl<R, T> Reader<T> for ObjectReader<R>
where
    T: JavaObject + JavaDeserializable,
    R: io::Read,
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

macro_rules! impl_array_reader {
    ($ty:ty, $class:ident) => {
        impl<R: io::Read> ArrayReader<$ty> for ObjectReader<R> {
            fn read_all(&mut self) -> io::Result<Vec<$ty>> {
                if self.begin_array()? != Class::$class() {
                    return Err(io::ErrorKind::InvalidData.into());
                }
                self.alloc_handle();
                let size = self.get_i32()?;
                if size < 0 {
                    return Err(io::ErrorKind::InvalidData.into());
                }
                let mut items = Vec::new();
                for _ in 0..size {
                    items.push(Reader::<$ty>::read(self)?);
                }
                Ok(items)
            }
        }
    };
}
impl_array_reader!(bool, class_of_boolean_array);
impl_array_reader!(u8, class_of_byte_array);
impl_array_reader!(i8, class_of_byte_array);
impl_array_reader!(u16, class_of_short_array);
impl_array_reader!(i16, class_of_short_array);
impl_array_reader!(u32, class_of_int_array);
impl_array_reader!(i32, class_of_int_array);
impl_array_reader!(u64, class_of_long_array);
impl_array_reader!(i64, class_of_long_array);
impl_array_reader!(usize, class_of_long_array);
impl_array_reader!(isize, class_of_long_array);
impl_array_reader!(f32, class_of_float_array);
impl_array_reader!(f64, class_of_double_array);
impl_array_reader!(String, class_of_string_array);

impl<R: io::Read> Reader<usize> for ObjectReader<R> {
    fn read(&mut self) -> io::Result<usize> {
        usize::try_from(self.get_u64()?).map_err(|_| io::ErrorKind::InvalidData.into())
    }
}
impl<R: io::Read> Reader<isize> for ObjectReader<R> {
    fn read(&mut self) -> io::Result<isize> {
        isize::try_from(self.get_i64()?).map_err(|_| io::ErrorKind::InvalidData.into())
    }
}

impl<R, T> ArrayReader<T> for ObjectReader<R>
where
    T: JavaObject + JavaDeserializable,
    R: io::Read,
{
    fn read_all(&mut self) -> io::Result<Vec<T>> {
        let class = self.begin_array()?;
        let t_class = Class::class_of_array(&T::class());
        if t_class != class {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let _h = self.alloc_handle();

        let size = self.get_i32()?;
        if size < 0 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut items = Vec::<T>::new();
        for _ in 0..size {
            let next = T::read_from(self)?;
            items.push(next);
        }

        Ok(items)
    }
}
