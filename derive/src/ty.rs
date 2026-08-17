use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{GenericArgument, PathArguments, Type, TypePath};

/// The Java type a Rust field maps to.
pub enum JavaTy {
    /// A JVM primitive: (`FieldBuilder` method, `FieldValue` variant, optional `as` cast).
    Prim(&'static str, &'static str, Option<&'static str>),
    /// `java.lang.String`.
    Str,
    /// Anything else — assumed to implement `JavaObject`.
    Object(Type),
    /// `Option<T>` where `T` is not a primitive.
    Nullable(Box<JavaTy>),
}

impl JavaTy {
    pub fn is_primitive(&self) -> bool {
        matches!(self, JavaTy::Prim(..))
    }

    /// Element type when this is an object array, peeling `Option`. Always `None`
    /// until Task 5 adds the array variants.
    pub fn obj_array_elem(&self) -> Option<&Type> {
        match self {
            JavaTy::Nullable(inner) => inner.obj_array_elem(),
            _ => None,
        }
    }

    /// The `FieldBuilder` call chained onto `Field::builder(name)`.
    pub fn field_method(&self) -> TokenStream {
        match self {
            JavaTy::Prim(method, _, _) => {
                let m = Ident::new(method, Span::call_site());
                quote!(.#m())
            }
            JavaTy::Str => quote!(.string()),
            JavaTy::Object(t) => quote!(
                .object(<#t as ::serde_java::JavaObject>::class().signature())
            ),
            JavaTy::Nullable(inner) => inner.field_method(),
        }
    }

    /// Builds the `FieldValue` for this field. `access` is a place expression such
    /// as `self.x`. `array_class` names the cached array-class static, used only by
    /// the object-array variant added in Task 5.
    pub fn value_expr(&self, access: &TokenStream, array_class: Option<&Ident>) -> TokenStream {
        match self {
            JavaTy::Prim(_, variant, cast) => {
                let v = Ident::new(variant, Span::call_site());
                match cast {
                    Some(c) => {
                        let c = Ident::new(c, Span::call_site());
                        quote!(::serde_java::FieldValue::#v(#access as #c))
                    }
                    None => quote!(::serde_java::FieldValue::#v(#access)),
                }
            }
            JavaTy::Str => quote!(::serde_java::FieldValue::String(&#access)),
            JavaTy::Object(t) => quote!(::serde_java::FieldValue::Object(
                <#t as ::serde_java::JavaObject>::class(),
                &#access
            )),
            JavaTy::Nullable(inner) => {
                let inner_value = inner.value_expr(&quote!((*__v)), array_class);
                quote!(match &#access {
                    ::std::option::Option::Some(__v) => #inner_value,
                    ::std::option::Option::None => ::serde_java::FieldValue::Null,
                })
            }
        }
    }
}

pub fn resolve(ty: &Type) -> syn::Result<JavaTy> {
    match ty {
        // `&str` is the only reference form supported so far; Task 5 adds `&[u8]`.
        Type::Reference(r) => {
            let inner = resolve(&r.elem)?;
            match inner {
                JavaTy::Str => Ok(inner),
                _ => Err(syn::Error::new(
                    ty.span(),
                    "reference fields are only supported for `&str` and primitive slices",
                )),
            }
        }
        Type::Path(p) => resolve_path(ty, p),
        other => Err(syn::Error::new(
            other.span(),
            "unsupported field type for `JavaSerialize`",
        )),
    }
}

fn resolve_path(orig: &Type, p: &TypePath) -> syn::Result<JavaTy> {
    let seg = p.path.segments.last().ok_or_else(|| {
        syn::Error::new(orig.span(), "unsupported field type for `JavaSerialize`")
    })?;
    let name = seg.ident.to_string();

    if matches!(seg.arguments, PathArguments::None) {
        return scalar(orig, &name);
    }

    match name.as_str() {
        "Vec" => {
            let _elem = single_generic_arg(orig, seg)?;
            Err(syn::Error::new(
                orig.span(),
                "array fields are not supported yet",
            ))
        }
        "Option" => {
            let inner_ty = single_generic_arg(orig, seg)?;
            let inner = resolve(inner_ty)?;
            if inner.is_primitive() {
                return Err(syn::Error::new(
                    orig.span(),
                    "Java primitives cannot be null; drop the `Option`, or use an object type",
                ));
            }
            if matches!(inner, JavaTy::Nullable(_)) {
                return Err(syn::Error::new(
                    orig.span(),
                    "nested `Option` has no Java equivalent",
                ));
            }
            Ok(JavaTy::Nullable(Box::new(inner)))
        }
        // A generic path such as `HashMap<K, V>` is treated as an opaque JavaObject.
        _ => Ok(JavaTy::Object(orig.clone())),
    }
}

fn scalar(orig: &Type, name: &str) -> syn::Result<JavaTy> {
    Ok(match name {
        "bool" => JavaTy::Prim("boolean", "Bool", None),
        "i8" => JavaTy::Prim("byte", "Byte", Some("u8")),
        "u8" => JavaTy::Prim("byte", "Byte", None),
        "u16" => JavaTy::Prim("char", "Char", None),
        "i16" => JavaTy::Prim("short", "Short", None),
        "i32" => JavaTy::Prim("int", "Int", None),
        "i64" => JavaTy::Prim("long", "Long", None),
        "f32" => JavaTy::Prim("float", "Float", None),
        "f64" => JavaTy::Prim("double", "Double", None),
        "String" | "str" => JavaTy::Str,
        "char" => {
            return Err(syn::Error::new(
                orig.span(),
                "Rust `char` is 4 bytes and does not match Java's 2-byte `char`; use `u16`",
            ));
        }
        "u32" | "u64" | "usize" | "isize" | "i128" | "u128" => {
            return Err(syn::Error::new(
                orig.span(),
                format!(
                    "`{name}` has no Java equivalent; use one of \
                     i8/u8/u16/i16/i32/i64/f32/f64"
                ),
            ));
        }
        _ => JavaTy::Object(orig.clone()),
    })
}

pub(crate) fn single_generic_arg<'a>(
    orig: &Type,
    seg: &'a syn::PathSegment,
) -> syn::Result<&'a Type> {
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return Err(syn::Error::new(
            orig.span(),
            "expected exactly one generic type argument",
        ));
    };
    let mut types = args.args.iter().filter_map(|a| match a {
        GenericArgument::Type(t) => Some(t),
        _ => None,
    });
    let first = types.next().ok_or_else(|| {
        syn::Error::new(orig.span(), "expected exactly one generic type argument")
    })?;
    if types.next().is_some() {
        return Err(syn::Error::new(
            orig.span(),
            "expected exactly one generic type argument",
        ));
    }
    Ok(first)
}
