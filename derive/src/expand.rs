use crate::attr;
use crate::ty::{self, JavaTy};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataStruct, DeriveInput, Fields, GenericParam, Path, Type};

struct Resolved {
    java_name: String,
    access: TokenStream,
    ser: FieldSer,
    /// The `FieldBuilder` call chained onto `Field::builder(name)`.
    method: TokenStream,
    is_primitive: bool,
    span: Span,
}

/// How a field's *value* is written.
enum FieldSer {
    /// Straight through the Rust -> Java mapping table.
    Mapped(JavaTy),
    /// Through `#[java(with = "...")]`'s `Layout` impl. `is_option` means the field's Rust
    /// type is `Option<T>`, where `T` (not `Option<T>`) is the `Layout`'s `Input`: `None`
    /// writes `null`, `Some(v)` goes through `Layout::layout(v)` as usual.
    With { path: Path, is_option: bool },
}

/// If `ty` is `Option<T>`, returns `T`. Used only to decide null-handling for `with` fields —
/// `T` itself is never resolved against the mapping table, since a `with` field's Rust type
/// only has to match the `Layout`'s `Input`.
fn peel_option(ty: &Type) -> Option<&Type> {
    let Type::Path(p) = ty else { return None };
    let seg = p.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    ty::single_generic_arg(ty, seg).ok()
}

/// The `FieldBuilder` call for an explicit `#[java(signature = "...")]`.
fn signature_method(sig: &str) -> TokenStream {
    match sig.strip_prefix('[') {
        // `FieldBuilder::array` prepends `[` itself, so strip the one the user wrote.
        Some(stripped) => quote!(.array(#stripped)),
        None => quote!(.object(#sig)),
    }
}

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let container = attr::parse_container(&input.attrs, input.ident.span())?;

    // Lifetimes are fine; type and const params are not, because the generated
    // `static CLASS: Lazy<Class>` cannot depend on them.
    for p in &input.generics.params {
        match p {
            GenericParam::Lifetime(_) => {}
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "`JavaSerialize` cannot be derived for a generic struct: the generated \
                     `static CLASS: Lazy<Class>` cannot depend on type parameters",
                ));
            }
        }
    }

    let named = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(n),
            ..
        }) => &n.named,
        Data::Struct(_) => {
            return Err(syn::Error::new(
                input.ident.span(),
                "`JavaSerialize` requires a struct with named fields",
            ));
        }
        Data::Enum(_) => {
            return Err(syn::Error::new(
                input.ident.span(),
                "`JavaSerialize` cannot be derived for an enum",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.ident.span(),
                "`JavaSerialize` cannot be derived for a union",
            ));
        }
    };

    let mut resolved: Vec<Resolved> = Vec::new();
    for f in named {
        let fa = attr::parse_field(f)?;
        if fa.skip {
            continue;
        }
        let ident = f.ident.as_ref().expect("named field");
        let java_name = fa
            .rename
            .clone()
            .unwrap_or_else(|| ident.to_string().trim_start_matches("r#").to_string());
        // `with` replaces the mapping table entirely: the field's Rust type is whatever the
        // `Layout` impl accepts as its `Input`, so it is not resolved here — which also means
        // the Java-side type has to be spelled out.
        let (ser, is_primitive, method) = match fa.with {
            Some(path) => {
                let Some(sig) = &fa.signature else {
                    return Err(syn::Error::new(
                        path.span(),
                        "`with` requires an explicit `#[java(signature = \"...\")]`: the Java \
                         field type cannot be derived from a `Layout` impl",
                    ));
                };
                let is_option = peel_option(&f.ty).is_some();
                (
                    FieldSer::With { path, is_option },
                    false,
                    signature_method(sig),
                )
            }
            None => {
                let jty = ty::resolve(&f.ty)?;
                if fa.signature.is_some() && jty.is_primitive() {
                    return Err(syn::Error::new(
                        f.ty.span(),
                        "`signature` cannot be applied to a primitive field",
                    ));
                }
                let method = match &fa.signature {
                    Some(sig) => signature_method(sig),
                    None => jty.field_method(),
                };
                let is_primitive = jty.is_primitive();
                (FieldSer::Mapped(jty), is_primitive, method)
            }
        };
        resolved.push(Resolved {
            java_name,
            access: quote!(self.#ident),
            is_primitive,
            ser,
            method,
            span: f.span(),
        });
    }

    // Duplicate check is independent of sort order (and thus of primitiveness),
    // so a `rename` collision that straddles the primitive/object boundary is
    // still caught. Attach the error to the offending (later) field's span.
    let mut seen: std::collections::HashMap<&str, ()> = std::collections::HashMap::new();
    for r in &resolved {
        if seen.contains_key(r.java_name.as_str()) {
            return Err(syn::Error::new(
                r.span,
                format!(
                    "duplicate Java field name `{}`; check your `rename` attributes",
                    r.java_name
                ),
            ));
        }
        seen.insert(r.java_name.as_str(), ());
    }

    // Mirror `Field::cmp`: primitives first, then alphabetically by Java name.
    // `sort_by` is stable, so equal keys keep declaration order.
    resolved.sort_by(|a, b| (!a.is_primitive, &a.java_name).cmp(&(!b.is_primitive, &b.java_name)));

    let mut field_calls: Vec<TokenStream> = Vec::new();
    let mut write_stmts: Vec<TokenStream> = Vec::new();
    let mut array_statics: Vec<TokenStream> = Vec::new();

    let mut uses_layout = false;

    for (idx, r) in resolved.iter().enumerate() {
        let name = &r.java_name;
        let method = &r.method;
        field_calls.push(quote!(
            .field(::serde_java::Field::builder(#name)#method)
        ));

        match &r.ser {
            FieldSer::Mapped(ty) => {
                let array_class = ty.obj_array_elem().map(|elem| {
                    let id = Ident::new(&format!("__ARRAY_CLASS_{idx}"), Span::call_site());
                    array_statics.push(quote!(
                        static #id: Lazy<::serde_java::Class> = Lazy::new(|| {
                            ::serde_java::Class::class_of_array(
                                &<#elem as ::serde_java::JavaObject>::class(),
                            )
                        });
                    ));
                    id
                });
                write_stmts.push(ty.write_stmts(&r.access, array_class.as_ref()));
            }
            FieldSer::With { path, is_option } => {
                uses_layout = true;
                let access = &r.access;
                // `#path` is generic in the general case (`ArrayList<'a, T>`), so its
                // parameters are left to inference rather than spelled out.
                write_stmts.push(if *is_option {
                    quote!(
                        match &#access {
                            ::std::option::Option::Some(__v) => {
                                ::serde_java::JavaWriteable::write_to(&#path::layout(__v), w)?;
                            }
                            ::std::option::Option::None => {
                                w.write_null()?;
                            }
                        }
                    )
                } else {
                    quote!(
                        ::serde_java::JavaWriteable::write_to(&#path::layout(&#access), w)?;
                    )
                });
            }
        }
    }

    // Brought into scope only when needed, so that `#path::layout(..)` resolves to the trait
    // method; an unconditional import would warn on every other derive.
    let layout_import = uses_layout.then(|| {
        quote!(
            use ::serde_java::Layout as _;
        )
    });

    // A struct whose fields are all `skip`ped writes nothing, which would leave `w` unused.
    let w = if write_stmts.is_empty() {
        Ident::new("_w", Span::call_site())
    } else {
        Ident::new("w", Span::call_site())
    };

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let class_name = &container.class;
    let suid = Literal::i64_suffixed(container.serial_version_uid);

    Ok(quote! {
        const _: () = {
            use ::serde_java::__private::Lazy;
            #layout_import

            static CLASS: Lazy<::serde_java::Class> = Lazy::new(|| {
                ::serde_java::Class::builder(#class_name, #suid)
                    #(#field_calls)*
                    .build()
            });

            #(#array_statics)*

            impl #impl_generics ::serde_java::JavaObject for #ident #ty_generics #where_clause {
                fn class() -> ::serde_java::Class {
                    ::core::clone::Clone::clone(&CLASS)
                }
            }

            impl #impl_generics ::serde_java::JavaSerializable for #ident #ty_generics #where_clause {
                fn write_fields(
                    &self,
                    #w: &mut ::serde_java::ObjectWriter<&mut dyn ::std::io::Write>,
                ) -> ::std::io::Result<()> {
                    #(#write_stmts)*
                    ::std::result::Result::Ok(())
                }
            }
        };
    })
}
