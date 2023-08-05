#![doc = include_str!("../docs/README.md")]

#[doc = include_str!("../docs/sumtype_attr.md")]
pub use macros::sumtype;

#[doc = include_str!("../docs/kinded.md")]
pub use macros::kinded;

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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TryIntoError {
    source: &'static str,
    actual: &'static str,
    expected: &'static str,
}
impl TryIntoError {
    /// Create a new `TryIntoError`
    pub fn new(source: &'static str, actual: &'static str, expected: &'static str) -> Self {
        Self {
            source,
            actual,
            expected,
        }
    }
    /// Source sumtype
    pub fn source(&self) -> &'static str {
        self.source
    }
    /// Actual variant
    pub fn actual(&self) -> &'static str {
        self.actual
    }
    /// Expected variant
    pub fn expected(&self) -> &'static str {
        self.expected
    }
}
impl std::fmt::Display for TryIntoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "sumtype '{}': variant '{}' expected but was '{}'",
            self.source, self.expected, self.actual
        ))
    }
}
impl std::error::Error for TryIntoError {}

pub trait SumType {
    type Kind;
    fn is_kind<K: KindFor<Self, Parent = Self::Kind>>(&self, kind: Self::Kind) -> bool {
        K::get_is(self)
    }
}

pub trait KindFor<S: ?Sized> {
    type Inner;
    type Parent;
    fn get_as(target: &S) -> Option<&Self::Inner>;
    fn get_into(target: S) -> Option<Self::Inner>
    where
        S: Sized;
    fn get_is(target: &S) -> bool;
}

enum MySum {
    Int(i64),
    Float(f64),
}

enum MySumKind {
    Int(MySumKindInt),
    Float,
}

impl SumType for MySum {
    type Kind = MySumKind;
}

struct MySumKindInt {}

impl KindFor<MySum> for MySumKindInt {
    type Parent = MySumKind;
    type Inner = i64;

    fn get_as(target: &MySum) -> Option<&Self::Inner> {
        match target {
            MySum::Int(i) => Some(i),
            _ => None,
        }
    }

    fn get_into(target: MySum) -> Option<Self::Inner>
    where
        MySum: Sized,
    {
        match target {
            MySum::Int(i) => Some(i),
            _ => None,
        }
    }

    fn get_is(target: &MySum) -> bool {
        matches!(target, MySum::Int(_))
    }
}
#[sumtype]
enum MySumDerive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    #[sumtype(ignore)]
    Not,
}

#[cfg(test)]
mod tests {
    use crate::sumtype;
    use crate::{SumType, TryIntoError};

    #[sumtype]
    #[derive(Clone)]
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
