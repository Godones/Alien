use onlyerror::Error;

#[derive(Debug, Error)]
pub enum AlienError {
    #[error("no space")]
    NoSpace
}