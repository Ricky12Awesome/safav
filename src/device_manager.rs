use cpal::SampleFormat;
use cpal::traits::HostTrait;

use crate::{Device, Host, Result};
use crate::device::{DeviceSource, NamedDevice};

#[derive(Debug)]
pub struct DeviceManager {
  host: Host,
  devices: Vec<NamedDevice>,
  current_device_index: usize,
}

fn filter_device(device: Device) -> Option<NamedDevice> {
  let name = device.name().ok()?;

  let input = device.default_input_config();
  let output = device.default_output_config();
  let is_input = input.is_ok();
  let supported = input.or(output).unwrap();

  if !(supported.channels() == 2 && supported.sample_format() == SampleFormat::F32) {
    return None;
  }

  let config = supported.config();
  let source = if is_input {
    DeviceSource::Input
  } else {
    DeviceSource::Output
  };

  Some(NamedDevice {
    name,
    source,
    config,
    device,
  })
}

fn filtered_devices(host: &Host) -> Result<Vec<NamedDevice>> {
  Ok(host.devices()?.filter_map(filter_device).collect())
}

impl DeviceManager {
  pub fn new() -> Result<Self> {
    let host = Host::new()?;
    let devices = filtered_devices(&host)?;

    Ok(Self {
      host,
      devices,
      current_device_index: 0,
    })
  }

  pub fn default_loopback_device(&self) -> Option<&NamedDevice> {
    let device = self.host.default_output_device()?;
    let name = device.name().ok()?;

    self.devices.iter().find(|device| device.name == name)
  }

  pub fn default_input_device(&self) -> Option<&NamedDevice> {
    let device = self.host.default_input_device()?;
    let name = device.name().ok()?;

    self.devices.iter().find(|device| device.name == name)
  }

  pub fn devices(&self) -> &Vec<NamedDevice> {
    &self.devices
  }

  pub fn refresh_devices(&mut self) -> Result<()> {
    self.devices = filtered_devices(&self.host)?;

    Ok(())
  }
}
