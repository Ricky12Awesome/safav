pub use error::*;
pub use listener::*;
pub use platform::*;

mod error;
mod listener;
mod platform;

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn get_application_name() -> Result<String> {
  std::env::args()
    .next()
    .as_ref()
    .map(std::path::Path::new)
    .and_then(std::path::Path::file_name)
    .and_then(std::ffi::OsStr::to_str)
    .map(String::from)
    .ok_or(Error::NoApplicationName)
}