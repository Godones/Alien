use onlyerror::Error;

#[derive(Debug, Error)]
pub enum AlienError {
    #[error("no space")]
    NoSpace,
    #[error("no memory")]
    Other,
}

pub type AlienResult<T> = Result<T, AlienError>;
