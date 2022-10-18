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
}
