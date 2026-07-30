use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct TiError(pub String);
pub type TiResult<T = ()> = Result<T, TiError>;
