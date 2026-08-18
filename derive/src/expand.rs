use crate::attr;
use crate::ty::{self, JavaTy};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataStruct, DeriveInput, Fields, GenericParam};

struct Resolved {
    java_name: String,
    access: TokenStream,
    ty: JavaTy,
    /// Set by `#[java(signature = "...")]`; overrides the schema side only.
    signature: Option<String>,
    is_primitive: bool,
    span: Span,
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
        let jty = ty::resolve(&f.ty)?;
        if fa.signature.is_some() && jty.is_primitive() {
            return Err(syn::Error::new(
                f.ty.span(),
                "`signature` cannot be applied to a primitive field",
            ));
        }
        resolved.push(Resolved {
            java_name,
            access: quote!(self.#ident),
            is_primitive: jty.is_primitive(),
            ty: jty,
            signature: fa.signature,
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
    resolved.sort_by(|a, b| {
        (!a.is_primitive, &a.java_name).cmp(&(!b.is_primitive, &b.java_name))
    });

    let mut field_calls: Vec<TokenStream> = Vec::new();
    let mut value_exprs: Vec<TokenStream> = Vec::new();
    let mut array_statics: Vec<TokenStream> = Vec::new();

    for (idx, r) in resolved.iter().enumerate() {
        let name = &r.java_name;
        let method = match &r.signature {
            // `FieldBuilder::array` prepends `[` itself, so strip the one the user wrote.
            Some(sig) if sig.starts_with('[') => {
                let stripped = &sig[1..];
                quote!(.array(#stripped))
            }
            Some(sig) => quote!(.object(#sig)),
            None => r.ty.field_method(),
        };
        field_calls.push(quote!(
            .field(::serde_java::Field::builder(#name)#method)
        ));

        let array_class = r.ty.obj_array_elem().map(|elem| {
            let id = Ident::new(&format!("__ARRAY_CLASS_{idx}"), Span::call_site());
            array_statics.push(quote!(
                static #id: Lazy<::serde_java::Class> = Lazy::new(|| {
                    ::serde_java::Class::class_of_object_array(
                        &<#elem as ::serde_java::JavaObject>::class(),
                    )
                });
            ));
            id
        });

        value_exprs.push(r.ty.value_expr(&r.access, array_class.as_ref()));
    }

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let class_name = &container.class;
    let suid = Literal::i64_suffixed(container.serial_version_uid);

    Ok(quote! {
        const _: () = {
            use ::serde_java::__private::Lazy;

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
                fn fields(&self) -> ::std::vec::Vec<::serde_java::FieldValue<'_>> {
                    ::std::vec![ #(#value_exprs),* ]
                }
            }
        };
    })
}
