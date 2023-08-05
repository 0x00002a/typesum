Generate a kinds enum for another enum

A kinds enum is an enum where each variant is the unit variant version of
the original, e.g.

```rust
use typesum::kinded;
#[kinded]
#[derive(Debug)]
enum MyKinded {
    I(i64),
    A { expensive_thing: String, also_expensive: Vec<Vec<Vec<Vec<Vec<Vec<Vec<String>>>>>>>, },
}
let my_thing = MyKinded::I(10);
assert_eq!(my_thing.kind(), MyKindedKind::I);
```

You can also customise the name of the generated enum

```rust
use typesum::kinded;
#[kinded(name = "SomeKind")]
#[derive(Debug)]
enum MyKinded {
    I(i64),
}
let my_thing = MyKinded::I(10);
assert_eq!(my_thing.kind(), SomeKind::I);
```

Or control whether `.kind()` is generated at all

```rust,compile_fail
use typesum::kinded;
#[kinded(no_kind_fn)]
#[derive(Debug)]
enum MyKinded {
    I(i64),
}
let my_thing = MyKinded::I(10);
assert_eq!(my_thing.kind(), SomeKind::I);
```

Or change the name for the kind function

```rust
use typesum::kinded;
#[kinded(kind_fn = "mykind")]
#[derive(Debug)]
enum MyKinded {
    I(i64),
}
let my_thing = MyKinded::I(10);
assert_eq!(my_thing.mykind(), MyKindedKind::I);
```
