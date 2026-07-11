use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct TiError(pub String);
pub type TiResult<T = ()> = Result<T, TiError>;

pub trait SafeSub {
    fn safe_sub(&self, amount: usize) -> TiResult<usize>;
}

impl SafeSub for usize {
    fn safe_sub(&self, amount: usize) -> TiResult<usize> {
        self.checked_sub(amount)
            .ok_or_else(|| TiError(format!("Underflow: {} - {}", self, amount)))
    }
}
