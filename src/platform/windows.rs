#![cfg(windows)]

use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Host, HostId, InputCallbackInfo, SampleFormat, SampleRate, Stream, StreamConfig};

use crate::{Device, Error, Listeners, Result};

pub struct WindowsHost {
  pub host: Host,
  pub devices: Vec<Device>,
  pub native_devices: HashMap<Device, cpal::Device>,
  pub listeners: Listeners,
  pub pending_device: Option<Device>,
  pub stream: Option<Stream>,
}

fn filter_device(device: cpal::Device) -> Option<(cpal::Device, Device)> {
  let name = device.name().ok()?;

  let input = device.default_input_config();
  let output = device.default_output_config();
  let supported = input.or(output).ok()?;

  if !(supported.channels() == 2 && supported.sample_format() == SampleFormat::F32) {
    return None;
  }

  let config = supported.config();
  let sample_rate = config.sample_rate.0;
  let buffer_size = match config.buffer_size {
    BufferSize::Fixed(size) => Some(size),
    BufferSize::Default => None,
  };

  Some((
    device,
    Device {
      name,
      sample_rate,
      buffer_size,
    },
  ))
}

fn filtered_devices(host: &Host) -> Result<(HashMap<Device, cpal::Device>, Vec<Device>)> {
  let default = host
    .default_output_device()
    .ok_or(Error::NoDefaultDeviceFound)?
    .name()?;

  let filtered = host.devices()?.filter_map(filter_device);
  let (lower, upper) = filtered.size_hint();
  let size = upper.unwrap_or(lower);
  let mut native_devices = HashMap::with_capacity(size);
  let mut devices = Vec::with_capacity(size);

  let mut default_index = 0;

  for (index, (native, device)) in filtered.enumerate() {
    if device.name == default {
      default_index = index;
    }

    devices.push(device.clone());
    native_devices.insert(device, native);
  }

  if !devices.is_empty() {
    devices.swap(0, default_index);
  }

  Ok((native_devices, devices))
}

impl WindowsHost {
  pub fn new() -> Result<Self> {
    let host = cpal::host_from_id(HostId::Wasapi)?;
    let (native_devices, devices) = filtered_devices(&host)?;
    let listeners = Listeners::new();

    Ok(Self {
      host,
      devices,
      native_devices,
      listeners,
      pending_device: None,
      stream: None,
    })
  }

  pub fn devices(&self) -> &Vec<Device> {
    &self.devices
  }

  fn _change_device(&mut self, device: &Device) -> Result<()> {
    let native = self
      .native_devices
      .get(device)
      .ok_or_else(|| Error::NoDeviceFound(device.name.to_owned()))?;

    let config = StreamConfig {
      channels: 2,
      sample_rate: SampleRate(device.sample_rate),
      buffer_size: device
        .buffer_size
        .map(BufferSize::Fixed)
        .unwrap_or(BufferSize::Default),
    };

    let data_cb = self.listeners.data_callback();
    let err_cb = |err| eprintln!("{err}");
    let stream = native.build_input_stream(&config, data_cb, err_cb)?;

    stream.play()?;

    self.stream = Some(stream);

    Ok(())
  }

  pub fn change_device(&mut self, device: &Device) -> Result<()> {
    if self.stream.is_some() {
      self._change_device(device)?;
    } else {
      self.pending_device = Some(device.clone());
    }

    Ok(())
  }

  pub fn listeners(&mut self) -> &mut Listeners {
    &mut self.listeners
  }

  pub fn listen(&mut self) -> Result<()> {
    match &self.pending_device.clone() {
      Some(device) => {
        self._change_device(device)?;
      }
      None => {
        if let Some(device) = &self.devices.first().cloned() {
          self._change_device(device)?;
        }
      }
    }

    Ok(())
  }

  pub fn refresh(&mut self) -> Result<()> {
    let (native_devices, devices) = filtered_devices(&self.host)?;

    self.devices = devices;
    self.native_devices = native_devices;

    Ok(())
  }
}
