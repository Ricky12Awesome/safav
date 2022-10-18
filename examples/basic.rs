use std::io::Write;
use cpal::traits::StreamTrait;
use safav::DeviceManager;
use std::thread::sleep;
use std::time::Duration;

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

  let device = manager.default_loopback_device().unwrap();

  println!("Default [{:?}]: {}", device.source(), device.name());

  let stream = device.build_stream(
    |data, _| {
      print!("{} ", data.iter().sum::<f32>());
      let _ = std::io::stdout().flush();
    },
    |err| eprintln!("{err}"),
  )?;

  stream.play()?;

  sleep(Duration::from_secs(5));
  Ok(())
}
