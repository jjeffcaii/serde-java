use super::{Class, FieldKind};
use crate::astr::AtomString;
use crate::util::to_modified_utf8;
use byteorder::{BigEndian, WriteBytesExt};
use hashbrown::HashMap;
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

pub struct ObjectWriter<W> {
    w: W,
    next_handle: u32,
    string_handles: HashMap<String, u32>,
    class_handles: HashMap<AtomString, u32>,
    blkmode: bool,
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
        };

        let r = f(&mut erased);

        self.next_handle = erased.next_handle;
        self.string_handles = std::mem::take(&mut erased.string_handles);
        self.class_handles = std::mem::take(&mut erased.class_handles);

        r
    }

    #[inline]
    fn alloc_handle(&mut self) -> u32 {
        let h = self.next_handle;
        self.next_handle += 1;
        h
    }

    #[inline]
    pub fn write_null(&mut self) -> io::Result<()> {
        self.put_u8(TC_NULL)
    }

    #[inline]
    pub fn write_reference(&mut self, handle: u32) -> io::Result<()> {
        self.put_u8(TC_REFERENCE)?;
        self.put_u32(handle)?;
        Ok(())
    }

    #[inline]
    pub fn write_byte(&mut self, v: u8) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(1)?;
        }
        self.put_u8(v)
    }

    #[inline]
    pub fn write_bool(&mut self, v: bool) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(1)?;
        }
        self.put_u8(v as u8)
    }

    #[inline]
    pub fn write_char(&mut self, v: u16) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(2)?;
        }
        self.put_u16(v)
    }

    #[inline]
    pub fn write_short(&mut self, v: i16) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(2)?;
        }
        self.put_i16(v)
    }

    #[inline]
    pub fn write_int(&mut self, v: i32) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(4)?;
        }
        self.put_i32(v)
    }

    #[inline]
    pub fn write_long(&mut self, v: i64) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(8)?;
        }
        self.put_i64(v)
    }

    #[inline]
    pub fn write_float(&mut self, v: f32) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(4)?;
        }
        self.put_f32(v)
    }

    #[inline]
    pub fn write_double(&mut self, v: f64) -> io::Result<()> {
        if self.blkmode {
            self.begin_block_data(8)?;
        }
        self.put_f64(v)
    }
}

impl<W: io::Write> ObjectWriter<W> {
    #[inline]
    pub fn write_string(&mut self, s: &str) -> io::Result<u32> {
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
    pub fn write_boolean_array(&mut self, data: &[bool]) -> io::Result<u32> {
        let class = Class::class_of_boolean_array();
        self.write_primitive_array(&class, data, |w, it| w.write_bool(*it))
    }

    #[inline]
    pub fn write_byte_array(&mut self, data: &[u8]) -> io::Result<u32> {
        let class = Class::class_of_byte_array();
        self.write_primitive_array(&class, data, |w, it| w.write_byte(*it))
    }

    pub fn write_i8_array(&mut self, data: &[i8]) -> io::Result<u32> {
        let class = Class::class_of_byte_array();
        self.write_primitive_array(&class, data, |w, it| w.write_byte(*it as u8))
    }

    #[inline]
    pub fn write_short_array(&mut self, data: &[i16]) -> io::Result<u32> {
        let class = Class::class_of_short_array();
        self.write_primitive_array(&class, data, |w, it| w.write_short(*it))
    }

    #[inline]
    pub fn write_u16_array(&mut self, data: &[u16]) -> io::Result<u32> {
        let class = Class::class_of_short_array();
        self.write_primitive_array(&class, data, |w, it| w.write_short(*it as i16))
    }

    #[inline]
    pub fn write_int_array(&mut self, data: &[i32]) -> io::Result<u32> {
        let class = Class::class_of_int_array();
        self.write_primitive_array(&class, data, |w, it| w.write_int(*it))
    }

    #[inline]
    pub fn write_u32_array(&mut self, data: &[u32]) -> io::Result<u32> {
        let class = Class::class_of_int_array();
        self.write_primitive_array(&class, data, |w, it| w.write_int(*it as i32))
    }

    #[inline]
    pub fn write_long_array(&mut self, data: &[i64]) -> io::Result<u32> {
        let class = Class::class_of_long_array();
        self.write_primitive_array(&class, data, |w, it| w.write_long(*it))
    }

    #[inline]
    pub fn write_u64_array(&mut self, data: &[u64]) -> io::Result<u32> {
        let class = Class::class_of_long_array();
        self.write_primitive_array(&class, data, |w, it| w.write_long(*it as i64))
    }

    #[inline]
    pub fn write_float_array(&mut self, data: &[f32]) -> io::Result<u32> {
        let class = Class::class_of_float_array();
        self.write_primitive_array(&class, data, |w, it| w.write_float(*it))
    }

    #[inline]
    pub fn write_double_array(&mut self, data: &[f64]) -> io::Result<u32> {
        let class = Class::class_of_double_array();
        self.write_primitive_array(&class, data, |w, it| w.write_double(*it))
    }

    #[inline]
    pub fn write_string_array(&mut self, data: &[String]) -> io::Result<u32> {
        let size = data.len();
        let h = self.begin_array(&Class::class_of_string_array(), size)?;

        for next in data {
            self.write_string(next)?;
        }
        Ok(h)
    }

    pub fn begin_array(&mut self, class: &Class, size: usize) -> io::Result<u32> {
        self.put_u8(TC_ARRAY)?;
        self.write_class(class)?;
        let handle = self.alloc_handle();
        self.put_u32(size as u32)?;

        Ok(handle)
    }

    #[inline]
    fn write_primitive_array<T>(
        &mut self,
        class: &Class,
        items: &[T],
        write_elem: impl Fn(&mut Self, &T) -> io::Result<()>,
    ) -> io::Result<u32> {
        self.put_u8(TC_ARRAY)?;
        self.write_class(class)?;
        let handle = self.alloc_handle();
        self.put_u32(items.len() as u32)?;
        for item in items {
            write_elem(self, item)?;
        }
        Ok(handle)
    }

    #[inline]
    pub fn write_class(&mut self, cd: &Class) -> io::Result<u32> {
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
            None => self.write_null()?,
        }
        Ok(handle)
    }
}

impl<W: io::Write> ObjectWriter<W> {
    #[inline]
    pub fn begin_object(&mut self, class: &Class) -> io::Result<u32> {
        self.put_u8(TC_OBJECT)?;

        self.write_class(class)?;

        let h = self.alloc_handle();

        // for v in values {
        //     match v {
        //         FieldValue::Byte(x) => self.write_byte(*x)?,
        //         FieldValue::Bool(x) => self.write_bool(*x)?,
        //         FieldValue::Char(x) => self.write_char(*x)?,
        //         FieldValue::Short(x) => self.write_short(*x)?,
        //         FieldValue::Int(x) => self.write_int(*x)?,
        //         FieldValue::Long(x) => self.write_long(*x)?,
        //         FieldValue::Float(x) => self.write_float(*x)?,
        //         FieldValue::Double(x) => self.write_double(*x)?,
        //         FieldValue::String(s) => {
        //             self.write_string(s)?;
        //         }
        //         FieldValue::Object(class, obj) => {
        //             // self.write_object(class, &obj.fields())?;
        //             unreachable!()
        //         }
        //         FieldValue::Null => self.write_null()?,
        //         FieldValue::BoolArray(_) => {
        //             todo!("write boolean array!")
        //         }
        //         FieldValue::CharArray(_) => {
        //             todo!("write character array!")
        //         }
        //         FieldValue::ByteArray(x) => {
        //             self.write_byte_array(*x)?;
        //         }
        //         FieldValue::ShortArray(x) => {
        //             self.write_short_array(*x)?;
        //         }
        //         FieldValue::IntArray(x) => {
        //             self.write_int_array(*x)?;
        //         }
        //         FieldValue::LongArray(x) => {
        //             self.write_long_array(*x)?;
        //         }
        //         FieldValue::FloatArray(x) => {
        //             self.write_float_array(*x)?;
        //         }
        //         FieldValue::DoubleArray(x) => {
        //             self.write_double_array(*x)?;
        //         }
        //         FieldValue::StringArray(x) => {
        //             todo!("write string array!")
        //         }
        //         FieldValue::Array(class_of_array, x) => {
        //             // info!(
        //             //     "begin write array: class={}, len={}",
        //             //     class_of_array.signature(),
        //             //     x.len()
        //             // );
        //             //
        //             // self.write_primitive_array(class_of_array, x, |w, (class_of_item, it)| {
        //             //     let next_fields = it.fields();
        //             //     w.write_object(class_of_item, &next_fields)?;
        //             //     Ok(())
        //             // })?;
        //             unreachable!()
        //         }
        //     }
        // }
        Ok(h)
    }

    #[inline]
    fn begin_block_data(&mut self, n: usize) -> io::Result<()> {
        if n <= 0xff {
            self.put_u8(TC_BLOCKDATA)?;
            self.put_u8(n as u8)?;
        } else {
            self.put_u8(TC_BLOCKDATALONG)?;
            self.put_u32(n as u32)?;
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn end_block_data(&mut self) -> io::Result<()> {
        self.put_u8(TC_ENDBLOCKDATA)
    }

    #[inline]
    pub fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }

    pub(crate) fn set_block_data_mode(&mut self, enabled: bool) {
        self.blkmode = enabled;
    }
}
