```rust
use typesum::SumType;
#[derive(SumType)]
enum MyT {
    Int(i64),
    #[sumtype(ignore)]
    Empty,
}
let v = MyT::Int(6);
assert_eq!(v.into_int(), Some(6));
```

Individual variants can be ignored with #[sumtype(ignore)]

```rust
use typesum::SumType;
#[derive(SumType)]
enum MyT {
    Int(i64),
    #[sumtype(ignore)]
    Empty,
}
```

```rust,compile_fail
use typesum::SumType;
#[derive(SumType)]
enum MyT {
    Int(i64),
    #[sumtype(ignore)]
    Empty,
}
let v = MyT::Empty;
v.as_empty(); // doesn't exist
```
