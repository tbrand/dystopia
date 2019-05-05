use failure::Error;
use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum DatabaseError {
    #[fail(display = "invalid or unsupported database url={}", url)]
    InvalidDatabaseUrl { url: String },
}
