## anyhow crate

In library code, we usually define informative errors as enum while
In application code, we do not care much about concrete error types and we want
to wrap various error types in a generic error type for convenience.

The anyhow crate serves the latter purpose. Without anyhow crate, we have to define
a custom error enum that covers all the possible error types in the application.

So, every function in naivedb-kernel and naivedb-bplustree-storage returns dedicated error types(`Result<T,E>`)
while every function in naivedb returns a generic error type `anyhow::Result<T>`.

`anyhow::Error` provides several functionality similar to Exception in other languages such as stacktrace or error context.