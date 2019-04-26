use failure::Error;
use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum NodeError {
    #[fail(display = "failed to join to the cloud")]
    JoiningFailure,
}
