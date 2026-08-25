use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{GenericArgument, PathArguments, Type, TypePath};

/// The Java type a Rust field maps to.
pub enum JavaTy {
    /// A JVM primitive: (`FieldBuilder` method, `ObjectWriter` method).
    Prim(&'static str, &'static str),
    /// `java.lang.String`. `true` when the Rust field is `&str` (already a reference), so
    /// `write_stmts` must not take another `&` of it.
    Str(bool),
    /// Anything else — assumed to implement `JavaObject`.
    Object(Type),
    /// `Option<T>` where `T` is not a primitive.
    Nullable(Box<JavaTy>),
    /// A JVM primitive array: (`FieldBuilder` method, `ObjectWriter` method).
    PrimArray(&'static str, &'static str),
    /// `Vec<T>` / `&[T]` where `T` maps to `Object`; carries the element type.
    ObjArray(Type),
}

impl JavaTy {
    pub fn is_primitive(&self) -> bool {
        matches!(self, JavaTy::Prim(..))
    }

    /// Element type when this is an object array, peeling `Option`.
    pub fn obj_array_elem(&self) -> Option<&Type> {
        match self {
            JavaTy::ObjArray(elem) => Some(elem),
            JavaTy::Nullable(inner) => inner.obj_array_elem(),
            _ => None,
        }
    }

    /// The `FieldBuilder` call chained onto `Field::builder(name)`.
    pub fn field_method(&self) -> TokenStream {
        match self {
            JavaTy::Prim(method, _) => {
                let m = Ident::new(method, Span::call_site());
                quote!(.#m())
            }
            JavaTy::Str(_) => quote!(.string()),
            JavaTy::Object(t) => quote!(
                .object(<#t as ::serde_java::JavaObject>::class().signature())
            ),
            JavaTy::Nullable(inner) => inner.field_method(),
            JavaTy::PrimArray(method, _) => {
                let m = Ident::new(method, Span::call_site());
                quote!(.#m())
            }
            JavaTy::ObjArray(elem) => quote!(
                .array(<#elem as ::serde_java::JavaObject>::class().signature())
            ),
        }
    }

    /// The statements that write this field's value through the writer bound to `w`.
    /// `access` is a place expression such as `self.x`. `array_class` names the cached
    /// array-class static, used only by the object-array variant.
    pub fn write_stmts(&self, access: &TokenStream, array_class: Option<&Ident>) -> TokenStream {
        match self {
            JavaTy::Prim(_, method) => {
                let m = Ident::new(method, Span::call_site());
                quote!(w.#m(#access)?;)
            }
            JavaTy::Str(is_ref) => {
                if *is_ref {
                    quote!(w.write(#access)?;)
                } else {
                    quote!(w.write(&#access)?;)
                }
            }
            JavaTy::Object(t) => quote!(
                <#t as ::serde_java::JavaWriteable>::write_to(&#access, w)?;
            ),
            JavaTy::Nullable(inner) => {
                let inner_stmts = inner.write_stmts(&quote!((*__v)), array_class);
                quote!(match &#access {
                    ::std::option::Option::Some(__v) => { #inner_stmts }
                    ::std::option::Option::None => { w.write(())?; }
                })
            }
            JavaTy::PrimArray(_, method) => {
                let m = Ident::new(method, Span::call_site());
                quote!(w.#m(&#access)?;)
            }
            JavaTy::ObjArray(elem) => {
                let cached = array_class
                    .expect("expand.rs must supply a cached array class for every object array");
                quote!(
                    w.begin_array(&#cached, #access.len())?;
                    for __it in #access.iter() {
                        <#elem as ::serde_java::JavaWriteable>::write_to(__it, w)?;
                    }
                )
            }
        }
    }
}

pub fn resolve(ty: &Type) -> syn::Result<JavaTy> {
    match ty {
        Type::Reference(r) => {
            let inner = resolve(&r.elem)?;
            match inner {
                JavaTy::Str(_) => Ok(JavaTy::Str(true)),
                JavaTy::PrimArray(..) => Ok(inner),
                _ => Err(syn::Error::new(
                    ty.span(),
                    "reference fields are only supported for `&str` and primitive slices \
                     such as `&[u8]`",
                )),
            }
        }
        Type::Path(p) => resolve_path(ty, p),
        Type::Slice(s) => resolve_array(ty, &s.elem),
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
        "Vec" => resolve_array(orig, single_generic_arg(orig, seg)?),
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
        "bool" => JavaTy::Prim("boolean", "write"),
        "i8" | "u8" => JavaTy::Prim("byte", "write"),
        "i16" | "u16" => JavaTy::Prim("short", "write"),
        "i32" | "u32" => JavaTy::Prim("int", "write"),
        "i64" | "u64" => JavaTy::Prim("long", "write"),
        "f32" => JavaTy::Prim("float", "write"),
        "f64" => JavaTy::Prim("double", "write"),
        "String" | "str" => JavaTy::Str(false),
        "char" => {
            return Err(syn::Error::new(
                orig.span(),
                "Rust `char` is 4 bytes and does not match Java's 2-byte `char`; use `u16`",
            ));
        }
        "usize" | "isize" | "i128" | "u128" => {
            return Err(syn::Error::new(
                orig.span(),
                format!(
                    "`{name}` has no Java equivalent; use one of \
                     i8/u8/i16/u16/i32/u32/i64/u64/f32/f64"
                ),
            ));
        }
        _ => JavaTy::Object(orig.clone()),
    })
}

/// Resolves an array field from its element type. Shared by `Vec<T>` and `&[T]`.
fn resolve_array(orig: &Type, elem: &Type) -> syn::Result<JavaTy> {
    let Type::Path(p) = elem else {
        return Err(syn::Error::new(
            orig.span(),
            "unsupported array element type",
        ));
    };
    let seg = p
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new(orig.span(), "unsupported array element type"))?;
    let name = seg.ident.to_string();

    if !matches!(seg.arguments, PathArguments::None) {
        if name == "Vec" || name == "Option" {
            return Err(syn::Error::new(
                orig.span(),
                format!("arrays of `{name}<..>` are not supported"),
            ));
        }
        return Ok(JavaTy::ObjArray(elem.clone()));
    }

    Ok(match name.as_str() {
        "bool" => JavaTy::PrimArray("boolean_array", "write_all"),
        "i8" => JavaTy::PrimArray("byte_array", "write_all"),
        "i16" => JavaTy::PrimArray("short_array", "write_all"),
        "i32" => JavaTy::PrimArray("int_array", "write_all"),
        "i64" | "isize" => JavaTy::PrimArray("long_array", "write_all"),
        "u8" => JavaTy::PrimArray("byte_array", "write_all"),
        "u16" => JavaTy::PrimArray("short_array", "write_all"),
        "u32" => JavaTy::PrimArray("int_array", "write_all"),
        "u64" | "usize" => JavaTy::PrimArray("long_array", "write_all"),
        "f32" => JavaTy::PrimArray("float_array", "write_all"),
        "f64" => JavaTy::PrimArray("double_array", "write_all"),
        "String" | "str" => JavaTy::PrimArray("string_array", "write_all"),
        "char" => {
            return Err(syn::Error::new(
                orig.span(),
                format!(
                    "arrays of `{name}` are not supported yet: `#[derive(JavaSerialize)]` \
                     has no value-side writer wired up for them"
                ),
            ));
        }
        "i128" | "u128" => {
            return Err(syn::Error::new(
                orig.span(),
                format!("`{name}` has no Java equivalent"),
            ));
        }
        _ => JavaTy::ObjArray(elem.clone()),
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
