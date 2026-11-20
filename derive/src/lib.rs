mod attr;
mod deserialize;
mod expand;
mod ty;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives `JavaObject` and `JavaSerializable` for a struct with named fields.
///
/// ```ignore
/// #[derive(JavaSerialize)]
/// #[java(class = "com.example.User", serial_version_uid = 1234)]
/// struct User {
///     id: i32,
///     #[java(rename = "address")]
///     address_alias: Address,
///     #[java(skip)]
///     cache: HashMap<String, String>,
///     // Value written by the `Layout` impl instead of the built-in type mapping;
///     // `signature` is required because it cannot be derived from a `Layout`.
///     #[java(signature = "Ljava/util/List;", with = "serde_java_ext::ArrayList")]
///     tags: Vec<String>,
/// }
/// ```
#[proc_macro_derive(JavaSerialize, attributes(java))]
pub fn derive_java_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives field and object deserialization for an owned, named-field struct.
/// Requires `JavaObject`, supplied by `JavaSerialize` or a manual implementation.
/// Skipped fields are initialized with `Default`; Layout adapters are unsupported.
#[proc_macro_derive(JavaDeserialize, attributes(java))]
pub fn derive_java_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    deserialize::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
