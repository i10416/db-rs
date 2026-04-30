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