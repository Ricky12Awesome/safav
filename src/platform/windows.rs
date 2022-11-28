#![cfg(windows)]

use std::cell::RefCell;
use std::collections::HashMap;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Host, HostId, SampleFormat, SampleRate, Stream, StreamConfig};

use crate::{Device, Error, Listener, Result};

pub struct WindowsHost {
  pub host: Host,
  pub devices: Vec<Device>,
  pub native_devices: HashMap<Device, cpal::Device>,
  pub listener: Listener,
  pub current_device_index: RefCell<Option<usize>>,
  pub stream: RefCell<Option<Stream>>,
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
    let listeners = Listener::default();

    Ok(Self {
      host,
      devices,
      native_devices,
      listener: listeners,
      current_device_index: RefCell::new(None),
      stream: RefCell::new(None),
    })
  }

  pub fn current_device_index(&self) -> Option<usize> {
    *self.current_device_index.borrow()
  }

  pub fn current_device(&self) -> Option<&Device> {
    self
      .current_device_index()
      .and_then(|i| self.devices.get(i))
  }

  pub fn default_device(&self) -> Result<&Device> {
    self.devices.first().ok_or(Error::NoDefaultDeviceFound)
  }

  pub fn devices(&self) -> &Vec<Device> {
    &self.devices
  }

  fn _change_stream(&self, device: &Device) -> Result<()> {
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

    let data_cb = self.listener.callback().get();
    let err_cb = |err| eprintln!("{err}");
    let stream = native.build_input_stream(&config, data_cb, err_cb)?;

    stream.play()?;

    *self.stream.borrow_mut() = Some(stream);

    Ok(())
  }

  fn _get_device_index(&self, device: &Device) -> Option<usize> {
    self.devices.iter().position(|dev| dev == device)
  }

  pub fn change_device_by_index(&self, index: usize) -> Result<()> {
    let device = self
      .devices
      .get(index)
      .ok_or(Error::InvalidDeviceIndex(index))?
      .to_owned();

    self.change_device(&device)
  }

  pub fn change_device(&self, device: &Device) -> Result<()> {
    if self.stream.borrow().is_some() {
      self._change_stream(device)?;
    }

    *self.current_device_index.borrow_mut() = self._get_device_index(device);

    Ok(())
  }

  pub fn listen(&mut self) -> Result<()> {
    match self.current_device_index() {
      Some(index) => {
        if let Some(device) = self.devices.get(index) {
          self._change_stream(device)?;
        }
      }
      None => {
        if let Ok(device) = self.default_device() {
          self._change_stream(device)?;

          *self.current_device_index.borrow_mut() = self._get_device_index(device);
        }
      }
    }

    Ok(())
  }

  pub fn refresh(&mut self) -> Result<()> {
    let (native_devices, devices) = filtered_devices(&self.host)?;

    let current = self
      .current_device_index()
      .and_then(|index| self.devices.get(index));

    let same_index = self
      .current_device_index()
      .and_then(|index| devices.get(index));

    if current != same_index {
      if let Some(current) = current {
        *self.current_device_index.borrow_mut() = self._get_device_index(current)
      }
    }

    self.devices = devices;
    self.native_devices = native_devices;

    Ok(())
  }
}
