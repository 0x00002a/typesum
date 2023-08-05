# TypeSum

Utilities for working with enums, primarily aimed at sum types.

## What's in it

### `#[sumtype]`: Generate all the functions and impls you could ever need for a sum type

If you want the full list it's in the [docs for the attribute](typesum::sumtype)

```rust
use typesum::{sumtype, TryIntoError};
#[sumtype]
#[derive(Debug, PartialEq, Eq)]
enum MyT {
    Int(i64),
    Bool(bool),
}
let mut v = MyT::Int(6);
assert!(v.is_int());
assert_eq!(v.as_int(), Some(&6));
assert_eq!(v.as_int_mut(), Some(&mut 6));
assert_eq!(v.try_as_int(), Ok(&6));
assert_eq!(v.try_as_int_mut(), Ok(&mut 6));
assert_eq!(v.try_as_bool(), Err(TryIntoError::new("MyT", "Int", "Bool")));
assert_eq!(MyT::from(false), MyT::Bool(false));
assert_eq!(MyT::from(false).try_into_int(), Err(TryIntoError::new("MyT", "Bool", "Int")));
```

Individual variants can be ignored with `#[sumtype(ignore)]`

```rust,compile_fail
use typesum::sumtype;
#[sumtype]
enum MyT {
    Int(i64),
    #[sumtype(ignore)]
    Empty,
}
let v = MyT::Empty;
v.as_empty(); // doesn't exist
```

It can even work with overlapping types

```rust
use typesum::sumtype;
#[sumtype(from = false, impl_try_into)]
#[derive(Debug, PartialEq, Eq)]
enum Overlap {
    #[sumtype(from)]
    Int1(i64),
    Int2(i64),
    Int3(i64),
    #[sumtype(from)]
    Bool1(bool),
    Bool2(bool),
}
assert_eq!(Overlap::from(0), Overlap::Int1(0));
assert_eq!(Overlap::Int3(5).try_into(), Ok(5));
assert_eq!(Overlap::from(false), Overlap::Bool1(false));
assert_eq!(Overlap::Bool2(false).try_into(), Ok(false));
```

### `#[kinded]`: Generate kind aliases for an enum

```rust
use typesum::kinded;

#[kinded]
enum LookAtMe {
    Hello { world: String },
    ImAUnit,
    OrPerhapsATuple(i64, u32),
}
let look = LookAtMe::ImAUnit;
assert_eq!(look.kind(), LookAtMeKind::ImAUnit);
```
