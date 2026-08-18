mod attr;
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
/// }
/// ```
#[proc_macro_derive(JavaSerialize, attributes(java))]
pub fn derive_java_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
