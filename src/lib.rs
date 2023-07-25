#![doc = include_str!("../docs/README.md")]
pub use macros::SumType;

extern crate self as typesum;
///
pub struct TryIntoError {
    from: &'static str,
    to: &'static str,
}
impl TryIntoError {
    pub fn new(from: &'static str, to: &'static str) -> Self {
        Self { from, to }
    }

    pub fn from(&self) -> &'static str {
        self.from
    }
    pub fn to(&self) -> &'static str {
        self.to
    }
}

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

#[cfg(test)]
mod tests {
    use crate::{MySum, SumType, TryIntoError};

    #[derive(SumType)]
    enum MySumDerive {
        #[sumtype(is = false)]
        Int(i64),
        #[sumtype(mut_as = false)]
        Float(f64),
        String(String),
        Bool(bool),
        #[sumtype(ignore)]
        Not,
    }

    #[derive(SumType)]
    enum MySumDeriveTyped<T> {
        A(T),
    }

    #[derive(SumType)]
    enum MySumDeriveLifetimed<'a> {
        A(&'a i32),
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
