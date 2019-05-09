use failure::Error;
use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum CliError {
    #[fail(
        display = "invalid combination: component={}, method={}",
        component, method
    )]
    InvalidCombination { component: String, method: String },
}
