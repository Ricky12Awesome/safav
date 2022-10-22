use std::io::{stdout, Write};

use safav::{DeviceManager, PollingStream};

fn main() -> safav::Result<()> {
  // Create a device manager instance
  let manager = DeviceManager::new()?;
  // Get default loopback device (output device)
  let device = manager.default_loopback_device().unwrap();
  // Create a polling based stream using that device
  // this is the short-hand way of doing it
  let stream = PollingStream::try_from(device)?;

  loop {
    // then you poll the most recent data from the stream
    let sum = stream.poll().iter().sum::<f32>();

    print!("\x1b[0J\x1b[H{sum}");
    stdout().flush().unwrap();
  }
}
