/// Converts a borrowed Rust value into a `JavaWriteable` representation for
/// `#[java(with = "...")]` fields, bypassing the derive macro's built-in Rust→Java type
/// mapping table.
///
/// `Input` is the field's Rust type (or, for an `Option<T>` field, `T` itself — `None` writes
/// `null` directly without going through `layout`). `Output` is typically a wrapper type that
/// implements `JavaWriteable`/`JavaObject` for the target Java type; `layout` performs the
/// conversion from the borrowed `Input` to that `Output`.
///
/// See `ext::Boolean`, `ext::number`, and `ext::list::ArrayList`/`LinkedList` for
/// implementations.
pub trait Layout<'a> {
    /// The field's Rust type, as declared on the struct (or its `Option<T>` inner type `T`).
    type Input;
    /// The `JavaWriteable` value produced from `Input`, written in place of the field.
    type Output;

    /// Converts a borrowed `Input` into `Output`.
    fn layout(input: &'a Self::Input) -> Self::Output;
}
