use cpal::traits::DeviceTrait;
use cpal::{InputCallbackInfo, StreamConfig, StreamError};

use crate::{Device, Result, Stream};

#[derive(Debug, Clone, Copy)]
pub enum DeviceSource {
  Input,
  Output,
  Both
}

pub struct NamedDevice {
  pub(crate) name: String,
  pub(crate) source: DeviceSource,
  pub(crate) config: StreamConfig,
  pub(crate) device: Device,
}

impl NamedDevice {
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn source(&self) -> DeviceSource {
    self.source
  }

  pub fn build_stream(
    &self,
    data_callback: impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static,
    error_callback: impl FnMut(StreamError) + Send + 'static,
  ) -> Result<Stream> {
    self
      .device
      .build_input_stream(&self.config, data_callback, error_callback)
      .map_err(Into::into)
  }
}
