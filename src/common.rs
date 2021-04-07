use std::error::Error;

pub type ErrorResult<T> = Result<T, Box<dyn Error>>;
