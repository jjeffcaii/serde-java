use super::reader::ObjectReader;
use crate::astr::AtomString;
use crate::{Class, ClassFlags, JavaObject};
use std::io;

/// Typed value reads from an existing object stream.
///
/// [`ObjectReader`] reads primitives in the current block-data mode and
/// decodes object values from their wire markers and class descriptors.
pub trait Reader<T> {
    /// Reads the next value of type `T` from the current stream position.
    fn read(&mut self) -> io::Result<T>;
}

/// Typed reads for Java arrays. The element type selects the expected Java
/// array class, which [`ObjectReader`] checks before reading the elements.
pub trait ArrayReader<T> {
    /// Reads one serialized Java array and returns its elements in stream order.
    fn read_all(&mut self) -> io::Result<Vec<T>>;
}

/// Defines how to read the default serialized fields of a Java class.
///
/// Implement [`Self::read_fields`] to decode the field values. A custom
/// [`JavaDeserializable::read_object`] callback can use
/// [`Self::default_read_object`] to read those values between blocks of custom data.
pub trait JavaFieldsDeserializable: Sized {
    /// The decoded field values used to construct the Rust object.
    type Fields;

    /// Reads the field values in class descriptor order, starting at the first
    /// field. Read primitive fields first, then object and array fields, with
    /// each group sorted by field name.
    fn read_fields<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read;

    /// Mirrors Java's `ObjectInputStream.defaultReadObject` mode switch.
    /// Temporarily disables block-data mode while reading the default fields,
    /// then restores the caller's mode. Finish reading any partially consumed
    /// primitive block before calling this method.
    fn default_read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self::Fields>
    where
        R: io::Read,
    {
        r.with_block_data_mode(false, Self::read_fields)
    }
}

/// Constructs a Rust value from its Java class data.
///
/// The object reader handles the wire header and class descriptor before
/// calling this trait. Implementations read the default fields and any custom
/// data needed to construct the value.
pub trait JavaDeserializable: Sized {
    /// Reads the object's class data. For a class with `WRITE_METHOD`, the object
    /// reader starts this callback in block-data mode. Call
    /// [`JavaFieldsDeserializable::default_read_object`] to read the default fields.
    ///
    /// Read custom object values explicitly. After the callback, the reader skips
    /// any remaining primitive blocks and consumes the end marker.
    fn read_object<R>(r: &mut ObjectReader<R>) -> io::Result<Self>
    where
        R: io::Read;
}

/// Reads a value from an existing [`ObjectReader`].
///
/// Types implementing [`JavaObject`] and [`JavaDeserializable`] get this
/// implementation automatically.
pub trait JavaReadable {
    /// The value produced by the read.
    type Item;

    /// Reads one value from an initialized stream. The object implementation
    /// checks the class descriptor before invoking the class-data callback.
    fn read_from<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<Self::Item>;
}

/// Convenience entry point for reading a [`JavaReadable`] value directly
/// from a byte slice containing a Java serialization stream.
pub trait JavaReadableExt<T> {
    /// Creates a reader from bytes containing a Java stream header and reads
    /// the first value.
    fn from_bytes(bytes: &[u8]) -> io::Result<T>;
}

impl<T: JavaDeserializable + JavaObject> JavaReadable for T {
    type Item = T;

    fn read_from<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<Self::Item> {
        Reader::read(r)
    }
}

macro_rules! impl_java_readable {
    ($($ty:ty),* $(,)?) => {
        $(
            impl JavaReadable for $ty {
                type Item = Self;

                fn read_from<R: io::Read>(r: &mut ObjectReader<R>) -> io::Result<Self> {
                    Reader::read(r)
                }
            }
        )*
    };
}

impl_java_readable!(
    bool, char, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, AtomString, String,
);

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
        // Object markers and class descriptors are outside primitive blocks.
        self.with_block_data_mode(false, |r| {
            let class = r.begin_object()?;
            if class != T::class() {
                return Err(io::ErrorKind::InvalidData.into());
            }
            // Reserve the object's handle before its fields allocate handles.
            r.alloc_handle();

            if class.flags().contains(ClassFlags::WRITE_METHOD) {
                // The end marker belongs to this object's custom data.
                r.with_block_data_mode(true, |r| {
                    let value = T::read_object(r)?;
                    r.end_custom_data()?;
                    Ok(value)
                })
            } else {
                T::read_object(r)
            }
        })
    }
}

macro_rules! impl_array_reader {
    ($ty:ty, $class:ident) => {
        impl<R: io::Read> ArrayReader<$ty> for ObjectReader<R> {
            fn read_all(&mut self) -> io::Result<Vec<$ty>> {
                // The array length and elements follow the descriptor in raw mode.
                self.with_block_data_mode(false, |r| {
                    if r.begin_array()? != Class::$class() {
                        return Err(io::ErrorKind::InvalidData.into());
                    }
                    r.alloc_handle();
                    let size = r.get_i32()?;
                    if size < 0 {
                        return Err(io::ErrorKind::InvalidData.into());
                    }
                    let mut items = Vec::new();
                    for _ in 0..size {
                        items.push(Reader::<$ty>::read(r)?);
                    }
                    Ok(items)
                })
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
        // Read the array in raw mode. Each element enters its own class-data
        // mode through read_from.
        self.with_block_data_mode(false, |r| {
            let class = r.begin_array()?;
            let t_class = Class::class_of_array(&T::class());
            if t_class != class {
                return Err(io::Error::from(io::ErrorKind::InvalidData));
            }

            let _h = r.alloc_handle();

            let size = r.get_i32()?;
            if size < 0 {
                return Err(io::ErrorKind::InvalidData.into());
            }
            let mut items = Vec::<T>::new();
            for _ in 0..size {
                let next = T::read_from(r)?;
                items.push(next);
            }

            Ok(items)
        })
    }
}
