## `std::ops::Index` & `std::ops::IndexMut`

By implementing `std::ops::Index` & `std::ops::IndexMut` for a type,
we can perform index access to internal data on the type.


## thiserror crate

It enables implicit conversion from an error of type T into custom error type by annotation on enum type.

```rust
#[derive(thiserror::Error)]
pub enum MyError {
  #[error(transparent)]
  Io(#[from] io::Error)
}
```

## anyhow crate

In library code, we usually define informative errors as enum while
In application code, we do not care much about concrete error types and we want
to wrap various error types in a generic error type for convenience.

The anyhow crate serves the latter purpose. Without anyhow crate, we have to define
a custom error enum that covers all the possible error types in the application.

So, every function in naivedb-kernel and naivedb-bplustree-storage returns dedicated error types(`Result<T,E>`)
while every function in naivedb returns a generic error type `anyhow::Result<T>`.

`anyhow::Error` provides several functionality similar to Exception in other languages such as stacktrace or error context.