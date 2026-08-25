use super::{Class, FieldKind};
use crate::astr::AtomString;
use crate::util::to_modified_utf8;
use byteorder::{BigEndian, WriteBytesExt};
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::io;

// Stream header
const STREAM_MAGIC: u16 = 0xaced;
const STREAM_VERSION: u16 = 0x0005;

// TC_* type code
const TC_NULL: u8 = 0x70;
const TC_REFERENCE: u8 = 0x71;
const TC_CLASSDESC: u8 = 0x72;
const TC_OBJECT: u8 = 0x73;
const TC_STRING: u8 = 0x74;
const TC_ARRAY: u8 = 0x75;
const TC_CLASS: u8 = 0x76;
const TC_BLOCKDATA: u8 = 0x77;
const TC_ENDBLOCKDATA: u8 = 0x78;
const TC_RESET: u8 = 0x79;
const TC_BLOCKDATALONG: u8 = 0x7a;
const TC_EXCEPTION: u8 = 0x7b;
const TC_LONGSTRING: u8 = 0x7c;
const TC_PROXYCLASSDESC: u8 = 0x7d;
const TC_ENUM: u8 = 0x7e;
const TC_MAX: u8 = 0x7e;
const TC_NULLREF: u8 = 0x70; // alias

const BASE_WIRE_HANDLE: u32 = 0x7e0000;

type BlockData = SmallVec<[u8; 32]>;

pub struct ObjectWriter<W> {
    w: W,
    next_handle: u32,
    string_handles: HashMap<String, u32>,
    class_handles: HashMap<AtomString, u32>,
    blkmode: bool,
    blk: BlockData,
}

impl<W: io::Write> ObjectWriter<W> {
    #[inline]
    fn put_u8(&mut self, b: u8) -> io::Result<()> {
        self.w.write_u8(b)
    }

    #[inline]
    fn put_u16(&mut self, v: u16) -> io::Result<()> {
        self.w.write_u16::<BigEndian>(v)
    }

    #[inline]
    fn put_u32(&mut self, v: u32) -> io::Result<()> {
        self.w.write_u32::<BigEndian>(v)
    }

    #[inline]
    fn put_u64(&mut self, v: u64) -> io::Result<()> {
        self.w.write_u64::<BigEndian>(v)
    }

    #[inline]
    fn put_i8(&mut self, v: i8) -> io::Result<()> {
        self.w.write_i8(v)
    }

    #[inline]
    fn put_i16(&mut self, v: i16) -> io::Result<()> {
        self.w.write_i16::<BigEndian>(v)
    }

    #[inline]
    fn put_i32(&mut self, v: i32) -> io::Result<()> {
        self.w.write_i32::<BigEndian>(v)
    }

    #[inline]
    fn put_i64(&mut self, v: i64) -> io::Result<()> {
        self.w.write_i64::<BigEndian>(v)
    }

    #[inline]
    fn put_f32(&mut self, v: f32) -> io::Result<()> {
        self.w.write_f32::<BigEndian>(v)
    }

    #[inline]
    fn put_f64(&mut self, v: f64) -> io::Result<()> {
        self.w.write_f64::<BigEndian>(v)
    }

    #[inline]
    fn put_all(&mut self, v: &[u8]) -> io::Result<()> {
        self.w.write_all(v)
    }
}

impl<W: io::Write> ObjectWriter<W> {
    pub fn new(mut w: W) -> io::Result<Self> {
        w.write_u16::<BigEndian>(STREAM_MAGIC)?;
        w.write_u16::<BigEndian>(STREAM_VERSION)?;

        Ok(Self {
            w,
            next_handle: BASE_WIRE_HANDLE,
            string_handles: Default::default(),
            class_handles: Default::default(),
            blkmode: true,
            blk: BlockData::new(),
        })
    }

    /// Runs `f` with a type-erased view of this writer, then takes the mutated stream state back.
    ///
    /// `ObjectWriter<W>` is generic over `W`, so methods taking it cannot be called through a
    /// `dyn` trait object. This hands out a `ObjectWriter<&mut dyn io::Write>` borrowing the same
    /// underlying sink, so handle allocation and the string/class back-reference tables keep
    /// advancing across the call.
    pub fn with_dyn<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ObjectWriter<&mut dyn io::Write>) -> R,
    {
        let mut erased = ObjectWriter {
            next_handle: self.next_handle,
            string_handles: std::mem::take(&mut self.string_handles),
            class_handles: std::mem::take(&mut self.class_handles),
            w: &mut self.w as &mut dyn io::Write,
            blkmode: self.blkmode,
            blk: std::mem::take(&mut self.blk),
        };

        let r = f(&mut erased);

        self.next_handle = erased.next_handle;
        self.string_handles = std::mem::take(&mut erased.string_handles);
        self.class_handles = std::mem::take(&mut erased.class_handles);
        self.blk = std::mem::take(&mut erased.blk);

        r
    }

    #[inline]
    fn alloc_handle(&mut self) -> u32 {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }

    #[inline]
    fn write_reference(&mut self, handle: u32) -> io::Result<()> {
        self.put_u8(TC_REFERENCE)?;
        self.put_u32(handle)?;
        Ok(())
    }

    #[inline]
    pub fn write_char(&mut self, v: u16) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u16::<BigEndian>(v)
        } else {
            self.put_u16(v)
        }
    }
}

pub trait Writer<T> {
    fn write(&mut self, input: T) -> io::Result<()>;
}

impl<W> Writer<()> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: ()) -> io::Result<()> {
        self.put_u8(TC_NULL)
    }
}

impl<W> Writer<bool> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: bool) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u8(input as u8)
        } else {
            self.put_u8(input as u8)
        }
    }
}

impl<W> Writer<char> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: char) -> io::Result<()> {
        let mut buf = [0u16; 2];
        let units = input.encode_utf16(&mut buf);
        if units.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "character {:?} is outside the BMP and cannot be represented by a single Java char",
                    input
                ),
            ));
        }

        if self.blkmode {
            self.blk.write_u16::<BigEndian>(units[0])
        } else {
            self.put_u16(units[0])
        }
    }
}

impl<W> Writer<i8> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: i8) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u8(input as u8)
        } else {
            self.put_u8(input as u8)
        }
    }
}

impl<W> Writer<u8> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: u8) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u8(input)
        } else {
            self.put_u8(input)
        }
    }
}

impl<W> Writer<i16> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: i16) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_i16::<BigEndian>(input)
        } else {
            self.put_i16(input)
        }
    }
}

impl<W> Writer<u16> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: u16) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u16::<BigEndian>(input)
        } else {
            self.put_u16(input)
        }
    }
}

impl<W> Writer<i32> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: i32) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_i32::<BigEndian>(input)
        } else {
            self.put_i32(input)
        }
    }
}

impl<W> Writer<u32> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: u32) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u32::<BigEndian>(input)
        } else {
            self.put_u32(input)
        }
    }
}

impl<W> Writer<i64> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: i64) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_i64::<BigEndian>(input)
        } else {
            self.put_i64(input)
        }
    }
}

impl<W> Writer<u64> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: u64) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u64::<BigEndian>(input)
        } else {
            self.put_u64(input)
        }
    }
}

impl<W> Writer<isize> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: isize) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_i64::<BigEndian>(input as i64)
        } else {
            self.put_i64(input as i64)
        }
    }
}

impl<W> Writer<usize> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: usize) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_u64::<BigEndian>(input as u64)
        } else {
            self.put_u64(input as u64)
        }
    }
}

impl<W> Writer<f32> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: f32) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_f32::<BigEndian>(input)
        } else {
            self.put_f32(input)
        }
    }
}

impl<W> Writer<f64> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: f64) -> io::Result<()> {
        if self.blkmode {
            self.blk.write_f64::<BigEndian>(input)
        } else {
            self.put_f64(input)
        }
    }
}

impl<'a, W> Writer<&'a str> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: &'a str) -> io::Result<()> {
        self.write_string(input)?;
        Ok(())
    }
}

impl<'a, W> Writer<&'a String> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, input: &'a String) -> io::Result<()> {
        self.write_string(input)?;
        Ok(())
    }
}

pub trait ArrayWriter<T> {
    fn write_all(&mut self, v: &[T]) -> io::Result<()>;
}

impl<W> ArrayWriter<bool> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[bool]) -> io::Result<()> {
        self.begin_array(&Class::class_of_boolean_array(), input.len())?;
        for next in input {
            self.write(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<char> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[char]) -> io::Result<()> {
        self.begin_array(&Class::class_of_char_array(), input.len())?;
        for next in input {
            self.write(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<u8> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[u8]) -> io::Result<()> {
        self.begin_array(&Class::class_of_byte_array(), input.len())?;
        for next in input {
            self.put_u8(*next)?;
        }

        Ok(())
    }
}

impl<W> ArrayWriter<i8> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[i8]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_byte_array(), size)?;
        for next in input {
            self.put_u8(*next as u8)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<i16> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[i16]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_short_array(), size)?;
        for next in input {
            self.put_i16(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<u16> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[u16]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_short_array(), size)?;
        for next in input {
            self.put_u16(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<i32> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[i32]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_int_array(), size)?;
        for next in input {
            self.put_i32(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<u32> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[u32]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_int_array(), size)?;
        for next in input {
            self.put_u32(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<i64> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[i64]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_long_array(), size)?;
        for next in input {
            self.put_i64(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<u64> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[u64]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_long_array(), size)?;
        for next in input {
            self.put_u64(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<isize> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[isize]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_long_array(), size)?;
        for next in input {
            self.put_i64(*next as i64)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<usize> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[usize]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_long_array(), size)?;
        for next in input {
            self.put_u64(*next as u64)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<f32> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[f32]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_float_array(), size)?;
        for next in input {
            self.write(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<f64> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[f64]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_double_array(), size)?;
        for next in input {
            self.write(*next)?;
        }
        Ok(())
    }
}

impl<W> ArrayWriter<String> for ObjectWriter<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, input: &[String]) -> io::Result<()> {
        let size = input.len();
        self.begin_array(&Class::class_of_string_array(), size)?;
        for next in input {
            self.write(next)?;
        }
        Ok(())
    }
}

impl<W: io::Write> ObjectWriter<W> {
    #[inline]
    pub(crate) fn write_string(&mut self, s: &str) -> io::Result<u32> {
        self.flush()?;
        if let Some(&h) = self.string_handles.get(s) {
            self.write_reference(h)?;
            debug!("write reference#{}: {}", h - BASE_WIRE_HANDLE, s);
            return Ok(h);
        }
        let (bytes, _chars) = to_modified_utf8(s);
        let len = bytes.len() as u16;
        let handle = self.alloc_handle();

        if len as usize <= u16::MAX as usize {
            self.put_u8(TC_STRING)?;
            self.put_u16(len)?;
        } else {
            self.put_u8(TC_LONGSTRING)?;
            self.put_u64(len as u64)?;
        }
        self.put_all(&bytes)?;
        self.string_handles.insert(s.to_string(), handle);
        Ok(handle)
    }

    #[inline]
    fn write_field_name(&mut self, name: &str) -> io::Result<()> {
        let (bytes, _chars) = to_modified_utf8(name);
        self.put_u16(bytes.len() as u16)?;
        self.put_all(&bytes)?;
        Ok(())
    }

    #[inline]
    fn write_class(&mut self, cd: &Class) -> io::Result<u32> {
        let name = cd.cached_name();
        if let Some(&h) = self.class_handles.get(&name) {
            self.write_reference(h)?;
            return Ok(h);
        }
        self.write_class_full(cd)
    }

    #[inline]
    fn write_class_full(&mut self, cd: &Class) -> io::Result<u32> {
        let handle = self.alloc_handle();
        self.class_handles.insert(cd.cached_name(), handle);

        self.put_u8(TC_CLASSDESC)?;

        let (name_bytes, name_len) = to_modified_utf8(cd.name());

        self.put_u16(name_len)?;
        self.put_all(&name_bytes)?;

        self.put_i64(cd.serial_version_uid())?;
        self.put_u8(cd.flags().bits())?;

        self.put_u16(cd.fields().len() as u16)?;

        for f in cd.fields().iter() {
            let name = f.name();
            match f.kind() {
                FieldKind::Primitive(type_code) => {
                    self.put_u8(*type_code as u8)?;
                    self.write_field_name(name)?;
                }
                FieldKind::Object(class_sig) => {
                    self.put_u8(b'L')?;
                    self.write_field_name(name)?;
                    self.write_string(class_sig)?;
                }
                FieldKind::Array(class_sig) => {
                    self.put_u8(b'[')?;
                    self.write_field_name(name)?;
                    self.write_string(class_sig)?;
                }
            }
        }

        self.put_u8(TC_ENDBLOCKDATA)?; // end of classAnnotation

        match cd.super_class() {
            Some(sup) => {
                self.write_class(sup)?;
            }
            None => self.put_u8(TC_NULL)?,
        }
        Ok(handle)
    }
}

impl<W: io::Write> ObjectWriter<W> {
    #[inline]
    pub fn begin_object(&mut self, class: &Class) -> io::Result<u32> {
        self.flush()?;

        self.put_u8(TC_OBJECT)?;

        self.write_class(class)?;

        Ok(self.alloc_handle())
    }

    #[inline]
    pub fn begin_array(&mut self, class: &Class, size: usize) -> io::Result<u32> {
        self.flush()?;
        self.put_u8(TC_ARRAY)?;
        self.write_class(class)?;
        let handle = self.alloc_handle();
        self.put_u32(size as u32)?;

        Ok(handle)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        if !self.blkmode || self.blk.is_empty() {
            return Ok(());
        }

        // write block data

        let n = self.blk.len();
        if n <= 0xff {
            self.put_u8(TC_BLOCKDATA)?;
            self.put_u8(n as u8)?;
        } else {
            self.put_u8(TC_BLOCKDATALONG)?;
            self.put_u32(n as u32)?;
        }

        self.w.write_all(&self.blk)?;

        self.blk.clear();

        Ok(())
    }

    #[inline]
    pub(crate) fn end(&mut self) -> io::Result<()> {
        self.flush()?;
        self.put_u8(TC_ENDBLOCKDATA)
    }

    pub(crate) fn set_block_data_mode(&mut self, enabled: bool) {
        self.blkmode = enabled;
    }
}
