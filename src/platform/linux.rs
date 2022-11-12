use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, HostId, Stream};
use pulsectl::controllers::types::ApplicationInfo;
use pulsectl::controllers::{AppControl, DeviceControl, SourceController};

use crate::{get_application_name, Device, Error, Listeners, Result};

pub struct LinuxHost {
  pub(crate) host: Host,
  pub(crate) devices: Vec<Device>,
  pub(crate) listeners: Listeners,
  pub(crate) stream: Option<Stream>,
  pub(crate) app: Option<ApplicationInfo>,
  pub(crate) pending_device: Option<Device>,
}

fn devices() -> Result<Vec<Device>> {
  let mut controller = SourceController::create()?;
  let devices = controller
    .list_devices()?
    .iter()
    .filter_map(|info| {
      Some(Device {
        name: info.description.clone()?,
        id: info.name.clone()?,
      })
    })
    .collect();

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
      pending_device: None,
    })
  }

  pub fn devices(&self) -> &Vec<Device> {
    &self.devices
  }

  pub fn change_device(&mut self, device: &Device) -> Result<()> {
    match &self.app {
      Some(app) => {
        let mut controller = SourceController::create()?;

        controller.move_app_by_name(app.index, &device.id)?;
      }
      None => self.pending_device = Some(device.to_owned()),
    }

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

    let name = get_application_name()?;
    let mut controller = SourceController::create()?;
    let apps = controller.list_applications()?;

    let app = apps
      .iter()
      .find(|app| app.proplist.get_str("application.name").as_ref() == Some(&name))
      .ok_or(Error::NoApplicationFound(name))?
      .to_owned();

    if let Some(device) = &self.pending_device {
      controller.move_app_by_name(app.index, &device.id)?;
    }

    self.app = Some(app);

    Ok(())
  }

  pub fn refresh(&mut self) -> Result<()> {
    self.devices = devices()?;

    Ok(())
  }
}
