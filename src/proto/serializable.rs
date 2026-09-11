use super::JavaObject;
use super::class::Class;
use super::object::ObjectView;
use super::writer::{ArrayWriter, ObjectWriter, Writer};
use std::io;

/// Defines how a type writes its own field values into the serialization stream — the
/// counterpart of Java's `Serializable` interface plus an optional custom
/// `writeObject(ObjectOutputStream)` method.
///
/// This trait only cares about "how the instance's field values are written", not "how the
/// object header/class descriptor is written" — that's [`JavaObject`] and
/// [`crate::object::Object`]'s job. The three methods layer on top of each other, from lowest to
/// highest:
///
/// - [`write_fields`](JavaSerializable::write_fields): the lowest level — writes each field's
///   value in the order declared by `Class`, equivalent to the field data Java gets via
///   `ObjectOutputStream.PutField`. **Must** be hand-written, and its order/types must match
///   `class()`'s field list exactly; there is currently no compile-time or runtime check, so
///   getting it wrong either makes the JVM reject the stream on deserialization or silently
///   misaligns the fields.
/// - [`default_write_object`](JavaSerializable::default_write_object): equivalent to calling
///   `ObjectOutputStream#defaultWriteObject()` in Java — temporarily turns off block-data mode,
///   calls `write_fields` to write the fields as-is, then restores block-data mode. Rarely called
///   directly; `write_object`'s default implementation already calls it.
/// - [`write_object`](JavaSerializable::write_object): the outermost level, corresponding to an
///   optional custom `private void writeObject(ObjectOutputStream out)` method in Java. The
///   default implementation is `default_write_object` (i.e. "no custom `writeObject`, use default
///   serialization"); override it when the Java class actually defines a custom `writeObject` —
///   e.g. calling `defaultWriteObject()` and then manually appending extra objects (this is how
///   `java.lang.Throwable`'s `stackTrace`/`suppressedExceptions` are handled — see the
///   implementation in `serde-java-ext`). When overriding, the corresponding `Class` must also
///   carry the `ClassFlags::WRITE_METHOD` flag so a `TC_ENDBLOCKDATA` is appended after the
///   fields.
///
/// # Example: default serialization (no `write_object` override)
///
/// ```
/// use once_cell::sync::Lazy;
/// use serde_java::*;
/// use std::io;
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl JavaObject for Point {
///     fn class() -> Class {
///         static CLASS: Lazy<Class> = Lazy::new(|| {
///             Class::builder("com.example.Point", 1)
///                 .field(Field::builder("x").int())
///                 .field(Field::builder("y").int())
///                 .build()
///         });
///         Clone::clone(&CLASS)
///     }
/// }
///
/// impl JavaSerializable for Point {
///     // Only this method is needed: write values in class()'s field order
///     // (x before y, since fields are sorted alphabetically).
///     fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
///         self.x.write_to(w)?;
///         self.y.write_to(w)?;
///         Ok(())
///     }
///     // write_object/default_write_object use the trait's default impl, no override needed.
/// }
/// ```
pub trait JavaSerializable {
    /// Writes each field's value, in the order declared by `class()`, equivalent to
    /// `java.io.ObjectOutputStream#defaultWriteFields`.
    ///
    /// Field ordering rule: all primitive fields first, then all object/array fields, each group
    /// sorted alphabetically by name (not the Rust struct's declaration order!). Primitives and
    /// strings are written with `x.write_to(w)`, nested objects likewise with `x.write_to(w)`
    /// (which recurses into the nested object), and arrays/`Vec`s the same way — see the various
    /// `JavaWriteable` impls for the exact forms.
    fn write_fields(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()>;

    /// Runs the "default serialization" logic, equivalent to
    /// `java.io.ObjectOutputStream#defaultWriteObject`: temporarily switches out of block-data
    /// mode, calls [`write_fields`](Self::write_fields) to write the field values, then restores
    /// block-data mode.
    ///
    /// Rarely called directly — except from within a custom
    /// [`write_object`](Self::write_object) that wants to write the default fields first and then
    /// manually append extra data (mirroring Java's pattern of calling `defaultWriteObject()`
    /// before writing anything else).
    fn default_write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let prev = w.set_block_data_mode(false);
        self.write_fields(w)?;
        w.set_block_data_mode(prev);
        Ok(())
    }

    /// Custom serialization entry point, equivalent to a Java class's own
    /// `private void writeObject(ObjectOutputStream out)`.
    ///
    /// The default implementation simply forwards to
    /// [`default_write_object`](Self::default_write_object), i.e. "no custom logic, write the
    /// declared fields as-is". Only override this when the Java class actually defines a custom
    /// `writeObject` — e.g. call `self.default_write_object(w)?` to write the regular fields
    /// first, then manually write extra objects or data. When overridden, remember to add the
    /// `ClassFlags::WRITE_METHOD` flag to the corresponding `Class`, so
    /// [`crate::object::Object::write_to`] appends a `TC_ENDBLOCKDATA` after the fields — without
    /// it the JVM won't be able to parse the resulting stream.
    fn write_object(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        self.default_write_object(w)
    }

    /// Assigns this instance a "virtual pointer" identity, used to share the same object
    /// reference (`TC_REFERENCE`) across multiple writes.
    ///
    /// Background: `ObjectWriter` itself does no object-identity deduplication — if the same Rust
    /// value is written to the stream twice, it gets two distinct handles by default and
    /// serializes as two separate Java objects (see the module docs, "Handles are allocated but
    /// never reused"). Some scenarios need multiple writes to resolve to the *same* object
    /// instead — e.g. a value that only ever appears as a singleton, or a field the Java side
    /// compares with `==` and expects to be the identical reference.
    ///
    /// When this returns `Some(ptr)`, `ptr` is treated as a stable key: the first time a given
    /// key is seen, the object is written normally and its wire handle is recorded; every
    /// subsequent write with the same key writes a `TC_REFERENCE` back to that handle instead of
    /// writing the object data again. The default returns `None`, meaning "always a distinct
    /// object, no reference deduplication".
    ///
    /// `ptr` is just an identity key — common choices are the instance's memory address
    /// (`self as *const Self as usize`) or some business-stable id.
    fn virtual_ptr(&self) -> Option<usize> {
        None
    }
}

/// JavaWriteable extends more features.
pub trait JavaWriteable {
    /// serialize object to writer.
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()>;
}

pub trait JavaWriteableExt {
    /// serialize object to bytes.
    fn to_bytes(&self) -> io::Result<Vec<u8>>;

    /// Serialize object then write to file.
    fn to_file<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<std::path::Path>;
}

impl<T: JavaWriteable> JavaWriteableExt for T {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut b = Vec::<u8>::new();
        let mut w = ObjectWriter::new(&mut b)?;
        w.with_dyn(|w| self.write_to(w))?;

        Ok(b)
    }

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

impl JavaWriteable for () {
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        w.write(())
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

macro_rules! impl_writeable_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl JavaWriteable for $ty {
                #[inline]
                fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
                    w.write(*self)
                }
            }
        )*
    };
}

impl_writeable_scalar!(
    bool, char, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64
);

macro_rules! impl_writeable_slice {
    ($($ty:ty),* $(,)?) => {
        $(
            impl JavaWriteable for [$ty] {
                #[inline]
                fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
                    w.write_all(self)
                }
            }
        )*
    };
}

impl_writeable_slice!(
    bool, char, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64, String
);

impl<T> JavaWriteable for T
where
    T: JavaSerializable + JavaObject,
{
    #[inline]
    fn write_to(&self, w: &mut ObjectWriter<&mut dyn io::Write>) -> io::Result<()> {
        let class = Self::class();
        let obj = {
            let mut bu = ObjectView::<T, ()>::builder(class, self);
            if let Some(ptr) = self.virtual_ptr() {
                bu = bu.key(ptr);
            }
            bu.build()
        };

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
