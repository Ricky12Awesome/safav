use cpal::platform::WasapiDevice as Device;
use cpal::platform::WasapiDevices as Devices;
use cpal::platform::WasapiHost as Host;
use cpal::platform::WasapiStream as Stream;
use cpal::platform::WasapiSupportedInputConfigs as SupportedInputConfigs;
use cpal::platform::WasapiSupportedOutputConfigs as SupportedOutputConfigs;

// pub struct DeviceManager {
//   host: Host,
//   devices: Vec<NamedDevice>,
// }
//
// fn filter_device(device: Device) -> Option<NamedDevice> {
//   let name = device.name().ok()?;
//
//   let input = device.default_input_config();
//   let output = device.default_output_config();
//   let is_input = input.is_ok();
//   let is_output = output.is_ok();
//   let supported = input.or(output).ok()?;
//
//   if !(supported.channels() == 2 && supported.sample_format() == SampleFormat::F32) {
//     return None;
//   }
//
//   let config = supported.config();
//   let source = match () {
//     _ if is_input && is_output => DeviceSource::Both,
//     _ if is_input => DeviceSource::Input,
//     _ if is_output => DeviceSource::Output,
//     _ => unreachable!()
//   };
//
//   Some(NamedDevice {
//     name,
//     source,
//     config,
//     device,
//   })
// }
//
// fn filtered_devices(host: &Host) -> Result<Vec<NamedDevice>> {
//   Ok(host.devices()?.filter_map(filter_device).collect())
// }
//
// impl DeviceManager {
//   pub fn new() -> Result<Self> {
//     let host = Host::new()?;
//     let devices = filtered_devices(&host)?;
//
//     Ok(Self {
//       host,
//       devices,
//     })
//   }
//
//   pub fn default_loopback_device(&self) -> Option<&NamedDevice> {
//     let device = self.host.default_output_device()?;
//     let name = device.name().ok()?;
//
//     self.devices.iter().find(|device| device.name == name)
//   }
//
//   pub fn default_input_device(&self) -> Option<&NamedDevice> {
//     let device = self.host.default_input_device()?;
//     let name = device.name().ok()?;
//
//     self.devices.iter().find(|device| device.name == name)
//   }
//
//   pub fn devices(&self) -> &Vec<NamedDevice> {
//     &self.devices
//   }
//
//   pub fn refresh_devices(&mut self) -> Result<()> {
//     self.devices = filtered_devices(&self.host)?;
//
//     Ok(())
//   }
// }