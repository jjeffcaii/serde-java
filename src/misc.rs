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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_signature() {
        let sig = to_signature("java.lang.String");
        assert_eq!("Ljava/lang/String;", sig.as_ref());
    }
}
