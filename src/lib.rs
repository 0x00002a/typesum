use macros::SumType;

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

#[derive(SumType)]
enum MySumDerive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}
