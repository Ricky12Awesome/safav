use std::thread::sleep;
use std::time::Duration;

use safav::{Host, Listener, PollingListener};

fn main() -> safav::Result<()> {
  let mut host = Host::new()?;
  let devices = host.devices();

  for dev in devices {
    println!("{dev}");
  }

  let polling = PollingListener::default();

  host.listeners().insert_raw("test", polling.callback())?;

  host.listen()?;

  for _ in 0..100 {
    println!("{}", polling.poll().iter().sum::<f32>());
    sleep(Duration::from_millis(50));
  }

  Ok(())
}
