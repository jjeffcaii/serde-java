use syn::spanned::Spanned;
use syn::{Attribute, Expr, ExprLit, ExprUnary, Field, Lit, LitStr, Path, UnOp};

pub struct ContainerAttr {
    pub class: String,
    pub serial_version_uid: i64,
}

pub struct FieldAttr {
    pub rename: Option<String>,
    pub skip: bool,
    pub signature: Option<String>,
    /// Set by `#[java(with = "...")]`: a type implementing `serde_java::Layout`
    /// that takes over the value side of this field.
    pub with: Option<Path>,
}

pub fn parse_container(attrs: &[Attribute], span: proc_macro2::Span) -> syn::Result<ContainerAttr> {
    let mut class: Option<String> = None;
    let mut suid: Option<i64> = None;

    for attr in attrs.iter().filter(|a| a.path().is_ident("java")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("class") {
                let s: LitStr = meta.value()?.parse()?;
                class = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("serial_version_uid") {
                let expr: Expr = meta.value()?.parse()?;
                suid = Some(parse_i64(&expr)?);
                Ok(())
            } else {
                Err(meta.error(
                    "unknown `java` container attribute; expected `class` or `serial_version_uid`",
                ))
            }
        })?;
    }

    let class = class.ok_or_else(|| {
        syn::Error::new(
            span,
            "missing `#[java(class = \"...\")]`: the fully-qualified Java class name is required",
        )
    })?;
    let serial_version_uid = suid.ok_or_else(|| {
        syn::Error::new(
            span,
            "missing `#[java(serial_version_uid = ...)]`: it cannot be derived from a Rust \
             struct because Java's default algorithm hashes method and constructor \
             signatures; copy the value from the Java class",
        )
    })?;

    Ok(ContainerAttr {
        class,
        serial_version_uid,
    })
}

pub fn parse_field(field: &Field) -> syn::Result<FieldAttr> {
    let mut out = FieldAttr {
        rename: None,
        skip: false,
        signature: None,
        with: None,
    };

    for attr in field.attrs.iter().filter(|a| a.path().is_ident("java")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let s: LitStr = meta.value()?.parse()?;
                out.rename = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("skip") {
                out.skip = true;
                Ok(())
            } else if meta.path.is_ident("signature") {
                let s: LitStr = meta.value()?.parse()?;
                let v = s.value();
                if !(v.starts_with('L') || v.starts_with('[')) {
                    return Err(syn::Error::new(
                        s.span(),
                        "`signature` must start with `L` or `[`; a primitive signature would \
                         break field ordering",
                    ));
                }
                out.signature = Some(v);
                Ok(())
            } else if meta.path.is_ident("with") {
                let s: LitStr = meta.value()?.parse()?;
                // Spans point inside the literal, so a malformed path is reported there.
                out.with = Some(s.parse()?);
                Ok(())
            } else {
                Err(meta.error(
                    "unknown `java` field attribute; expected `rename`, `skip`, `signature` or \
                     `with`",
                ))
            }
        })?;
    }

    Ok(out)
}

/// Parses an integer literal, optionally negated. Goes through `i128` so that
/// `i64::MIN` (whose magnitude does not fit in `i64`) still parses.
fn parse_i64(expr: &Expr) -> syn::Result<i64> {
    let (neg, lit) = match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => (false, i),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => match &**expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(i), ..
            }) => (true, i),
            other => {
                return Err(syn::Error::new(other.span(), "expected an integer literal"));
            }
        },
        other => {
            return Err(syn::Error::new(other.span(), "expected an integer literal"));
        }
    };

    let magnitude = lit.base10_parse::<i128>()?;
    let value = if neg { -magnitude } else { magnitude };
    i64::try_from(value)
        .map_err(|_| syn::Error::new(lit.span(), "value does not fit in a Java `long` (i64)"))
}
