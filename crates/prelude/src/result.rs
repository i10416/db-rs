pub fn swap<S, T>(result: Result<S, T>) -> Result<T, S> {
    match result {
        Ok(s) => Err(s),
        Err(t) => Ok(t),
    }
}
