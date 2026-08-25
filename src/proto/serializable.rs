use super::class::Class;
use super::object::Object;
use super::writer::{ArrayWriter, ObjectWriter, Writer};
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
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(self)
    }
}

impl JavaWriteable for str {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(self)
    }
}

impl JavaWriteable for bool {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for char {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for i8 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for u8 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for i16 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for u16 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for i32 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for u32 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for i64 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for u64 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for isize {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for usize {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for f32 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for f64 {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(*self)
    }
}

impl JavaWriteable for [bool] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [char] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [i8] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [u8] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [i16] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [u16] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [i32] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [u32] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [i64] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [u64] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [isize] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [usize] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [f32] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [f64] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl JavaWriteable for [String] {
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_all(self)
    }
}

impl<T> JavaWriteable for T
where
    T: JavaSerializable + JavaObject,
{
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Self::class();

        let obj = Object::<T, ()>::builder(class).this(self).build();

        obj.write_to(w)?;

        Ok(())
    }
}

impl<T> JavaWriteable for [T]
where
    T: JavaSerializable + JavaObject,
{
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Class::class_of_array(&T::class());
        w.begin_array(&class, self.len())?;

        for next in self {
            next.write_to(w)?;
        }
        Ok(())
    }
}
