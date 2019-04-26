use failure::Error;
use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum NodeStateError {
    #[fail(display = "invalid state={}", s)]
    InvalidState { s: String },
}

#[derive(Debug, Fail)]
pub enum AuditError {
    #[fail(display = "invalid audit")]
    InvalidAudit,
}
