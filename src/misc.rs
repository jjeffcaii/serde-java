use crate::astr::AtomString;
use smallvec::SmallVec;

pub(crate) fn to_signature(name: &str) -> AtomString {
    let mut v = SmallVec::<[u8; 128]>::with_capacity(name.len() + 2);
    v.push(b'L');
    for next in name.bytes() {
        match next {
            b'.' => v.push(b'/'),
            other => v.push(other),
        }
    }
    v.push(b';');

    let sig = unsafe { std::str::from_utf8_unchecked(&v) };
    AtomString::from(sig)
}

const N: usize = 32;

/// 编码为 Java modified-UTF-8，返回 (字节数据, utf16_code_unit_count)
pub fn to_modified_utf8(s: &str) -> (SmallVec<[u8; N]>, u16) {
    let mut out = SmallVec::<[u8; N]>::with_capacity(s.len());
    let mut count: u16 = 0;

    for ch in s.chars() {
        let mut buf = [0u16; 2];
        let units = ch.encode_utf16(&mut buf);
        for &unit in units.iter() {
            count += 1;
            match unit {
                0x0001..=0x007F => out.push(unit as u8),
                0 | 0x0080..=0x07FF => {
                    out.push(0xC0 | ((unit >> 6) as u8 & 0x1F));
                    out.push(0x80 | (unit as u8 & 0x3F));
                }
                _ => {
                    out.push(0xE0 | ((unit >> 12) as u8 & 0x0F));
                    out.push(0x80 | ((unit >> 6) as u8 & 0x3F));
                    out.push(0x80 | (unit as u8 & 0x3F));
                }
            }
        }
    }
    (out, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_signature() {
        let sig = to_signature("java.lang.String");
        assert_eq!("Ljava/lang/String;", sig.as_ref());
    }
}
