extern crate string_cache_codegen;

use std::env;
use std::path::Path;

fn main() {
    string_cache_codegen::AtomType::new("astr::AtomString", "astr!")
        .atoms(&[
            // COMMON FIELDS
            "id",
            "[B",
            "Ljava/lang/String;",
            "Ljava/io/Serializable;",
        ])
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("astr.rs"))
        .unwrap();
}
