pub use error::*;
pub use fft::*;
pub use listener::*;
pub use platform::*;

mod error;
mod fft;
mod listener;
mod platform;

pub type Result<T> = std::result::Result<T, Error>;
