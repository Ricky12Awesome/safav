#![cfg(target_os = "linux")]

use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, HostId, Stream};
use pulsectl::controllers::types::ApplicationInfo;
use pulsectl::controllers::{AppControl, DeviceControl, SourceController};

use crate::{Device, Error, Listeners, Result};

pub struct LinuxHost {
  pub host: Host,
  pub devices: Vec<Device>,
  pub listeners: Listeners,
  pub stream: Option<Stream>,
  pub app: Option<ApplicationInfo>,
  pub current_device_index: Option<usize>,
}

fn get_application_name() -> Result<String> {
  std::env::args()
    .next()
    .as_ref()
    .map(std::path::Path::new)
    .and_then(std::path::Path::file_name)
    .and_then(std::ffi::OsStr::to_str)
    .map(String::from)
    .ok_or(Error::NoApplicationName)
}

fn devices() -> Result<Vec<Device>> {
  let mut controller = SourceController::create()?;
  let mut default = Some(0);
  let mut devices = controller
    .list_devices()?
    .iter()
    .enumerate()
    .filter_map(|(index, info)| {
      default = info.monitor.map(|_| index);

      Some(Device {
        name: info.description.clone()?,
        index: info.index,
        id: info.name.clone()?,
      })
    })
    .collect::<Vec<_>>();

  if let Some(default) = default {
    devices.swap(0, default);
  }

  Ok(devices)
}

impl LinuxHost {
  pub fn new() -> Result<Self> {
    let host = cpal::host_from_id(HostId::Alsa)?;

    Ok(Self {
      host,
      devices: devices()?,
      listeners: Listeners::new(),
      stream: None,
      app: None,
      current_device_index: None,
    })
  }

  pub fn current_device_index(&self) -> Option<usize> {
    self.current_device_index
  }

  pub fn current_device(&self) -> Option<&Device> {
    self.current_device_index.and_then(|i| self.devices.get(i))
  }

  pub fn default_device(&self) -> Result<&Device> {
    self.devices.first().ok_or(Error::NoDefaultDeviceFound)
  }

  pub fn devices(&self) -> &Vec<Device> {
    &self.devices
  }

  fn _get_app(&self, controller: &mut SourceController) -> Result<ApplicationInfo> {
    if let Some(app) = &self.app {
      return Ok(controller.get_app_by_index(app.index)?);
    };

    let name = get_application_name()?;
    let apps = controller.list_applications()?;

    Ok(
      apps
        .iter()
        .find(|app| app.proplist.get_str("application.name").as_ref() == Some(&name))
        .ok_or(Error::NoApplicationFound(name))?
        .to_owned(),
    )
  }

  fn _change_device(
    &self,
    controller: &mut SourceController,
    app: &ApplicationInfo,
    device: &Device,
  ) -> Result<()> {
    // Needs to have some delay, 20 ms seems to have no issues from my testing
    // don't know why this sometimes doesn't work if you do it too fast though
    sleep(Duration::from_millis(20));
    controller.move_app_by_index(app.index, device.index)?;

    Ok(())
  }

  pub fn change_device_by_index(&mut self, index: usize) -> Result<()> {
    let device = self
      .devices
      .get(index)
      .ok_or(Error::InvalidDeviceIndex(index))?
      .to_owned();

    self.change_device(&device)
  }

  pub fn change_device(&mut self, device: &Device) -> Result<()> {
    match &self.app {
      Some(app) => self._change_device(&mut SourceController::create()?, app, device)?,
      None => (),
    }

    self.current_device_index = self.devices.iter().position(|dev| dev == device);

    Ok(())
  }

  pub fn listeners(&mut self) -> &mut Listeners {
    &mut self.listeners
  }

  pub fn listen(&mut self) -> Result<()> {
    let device = self
      .host
      .default_input_device()
      .ok_or(Error::NoDefaultDeviceFound)?;

    let config = device.default_input_config()?.config();

    let data_cb = self.listeners.data_callback();
    let err_cb = |err| eprintln!("{err}");
    let stream = device.build_input_stream(&config, data_cb, err_cb)?;

    stream.play()?;

    self.stream = Some(stream);

    let mut controller = SourceController::create()?;
    let app = self._get_app(&mut controller)?;

    match self.current_device() {
      Some(device) => {
        self._change_device(&mut controller, &app, device)?;
      }
      None => {
        sleep(Duration::from_millis(10));
        let app = self._get_app(&mut controller)?;

        self.current_device_index = self
          .devices
          .iter()
          .position(|dev| dev.index == app.connection_id);
      }
    }

    self.app = Some(app);

    Ok(())
  }

  pub fn refresh(&mut self) -> Result<()> {
    let devices = devices()?;

    let current = self
      .current_device_index
      .and_then(|index| self.devices.get(index));

    let same_index = self
      .current_device_index
      .and_then(|index| devices.get(index));

    if current != same_index {
      if let Some(current) = current {
        self.current_device_index = devices.iter().position(|dev| dev == current);
      }
    }

    self.devices = devices;

    Ok(())
  }
}
