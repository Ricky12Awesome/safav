use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub enum Error {
  HostUnavailable(#[from] cpal::HostUnavailable),
  DeviceError(#[from] cpal::DevicesError),
  DeviceNameError(#[from] cpal::DeviceNameError),
  StreamError(#[from] cpal::StreamError),
  BuildStreamError(#[from] cpal::BuildStreamError),
  DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),
  BackendSpecificError(#[from] cpal::BackendSpecificError),
  PauseStreamError(#[from] cpal::PauseStreamError),
  PlayStreamError(#[from] cpal::PlayStreamError),
  SupportedStreamConfigsError(#[from] cpal::SupportedStreamConfigsError),
  IoError(#[from] std::io::Error),

  #[error("Listener already exists for {0}")]
  ListenerAlreadyExists(String),

  #[error("Couldn't find a default device")]
  NoDefaultDeviceFound,

  #[error("Couldn't determine application name")]
  NoApplicationName,

  #[error("Couldn't find a device named '{0}'")]
  NoDeviceFound(String),

  #[cfg(target_os = "linux")]
  NoApplicationFound(String),

  #[cfg(target_os = "linux")]
  ControllerError(#[from] pulsectl::controllers::errors::ControllerError),
}
