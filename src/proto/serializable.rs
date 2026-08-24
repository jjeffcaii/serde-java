use super::class::Class;
use super::object::Object;
use super::writer::ObjectWriter;
use std::collections::VecDeque;
use std::io;

pub trait JavaObject {
    fn class() -> Class;
}

pub trait JavaSerializable {
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()>;

    fn default_write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.set_block_data_mode(false);
        self.write_fields(w)?;
        w.set_block_data_mode(true);
        Ok(())
    }

    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)
    }
}

/// JavaWriteable extends more features.
pub trait JavaWriteable {
    /// serialize object to writer.
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()>;

    /// serialize object to bytes.
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut b = Vec::<u8>::new();
        let mut w = ObjectWriter::new(&mut b)?;
        w.with_dyn(|w| self.write_to(w))?;

        Ok(b)
    }

    /// Serialize object then write to file.
    fn to_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let mut f = std::fs::File::create(path)?;
        let mut w = ObjectWriter::new(&mut f)?;
        w.with_dyn(|w| self.write_to(w))?;

        Ok(())
    }
}

impl JavaWriteable for String {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_string(self)?;
        Ok(())
    }
}

impl JavaWriteable for str {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_string(self)?;
        Ok(())
    }
}

impl JavaWriteable for bool {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_bool(*self)?;
        Ok(())
    }
}

impl JavaWriteable for i8 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_byte(*self as u8)?;
        Ok(())
    }
}

impl JavaWriteable for u8 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_byte(*self)?;
        Ok(())
    }
}

impl JavaWriteable for i16 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_short(*self)?;
        Ok(())
    }
}

impl JavaWriteable for u16 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_short(*self as i16)?;
        Ok(())
    }
}

impl JavaWriteable for i32 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_int(*self)?;
        Ok(())
    }
}

impl JavaWriteable for u32 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_int(*self as i32)?;
        Ok(())
    }
}

impl JavaWriteable for i64 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_long(*self)?;
        Ok(())
    }
}

impl JavaWriteable for u64 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_long(*self as i64)?;
        Ok(())
    }
}

impl JavaWriteable for f32 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_float(*self)?;
        Ok(())
    }
}

impl JavaWriteable for f64 {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_double(*self)?;
        Ok(())
    }
}

impl JavaWriteable for [u8] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_byte_array(self)?;
        Ok(())
    }
}

impl JavaWriteable for [i16] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_short_array(self)?;
        Ok(())
    }
}

impl JavaWriteable for [i32] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_int_array(self)?;
        Ok(())
    }
}

impl JavaWriteable for [i64] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_long_array(self)?;
        Ok(())
    }
}

impl JavaWriteable for [f32] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_float_array(self)?;
        Ok(())
    }
}

impl JavaWriteable for [f64] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_double_array(self)?;
        Ok(())
    }
}

impl JavaWriteable for [String] {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.begin_array(&Class::class_of_string_array(), self.len())?;
        for next in self {
            w.write_string(next)?;
        }
        Ok(())
    }
}

impl<T> JavaWriteable for T
where
    T: JavaSerializable + JavaObject,
{
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Self::class();

        let obj = Object::<T, ()>::builder(class).this(self).build();

        obj.write_to(w)?;

        Ok(())
    }
}

impl<T> JavaWriteable for VecDeque<T>
where
    T: JavaSerializable + JavaObject,
{
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Class::class_of_array(&T::class());
        w.begin_array(&class, self.len())?;

        for next in self {
            next.write_to(w)?;
        }
        Ok(())
    }
}

impl<T> JavaWriteable for Vec<T>
where
    T: JavaSerializable + JavaObject,
{
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Class::class_of_array(&T::class());
        w.begin_array(&class, self.len())?;

        for next in self {
            next.write_to(w)?;
        }
        Ok(())
    }
}
