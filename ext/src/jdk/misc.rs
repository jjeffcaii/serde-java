use super::boolean::Boolean;
use super::character::Character;
use super::number::{Byte, Double, Float, Integer, Long, Short};
use serde_java::{JavaReadable, JavaWriteable, Layout, ObjectReader, ObjectWriter, Reader};
use std::any::Any;
use std::io;

// auto write boxed primitive values, eg: i32 should be written by Integer
// TODO: how to make it zero-cost???
#[inline]
pub(crate) fn write_boxed<T: 'static + JavaWriteable>(
    w: &mut ObjectWriter<&mut dyn io::Write>,
    t: &T,
) -> io::Result<()> {
    if let Some(i) = (t as &dyn Any).downcast_ref::<bool>() {
        return Boolean::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<char>() {
        return Character::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<i8>() {
        return Byte::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<i16>() {
        return Short::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<i32>() {
        return Integer::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<i64>() {
        return Long::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<u8>() {
        return Byte::layout(&(*i as i8)).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<u16>() {
        return Short::layout(&(*i as i16)).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<u32>() {
        return Integer::layout(&(*i as i32)).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<u64>() {
        return Long::layout(&(*i as i64)).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<f32>() {
        return Float::layout(i).write_to(w);
    }
    if let Some(i) = (t as &dyn Any).downcast_ref::<f64>() {
        return Double::layout(i).write_to(w);
    }

    t.write_to(w)
}

/// Reads Java wrapper objects for primitive types and uses the regular reader
/// for other types.
pub(crate) fn read_boxed<T, R>(r: &mut ObjectReader<R>) -> io::Result<T>
where
    T: 'static + JavaReadable<Item = T>,
    R: io::Read,
{
    let mut value = None::<T>;
    let slot = &mut value as &mut dyn Any;

    macro_rules! read_wrapper {
        ($ty:ty, $wrapper:ty, $inner:ty) => {
            if let Some(slot) = slot.downcast_mut::<Option<$ty>>() {
                let wrapped = Reader::<$wrapper>::read(r)?;
                *slot = Some(<$inner>::from(wrapped) as $ty);
                return Ok(value.unwrap());
            }
        };
    }

    read_wrapper!(bool, Boolean, bool);
    read_wrapper!(char, Character, char);
    read_wrapper!(i8, Byte, i8);
    read_wrapper!(i16, Short, i16);
    read_wrapper!(i32, Integer, i32);
    read_wrapper!(i64, Long, i64);
    read_wrapper!(u8, Byte, i8);
    read_wrapper!(u16, Short, i16);
    read_wrapper!(u32, Integer, i32);
    read_wrapper!(u64, Long, i64);
    read_wrapper!(f32, Float, f32);
    read_wrapper!(f64, Double, f64);

    T::read_from(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_boxed_round_trip() -> io::Result<()> {
        macro_rules! round_trip {
            ($ty:ty, $value:expr) => {{
                let input: $ty = $value;
                let mut bytes = Vec::new();
                ObjectWriter::new(&mut bytes)?.with_dyn(|w| write_boxed(w, &input))?;
                let mut reader = ObjectReader::new(bytes.as_slice())?;
                let output: $ty = read_boxed(&mut reader)?;
                assert_eq!(output, input);
            }};
        }

        round_trip!(bool, true);
        round_trip!(char, '世');
        round_trip!(i8, i8::MIN);
        round_trip!(i16, i16::MIN);
        round_trip!(i32, i32::MIN);
        round_trip!(i64, i64::MIN);
        round_trip!(u8, u8::MAX);
        round_trip!(u16, u16::MAX);
        round_trip!(u32, u32::MAX);
        round_trip!(u64, u64::MAX);
        round_trip!(f32, -1.5);
        round_trip!(f64, 2.5);
        round_trip!(String, String::from("fallback"));

        Ok(())
    }
}
