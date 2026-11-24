use crate::{
    attr,
    ty::{self, JavaTy},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, spanned::Spanned};

fn read_expr(ty: &JavaTy) -> TokenStream {
    match ty {
        JavaTy::Nullable(inner) => {
            let read = read_expr(inner);
            quote! { if r.read_null()? { None } else { Some(#read) } }
        }
        JavaTy::PrimArray(..) | JavaTy::ObjArray(_) => {
            quote!(::serde_java::ArrayReader::read_all(r)?)
        }
        _ => quote!(::serde_java::Reader::read(r)?),
    }
}

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    attr::parse_container(&input.attrs, input.ident.span())?;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "JavaDeserialize does not support generic structs",
        ));
    }
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => &n.named,
            _ => {
                return Err(syn::Error::new(
                    s.fields.span(),
                    "JavaDeserialize requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "JavaDeserialize requires a struct with named fields",
            ));
        }
    };
    let mut resolved = Vec::new();
    let mut init = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for f in fields {
        let a = attr::parse_field(f)?;
        let ident = f.ident.as_ref().unwrap();
        if a.skip {
            init.push(quote!(#ident: ::core::default::Default::default()));
            continue;
        }
        if a.with.is_some() {
            return Err(syn::Error::new(
                f.span(),
                "JavaDeserialize does not support `with` / Layout adapters",
            ));
        }
        fn owned(t: &Type) -> bool {
            match t {
                Type::Reference(_) | Type::Slice(_) => false,
                Type::Path(p) => p.path.segments.iter().all(|s| match &s.arguments {
                    syn::PathArguments::AngleBracketed(a) => a.args.iter().all(|a| match a {
                        syn::GenericArgument::Type(t) => owned(t),
                        syn::GenericArgument::Lifetime(_) => false,
                        _ => true,
                    }),
                    _ => true,
                }),
                _ => true,
            }
        }
        if !owned(&f.ty) {
            return Err(syn::Error::new(
                f.ty.span(),
                "JavaDeserialize requires owned field types",
            ));
        }
        let jt = ty::resolve(&f.ty)?;
        if a.signature.is_some() && jt.is_primitive() {
            return Err(syn::Error::new(
                f.span(),
                "`signature` cannot be applied to a primitive field",
            ));
        }
        let name = a
            .rename
            .unwrap_or_else(|| ident.to_string().trim_start_matches("r#").to_owned());
        if !seen.insert(name.clone()) {
            return Err(syn::Error::new(f.span(), "duplicate Java field name"));
        }
        resolved.push((!jt.is_primitive(), name, ident, &f.ty, read_expr(&jt)));
        init.push(quote!(#ident));
    }
    resolved.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let types: Vec<_> = resolved.iter().map(|f| f.3).collect();
    let names: Vec<_> = resolved.iter().map(|f| f.2).collect();
    let reads: Vec<_> = resolved.iter().map(|f| &f.4).collect();
    let ident = &input.ident;
    Ok(quote! {
        impl ::serde_java::JavaFieldsDeserializable for #ident {
            type Fields = (#(#types,)*);
            fn read_fields<R: ::std::io::Read>(r: &mut ::serde_java::ObjectReader<R>) -> ::std::io::Result<Self::Fields> {
                Ok((#(#reads,)*))
            }
        }
        impl ::serde_java::JavaDeserializable for #ident {
            fn read_object<R: ::std::io::Read>(r: &mut ::serde_java::ObjectReader<R>) -> ::std::io::Result<Self> {
                let (#(#names,)*) = <Self as ::serde_java::JavaFieldsDeserializable>::default_read_object(r)?;
                Ok(Self { #(#init,)* })
            }
        }
    })
}
