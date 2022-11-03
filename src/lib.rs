pub use device::*;
pub use device_manager::*;
pub use error::*;
pub use stream::*;
pub(crate) use platform::*;

mod device;
mod device_manager;
mod error;
mod stream;

pub(crate) mod platform {
  #[cfg(windows)]
  pub use windows::*;
  #[cfg(target_os = "linux")]
  pub use linux::*;

  #[cfg(windows)]
  pub mod windows {
    pub use cpal::platform::WasapiDevice as Device;
    pub use cpal::platform::WasapiDevices as Devices;
    pub use cpal::platform::WasapiHost as Host;
    pub use cpal::platform::WasapiStream as Stream;
    pub use cpal::platform::WasapiSupportedInputConfigs as SupportedInputConfigs;
    pub use cpal::platform::WasapiSupportedOutputConfigs as SupportedOutputConfigs;
  }

  #[cfg(target_os = "linux")]
  pub mod linux {
    pub use cpal::platform::AlsaDevice as Device;
    pub use cpal::platform::AlsaDevices as Devices;
    pub use cpal::platform::AlsaHost as Host;
    pub use cpal::platform::AlsaStream as Stream;
    pub use cpal::platform::AlsaSupportedInputConfigs as SupportedInputConfigs;
    pub use cpal::platform::AlsaSupportedOutputConfigs as SupportedOutputConfigs;
  }
}

pub type Result<T> = std::result::Result<T, Error>;
