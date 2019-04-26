use failure::Error;
use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum RequestError {
    #[fail(display = "failed to lookup ip address of {}", host)]
    LookupFailure { host: String },

    #[fail(display = "invalid request (request host not found)")]
    HostNotFound,

    #[fail(display = "invalid request (request path not found)")]
    PathNotFound,
}
