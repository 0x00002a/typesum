#![doc = include_str!("../docs/README.md")]

use std::marker::PhantomData;

#[doc = include_str!("../docs/sumtype_attr.md")]
#[cfg(feature = "sumtype")]
pub use typesum_macros::sumtype;

#[doc = include_str!("../docs/kinded.md")]
#[cfg(feature = "kinded")]
pub use typesum_macros::kinded;

extern crate self as typesum;
/// Error type for TryInto impl's on derived sumtypes
///
///
/// ```
/// use typesum::{sumtype, TryIntoError};
/// #[sumtype(impl_try_into)]
/// enum MySum {
///     I(i64),
///     B(bool),
/// }
/// let v = MySum::B(true);
/// let r: Result<i64, _> = v.try_into();
/// assert_eq!(r, Err(TryIntoError::new("MySum", "B", "I")));
/// let e = r.unwrap_err();
/// assert_eq!(e.source(), "MySum");
/// assert_eq!(e.actual(), "B");
/// assert_eq!(e.expected(), "I");
/// ```
#[impl_tools::autoimpl(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TryIntoError<Source> {
    discrim: PhantomData<Source>,
    source: &'static str,
    actual: &'static str,
    expected: &'static str,
}
impl<Source> TryIntoError<Source> {
    /// Create a new `TryIntoError`
    pub fn new(source: &'static str, actual: &'static str, expected: &'static str) -> Self {
        Self {
            discrim: PhantomData::default(),
            source,
            actual,
            expected,
        }
    }
    /// Source sumtype
    ///
    /// The source is the enum that the operation failed for
    pub fn source(&self) -> &'static str {
        self.source
    }
    /// Actual variant
    ///
    /// The actual variant is the one that the source was actually
    /// an instance of
    pub fn actual(&self) -> &'static str {
        self.actual
    }
    /// Expected variant
    ///
    /// The expected variant is the one we are expecting the source to
    /// be
    pub fn expected(&self) -> &'static str {
        self.expected
    }
}
impl<S> std::fmt::Display for TryIntoError<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "sumtype '{}': variant '{}' expected but was '{}'",
            self.source, self.expected, self.actual
        ))
    }
}
impl<S> std::error::Error for TryIntoError<S> {}

#[cfg(test)]
mod tests {
    use crate::sumtype;
    use crate::TryIntoError;

    #[sumtype]
    #[derive(Clone)]
    #[allow(unused)]
    enum MySumDerive {
        #[sumtype(is = false)]
        Int(i64),
        #[sumtype(as_mut = false, try_into = false)]
        Float(f64),
        String(String),
        #[sumtype(impl_try_into)]
        Bool(bool),
        #[sumtype(ignore)]
        Not,
    }

    #[sumtype]
    enum MySumDeriveTyped<T> {
        A(T),
    }

    #[sumtype]
    enum MySumDeriveLifetimed<'a> {
        A(&'a i32),
    }
    #[test]
    fn my_sum_derive_try_into() {
        let v = MySumDerive::Int(64);
        assert_eq!(
            v.clone().try_into_string(),
            Err(TryIntoError::new("MySumDerive", "Int", "String"))
        );
        let k: Result<bool, _> = v.try_into();
        assert_eq!(k, Err(TryIntoError::new("MySumDerive", "Int", "Bool")));
    }

    #[test]
    fn test_derive_typed() {
        fn assert_typed<T>(val: &MySumDeriveTyped<T>) -> &T {
            val.as_a().unwrap()
        }
        assert_typed(&MySumDeriveTyped::A(6));
    }
    #[test]
    fn test_lifetime_deser() {
        fn assert_lifetime(val: MySumDeriveLifetimed<'_>) -> &i32 {
            val.as_a().unwrap()
        }
        assert_lifetime(MySumDeriveLifetimed::A(&5));
    }
}
