use safav::DeviceManager;

fn main() -> safav::Result<()> {
  let manager = DeviceManager::new()?;

  println!(
    "{:#?}",
    manager
      .devices()
      .iter()
      .map(|it| format!("[{:?}]: {}", it.source(), it.name()))
      .collect::<Vec<_>>()
  );

  let input = manager.default_input_device();
  let output = manager.default_loopback_device();

  match input {
    None => println!("No default input device found"),
    Some(device) => println!("Default [{:?}]: {}", device.source(), device.name()),
  }

  match output {
    None => println!("No default output (loopback) device found"),
    Some(device) => println!("Default [{:?}]: {}", device.source(), device.name()),
  }

  Ok(())
}
