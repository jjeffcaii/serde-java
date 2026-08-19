use super::class::Class;
use super::object::Object;
use super::writer::JavaWriter;
use std::io;

pub trait JavaObject {
    fn class() -> Class;
}

pub trait JavaSerializable {
    fn write_object(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()>;
}

/// JavaWriteable extends more features.
pub trait JavaWriteable {
    /// serialize object to writer.
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()>;

    /// serialize object to bytes.
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut b = Vec::<u8>::new();
        let mut w = JavaWriter::new(&mut b)?;
        w.with_dyn(|w| self.write_to(w))?;

        Ok(b)
    }

    /// Serialize object then write to file.
    fn to_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let mut f = std::fs::File::create(path)?;
        let mut w = JavaWriter::new(&mut f)?;
        w.with_dyn(|w| self.write_to(w))?;

        Ok(())
    }
}

impl JavaWriteable for String {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_string(self)?;
        Ok(())
    }
}

impl JavaWriteable for str {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_string(self)?;
        Ok(())
    }
}

impl JavaWriteable for bool {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_bool(*self)?;
        Ok(())
    }
}

impl JavaWriteable for i8 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_byte(*self as u8)?;
        Ok(())
    }
}

impl JavaWriteable for u8 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_byte(*self)?;
        Ok(())
    }
}

impl JavaWriteable for i16 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_short(*self)?;
        Ok(())
    }
}

impl JavaWriteable for u16 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_short(*self as i16)?;
        Ok(())
    }
}

impl JavaWriteable for i32 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_int(*self)?;
        Ok(())
    }
}

impl JavaWriteable for u32 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_int(*self as i32)?;
        Ok(())
    }
}

impl JavaWriteable for i64 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_long(*self)?;
        Ok(())
    }
}

impl JavaWriteable for u64 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_long(*self as i64)?;
        Ok(())
    }
}

impl JavaWriteable for f32 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_float(*self)?;
        Ok(())
    }
}

impl JavaWriteable for f64 {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write_double(*self)?;
        Ok(())
    }
}

// pub fn write_to<W, O>(w: &mut JavaWriter<W>, obj: &O) -> io::Result<()>
// where
//     W: io::Write,
//     O: JavaSerializable + JavaObject,
// {
//     let class = O::class();
//     if class.flags().contains(ClassFlags::WRITE_METHOD) {
//         w.write_object(&class, &[])?;
//         // w.custom_block_begin()?;
//         w.with_dyn(|w| obj.write_object(w))?;
//         w.end_block_data()?;
//         // w.custom_block_end()?;
//     } else {
//         w.write_object(&class, &obj.fields())?;
//     }
//
//     Ok(())
// }

impl<T: JavaSerializable + JavaObject> JavaWriteable for T {
    fn write_to(&self, w: &mut JavaWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Self::class();

        let obj = Object::<T, ()>::builder(class).this(self).build();

        obj.write_to(w)?;

        Ok(())
    }
}
