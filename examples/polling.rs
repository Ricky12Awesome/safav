use std::io::{stdout, Write};

use safav::{DeviceManager, PollingStream};

const ESC: char = '\x1b';

fn main() -> safav::Result<()> {
  let manager = DeviceManager::new()?;
  let device = manager.default_loopback_device().unwrap();
  let mut stream = PollingStream::new(1024);

  stream.change_to(device)?;

  loop {
    let sum = stream.poll().iter().sum::<f32>();

    print!("{ESC}[0J{ESC}[H{sum}");
    stdout().flush().unwrap();
  }
}
