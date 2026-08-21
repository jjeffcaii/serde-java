use crate::util::to_modified_utf8;

pub const PUBLIC: i32 = 0x0001;
pub const PRIVATE: i32 = 0x0002;
pub const PROTECTED: i32 = 0x0004;
pub const STATIC: i32 = 0x0008;
pub const FINAL: i32 = 0x0010;
pub const SYNCHRONIZED: i32 = 0x0020;
pub const VOLATILE: i32 = 0x0040;
pub const TRANSIENT: i32 = 0x0080;
pub const NATIVE: i32 = 0x0100;
pub const INTERFACE: i32 = 0x0200;
pub const ABSTRACT: i32 = 0x0400;
pub const STRICT: i32 = 0x0800;

pub struct FieldSig<'a> {
    pub name: &'a str,
    pub modifiers: i32,    // raw modifiers; masking happens inside the function
    pub type_sig: &'a str, // type signature, e.g. "Ljava/lang/String;" / "I"
}

pub struct MethodSig<'a> {
    pub name: &'a str,
    pub modifiers: i32,
    pub descriptor: &'a str, // JVM method descriptor, e.g. "(Ljava/lang/String;)V"
}

pub struct ClassMetadata<'a> {
    pub class_name: &'a str, // fully-qualified name, e.g. "com.example.ExtInfo"
    pub class_modifiers: i32,
    pub is_interface: bool,
    pub interfaces: Vec<&'a str>, // fully-qualified names of directly implemented interfaces
    pub fields: Vec<FieldSig<'a>>, // every declared field (static/transient included; filtered inside)
    pub has_static_initializer: bool, // whether a <clinit> exists (static block, or static field initializer)
    pub constructors: Vec<MethodSig<'a>>, // name is always "<init>"; descriptor is required
    pub methods: Vec<MethodSig<'a>>,  // constructors excluded
}

use sha1::{Digest, Sha1};

pub fn compute_default_suid(meta: &ClassMetadata) -> i64 {
    let mut buf: Vec<u8> = Vec::new();

    // 1. class name
    write_utf(&mut buf, meta.class_name);

    // 2. class modifiers (only PUBLIC/FINAL/INTERFACE/ABSTRACT survive)
    let mut class_mods = meta.class_modifiers & (PUBLIC | FINAL | INTERFACE | ABSTRACT);
    if meta.is_interface {
        class_mods = if !meta.methods.is_empty() {
            class_mods | ABSTRACT
        } else {
            class_mods & !ABSTRACT
        };
    }
    buf.extend_from_slice(&class_mods.to_be_bytes());

    // 3. interface names, sorted alphabetically
    let mut ifaces = meta.interfaces.clone();
    ifaces.sort();
    for name in &ifaces {
        write_utf(&mut buf, name);
    }

    // 4. fields: drop (private && (static || transient)) ones, sort by name
    let mut fields: Vec<&FieldSig> = meta
        .fields
        .iter()
        .filter(|f| {
            let mods = f.modifiers
                & (PUBLIC | PRIVATE | PROTECTED | STATIC | FINAL | VOLATILE | TRANSIENT);
            (mods & PRIVATE) == 0 || (mods & (STATIC | TRANSIENT)) == 0
        })
        .collect();
    fields.sort_by(|a, b| a.name.cmp(b.name));
    for f in fields {
        let mods =
            f.modifiers & (PUBLIC | PRIVATE | PROTECTED | STATIC | FINAL | VOLATILE | TRANSIENT);
        write_utf(&mut buf, f.name);
        buf.extend_from_slice(&mods.to_be_bytes());
        write_utf(&mut buf, f.type_sig);
    }

    // 5. static initializer
    if meta.has_static_initializer {
        write_utf(&mut buf, "<clinit>");
        buf.extend_from_slice(&STATIC.to_be_bytes());
        write_utf(&mut buf, "()V");
    }

    // 6. constructors: drop private ones, sort by descriptor
    let cons_mask =
        PUBLIC | PRIVATE | PROTECTED | STATIC | FINAL | SYNCHRONIZED | NATIVE | ABSTRACT | STRICT;
    let mut cons: Vec<&MethodSig> = meta
        .constructors
        .iter()
        .filter(|c| (c.modifiers & PRIVATE) == 0)
        .collect();
    cons.sort_by(|a, b| a.descriptor.cmp(b.descriptor));
    for c in cons {
        let mods = c.modifiers & cons_mask;
        write_utf(&mut buf, "<init>");
        buf.extend_from_slice(&mods.to_be_bytes());
        write_utf(&mut buf, &c.descriptor.replace('/', "."));
    }

    // 7. ordinary methods: drop private ones, sort by (name, descriptor)
    let mut methods: Vec<&MethodSig> = meta
        .methods
        .iter()
        .filter(|m| (m.modifiers & PRIVATE) == 0)
        .collect();
    methods.sort_by(|a, b| a.name.cmp(b.name).then(a.descriptor.cmp(b.descriptor)));
    for m in methods {
        let mods = m.modifiers & cons_mask;
        write_utf(&mut buf, m.name);
        buf.extend_from_slice(&mods.to_be_bytes());
        write_utf(&mut buf, &m.descriptor.replace('/', "."));
    }

    // 8. SHA-1 digest; fold the first 8 bytes little-endian into an i64
    let digest = Sha1::digest(&buf);
    let mut hash: i64 = 0;
    for i in (0..8).rev() {
        hash = (hash << 8) | (digest[i] as i64);
    }
    hash
}

/// Mirrors DataOutputStream.writeUTF: a 2-byte length (in modified-UTF-8 bytes), then the content.
fn write_utf(buf: &mut Vec<u8>, s: &str) {
    let (bytes, len) = to_modified_utf8(s); // reuses the crate's modified-UTF-8 encoder
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        pretty_env_logger::try_init_timed().ok();
    }

    #[test]
    fn test_suid() {
        init();

        let meta = ClassMetadata {
            class_name: "com.example.ExtInfo",
            class_modifiers: PUBLIC,
            is_interface: false,
            interfaces: vec!["java.io.Serializable"],
            fields: vec![
                FieldSig {
                    name: "id",
                    modifiers: PRIVATE,
                    type_sig: "I",
                },
                FieldSig {
                    name: "key",
                    modifiers: PRIVATE,
                    type_sig: "Ljava/lang/String;",
                },
                FieldSig {
                    name: "value",
                    modifiers: PRIVATE,
                    type_sig: "Ljava/lang/String;",
                },
            ],
            has_static_initializer: false,
            constructors: vec![
                MethodSig {
                    name: "<init>",
                    modifiers: PUBLIC,
                    descriptor: "()V",
                },
                MethodSig {
                    name: "<init>",
                    modifiers: PUBLIC,
                    descriptor: "(ILjava/lang/String;Ljava/lang/String;)V",
                },
            ],
            methods: vec![
                // getters/setters/equals/hashCode/toString/canEqual generated by Lombok @Data
                MethodSig {
                    name: "getId",
                    modifiers: PUBLIC,
                    descriptor: "()I",
                },
                MethodSig {
                    name: "getKey",
                    modifiers: PUBLIC,
                    descriptor: "()Ljava/lang/String;",
                },
                MethodSig {
                    name: "getValue",
                    modifiers: PUBLIC,
                    descriptor: "()Ljava/lang/String;",
                },
                // ... the remaining methods, listed as declared
            ],
        };

        info!("generated SUID: {}", compute_default_suid(&meta));

        let suid = compute_default_suid(&meta);
        assert_eq!(650544313874690833, suid);
    }
}
