pub use error::*;
pub use listener::*;
pub use platform::*;

mod error;
mod listener;
mod platform;

pub type Result<T> = std::result::Result<T, Error>;
