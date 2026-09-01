extern crate string_cache_codegen;

use std::env;
use std::path::Path;

fn main() {
    string_cache_codegen::AtomType::new("astr::AtomString", "astr!")
        .atoms(&[
            // class names
            "java.lang.Object",
            "java.lang.Boolean",
            "java.lang.Byte",
            "java.lang.Short",
            "java.lang.Integer",
            "java.lang.Long",
            "java.lang.Float",
            "java.lang.Double",
            "java.lang.String",
            "java.util.ArrayList",
            "java.util.LinkedList",
            // class signatures
            "[B",
            "[C",
            "[Z",
            "[S",
            "[I",
            "[J",
            "[D",
            "[F",
            "[Ljava/lang/String;",
            "Ljava/lang/Object;",
            "Ljava/lang/Boolean;",
            "Ljava/lang/Byte;",
            "Ljava/lang/Short;",
            "Ljava/lang/Integer;",
            "Ljava/lang/Long;",
            "Ljava/lang/Float;",
            "Ljava/lang/Double;",
            "Ljava/lang/String;",
            "Ljava/io/Serializable;",
            "Ljava/util/List;",
            "Ljava/util/ArrayList;",
            "Ljava/util/LinkedList;",
        ])
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("astr.rs"))
        .unwrap();
}
