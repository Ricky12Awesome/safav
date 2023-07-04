use std::fmt::{Display, Formatter};

use crate::{AudioData, AudioListener, Listener, Result};

#[cfg(target_os = "linux")]
pub(crate) mod linux;

#[cfg(windows)]
pub(crate) mod windows;

pub struct Host {
  #[cfg(windows)]
  inner: windows::WindowsHost,
  #[cfg(target_os = "linux")]
  inner: linux::LinuxHost,
}

impl Host {
  pub fn new() -> Result<Self> {
    Ok(Self {
      #[cfg(windows)]
      inner: windows::WindowsHost::new()?,

      #[cfg(target_os = "linux")]
      inner: linux::LinuxHost::new()?,
    })
  }

  /// Gets the current device that is being listened too by index
  pub fn current_device_index(&self) -> Option<usize> {
    self.inner.current_device_index()
  }

  /// Gets the current device that is being listened too
  pub fn current_device(&self) -> Option<&Device> {
    self.inner.current_device()
  }

  /// Gets a list of the default device
  pub fn default_device(&self) -> Result<&Device> {
    self.inner.default_device()
  }

  /// Get a list of devices
  pub fn devices(&self) -> &Vec<Device> {
    self.inner.devices()
  }

  /// change the audio device to listen too by index of [Self::devices]
  pub fn change_device_by_index(&self, index: usize) -> Result<()> {
    self.inner.change_device_by_index(index)
  }

  /// Changes the audio device to listen too
  pub fn change_device(&self, device: &Device) -> Result<()> {
    self.inner.change_device(device)
  }

  /// Starts the listener to listen to audio
  pub fn listen(&mut self) -> Result<()> {
    self.inner.listen()
  }

  /// Creates a new listener that can be shared between threads since host itself can't be shared
  pub fn create_listener<T: AudioData>(&self) -> AudioListener<T> {
    self.inner.listener.clone().create()
  }

  /// Refreshes audio devices
  pub fn refresh(&mut self) -> Result<()> {
    self.inner.refresh()
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// represents an audio device
pub struct Device {
  /// on linux this would be `device.description`
  name: String,

  #[cfg(windows)]
  sample_rate: u32,

  #[cfg(windows)]
  buffer_size: Option<u32>,

  #[cfg(target_os = "linux")]
  /// PulseAudio device index
  index: u32,

  #[cfg(target_os = "linux")]
  /// on linux this would be `device.name`
  id: String,
}

impl Device {
  /// Gets the devices name
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl Display for Device {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(&self.name, f)
  }
}
