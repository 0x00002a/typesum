Generate useful functions for a sumtype

This can generate both the usual `as, as_mut, into` as well as `try` variants
(which return a `Result` rather than `Option`). The methods that get generated
can be configured by passing different arguments to the `sumtype` invocation,
either at the top level (which sets the default) or on a per variant basis

```rust,compile_fail
use typesum::{sumtype, TryIntoError};
#[sumtype(is = false)]
enum MySum {
    I(i64),
    B(bool),
}
MySum::I(50).is_i() // uh oh, we disabled this one
```

## Options

| Function prefix | Argument name | Return           | Default |
| --------------- | ------------- | ---------------- | ------- |
| `try_as_`       | `try_as`      | `Result<&T>`     | `true`  |
| `try_as_mut_`   | `try_as_mut`  | `Result<&mut T>` | `true`  |
| `try_into_`     | `try_into`    | `Result<T>`      | `true`  |
| `as_`           | `as`          | `Option<&T>`     | `true`  |
| `as_mut`        | `as_mut`      | `Option<&mut T>` | `true`  |
| `into_`         | `into`        | `Option<T>`      | `true`  |
| `is_`           | `is`          | `bool`           | `true`  |

Also the impls

| Trait        | Argument name   | Default |
| ------------ | --------------- | ------- |
| `From<T>`    | `from`          | `true`  |
| `TryInto<T>` | `impl_try_into` | `false` |

This is a total of 7 functions and 2 impls per enum variant, which
can explode pretty quick

```rust
use typesum::{sumtype, TryIntoError};
#[sumtype(impl_try_into)]
enum MySum {
    I(i64),
    B(bool),
}
let v = MySum::B(true);
let r: Result<i64, _> = v.try_into();
assert_eq!(r, Err(TryIntoError::new("MySum", "B", "I")));

```

### `all` and `ignore`

You can turn on and off everything with the `all` option (`ignore` is an alias
for `all = false`):

```rust
use typesum::{sumtype, TryIntoError};
#[sumtype(all = false, is = true)]
enum MySum {
    I(i64),
    B(bool),
}
let m = MySum::B(true);
assert!(!m.is_i());
```

This of course works at the level of variants, inheriting as usual

```rust
use typesum::{sumtype, TryIntoError};
#[sumtype(all = false)]
enum MySum {
    I(i64),
    #[sumtype(all)]
    B(bool),
}
let m = MySum::B(true);
assert_eq!(m.into_b(), Some(true));
```

But you are not allowed to have a sumtype annotation that doesn't do anything

```rust,compile_fail
use typesum::{sumtype, TryIntoError};
#[sumtype]
enum MySum {
    #[sumtype(ignore)]
    I(i64),
    #[sumtype(ignore)]
    B(bool),
}
```

In any way

```rust,compile_fail
use typesum::{sumtype, TryIntoError};
#[sumtype(all = false)]
enum MySum {
    I(i64),
    B(bool),
}
```

## TryInto and generic types

Because of the blanket impl on `TryInto` in the standard library, it is not possible to
implement `TryInto` for generic types. For this reason `TryInto` implementations will not
be generated for generic enum's.
