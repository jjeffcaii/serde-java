use crate::misc::to_modified_utf8;

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
    pub modifiers: i32,    // 原始修饰符，函数内部会做掩码过滤
    pub type_sig: &'a str, // 类型签名，如 "Ljava/lang/String;" / "I"
}

pub struct MethodSig<'a> {
    pub name: &'a str,
    pub modifiers: i32,
    pub descriptor: &'a str, // JVM 方法描述符，如 "(Ljava/lang/String;)V"
}

pub struct ClassMetadata<'a> {
    pub class_name: &'a str, // 全限定名，如 "com.example.ExtInfo"
    pub class_modifiers: i32,
    pub is_interface: bool,
    pub interfaces: Vec<&'a str>,         // 直接实现的接口全限定名
    pub fields: Vec<FieldSig<'a>>,        // 声明的所有字段（含 static/transient，函数内部过滤）
    pub has_static_initializer: bool,     // 是否有 <clinit>（静态代码块 or 静态字段带初始值）
    pub constructors: Vec<MethodSig<'a>>, // name 固定用 "<init>"，descriptor 必填
    pub methods: Vec<MethodSig<'a>>,      // 不含构造函数
}

use sha1::{Digest, Sha1};

pub fn compute_default_suid(meta: &ClassMetadata) -> i64 {
    let mut buf: Vec<u8> = Vec::new();

    // 1. class name
    write_utf(&mut buf, meta.class_name);

    // 2. class modifiers（只保留 PUBLIC/FINAL/INTERFACE/ABSTRACT）
    let mut class_mods = meta.class_modifiers & (PUBLIC | FINAL | INTERFACE | ABSTRACT);
    if meta.is_interface {
        class_mods = if !meta.methods.is_empty() {
            class_mods | ABSTRACT
        } else {
            class_mods & !ABSTRACT
        };
    }
    buf.extend_from_slice(&(class_mods as i32).to_be_bytes());

    // 3. 接口名，按字母排序
    let mut ifaces = meta.interfaces.clone();
    ifaces.sort();
    for name in &ifaces {
        write_utf(&mut buf, name);
    }

    // 4. 字段：过滤掉 (private && (static || transient)) 的字段，按 name 排序
    let mut fields: Vec<&FieldSig> = meta
        .fields
        .iter()
        .filter(|f| {
            let mods = f.modifiers
                & (PUBLIC | PRIVATE | PROTECTED | STATIC | FINAL | VOLATILE | TRANSIENT);
            (mods & PRIVATE) == 0 || (mods & (STATIC | TRANSIENT)) == 0
        })
        .collect();
    fields.sort_by(|a, b| a.name.cmp(&b.name));
    for f in fields {
        let mods =
            f.modifiers & (PUBLIC | PRIVATE | PROTECTED | STATIC | FINAL | VOLATILE | TRANSIENT);
        write_utf(&mut buf, f.name);
        buf.extend_from_slice(&(mods as i32).to_be_bytes());
        write_utf(&mut buf, f.type_sig);
    }

    // 5. 静态初始化块
    if meta.has_static_initializer {
        write_utf(&mut buf, "<clinit>");
        buf.extend_from_slice(&(STATIC as i32).to_be_bytes());
        write_utf(&mut buf, "()V");
    }

    // 6. 构造函数：过滤 private，按 descriptor 排序
    let cons_mask =
        PUBLIC | PRIVATE | PROTECTED | STATIC | FINAL | SYNCHRONIZED | NATIVE | ABSTRACT | STRICT;
    let mut cons: Vec<&MethodSig> = meta
        .constructors
        .iter()
        .filter(|c| (c.modifiers & PRIVATE) == 0)
        .collect();
    cons.sort_by(|a, b| a.descriptor.cmp(&b.descriptor));
    for c in cons {
        let mods = c.modifiers & cons_mask;
        write_utf(&mut buf, "<init>");
        buf.extend_from_slice(&(mods as i32).to_be_bytes());
        write_utf(&mut buf, &c.descriptor.replace('/', "."));
    }

    // 7. 普通方法：过滤 private，按 (name, descriptor) 排序
    let mut methods: Vec<&MethodSig> = meta
        .methods
        .iter()
        .filter(|m| (m.modifiers & PRIVATE) == 0)
        .collect();
    methods.sort_by(|a, b| a.name.cmp(&b.name).then(a.descriptor.cmp(&b.descriptor)));
    for m in methods {
        let mods = m.modifiers & cons_mask;
        write_utf(&mut buf, m.name);
        buf.extend_from_slice(&(mods as i32).to_be_bytes());
        write_utf(&mut buf, &m.descriptor.replace('/', "."));
    }

    // 8. SHA-1 摘要，取前 8 字节按小端序拼成 i64
    let digest = Sha1::digest(&buf);
    let mut hash: i64 = 0;
    for i in (0..8).rev() {
        hash = (hash << 8) | (digest[i] as i64);
    }
    hash
}

/// 对应 DataOutputStream.writeUTF：先写 2 字节长度（modified-UTF-8 字节数），再写内容
fn write_utf(buf: &mut Vec<u8>, s: &str) {
    let (bytes, len) = to_modified_utf8(s); // 复用之前实现的 modified-UTF-8 编码函数
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
                // Lombok @Data 生成的 getter/setter/equals/hashCode/toString/canEqual...
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
                // ... 其余方法照实填
            ],
        };

        info!("generated SUID: {}", compute_default_suid(&meta));

        let suid = compute_default_suid(&meta);
        assert_eq!(650544313874690833, suid);
    }
}
