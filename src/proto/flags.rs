// Stream header
pub(crate) const STREAM_MAGIC: u16 = 0xaced;
pub(crate) const STREAM_VERSION: u16 = 0x0005;

// TC_* type code
pub(crate) const TC_NULL: u8 = 0x70;
pub(crate) const TC_REFERENCE: u8 = 0x71;
pub(crate) const TC_CLASSDESC: u8 = 0x72;
pub(crate) const TC_OBJECT: u8 = 0x73;
pub(crate) const TC_STRING: u8 = 0x74;
pub(crate) const TC_ARRAY: u8 = 0x75;
pub(crate) const TC_CLASS: u8 = 0x76;
pub(crate) const TC_BLOCKDATA: u8 = 0x77;
pub(crate) const TC_ENDBLOCKDATA: u8 = 0x78;
pub(crate) const TC_RESET: u8 = 0x79;
pub(crate) const TC_BLOCKDATALONG: u8 = 0x7a;
pub(crate) const TC_EXCEPTION: u8 = 0x7b;
pub(crate) const TC_LONGSTRING: u8 = 0x7c;
pub(crate) const TC_PROXYCLASSDESC: u8 = 0x7d;
pub(crate) const TC_ENUM: u8 = 0x7e;
pub(crate) const TC_MAX: u8 = 0x7e;
pub(crate) const TC_NULLREF: u8 = 0x70; // alias

pub(crate) const BASE_WIRE_HANDLE: u32 = 0x7e0000;
