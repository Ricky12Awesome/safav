use std::{thread::sleep, time::Duration};

use safav::{Host, Listener};

fn main() -> safav::Result<()> {
  let mut host = Host::new()?;
  let devices = host.devices();

  for dev in devices {
    println!("{dev}");
  }

  println!("Default Device: {}", host.default_device()?);

  let polling = Listener::default();

  host.listen()?;

  for _ in 0..100 {
    println!("{}", polling.poll().iter().sum::<f32>());
    sleep(Duration::from_millis(50));
  }

  Ok(())
}
