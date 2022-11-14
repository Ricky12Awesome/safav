use std::fmt::{Display, Formatter};

use crate::{Listeners, Result};

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
  #[cfg(windows)]
  pub fn new() -> Result<Self> {
    Ok(Self {
      inner: windows::WindowsHost::new()?,
    })
  }

  #[cfg(target_os = "linux")]
  pub fn new() -> Result<Self> {
    Ok(Self {
      inner: linux::LinuxHost::new()?,
    })
  }

  pub fn current_device_index(&self) -> Option<usize> {
    self.inner.current_device_index()
  }

  pub fn current_device(&self) -> Option<&Device> {
    self.inner.current_device()
  }

  pub fn default_device(&self) -> Result<&Device> {
    self.inner.default_device()
  }

  pub fn devices(&self) -> &Vec<Device> {
    self.inner.devices()
  }

  pub fn change_device_by_index(&mut self, index: usize) -> Result<()> {
    self.inner.change_device_by_index(index)
  }
  
  pub fn change_device(&mut self, device: &Device) -> Result<()> {
    self.inner.change_device(device)
  }

  pub fn listeners(&mut self) -> &mut Listeners {
    self.inner.listeners()
  }

  pub fn listen(&mut self) -> Result<()> {
    self.inner.listen()
  }

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
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl Display for Device {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(&self.name, f)
  }
}
