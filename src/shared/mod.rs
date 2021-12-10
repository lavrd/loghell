use std::error::Error;

pub type FnRes<T> = Result<T, Box<dyn Error>>;
