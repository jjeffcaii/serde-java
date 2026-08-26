use super::boolean::Boolean;
use super::character::Character;
use super::number::{Byte, Double, Float, Integer, Long, Short};
use serde_java::{JavaWriteable, Layout, ObjectWriter};
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
