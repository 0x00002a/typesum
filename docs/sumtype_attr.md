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

```
use typesum::{sumtype, TryIntoError};
#[sumtype(impl_try_into)]
enum MySum {
    I(i64),
    B(bool),
}
let v = MySum::B(true);
let r: Result<i64, _> = v.try_into();
assert_eq!(r, Err(TryIntoError::new("MySum", "i64")));

```
