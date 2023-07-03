use std::{
  thread::sleep,
  time::{Duration, Instant},
};

use safav::Host;

fn main() -> safav::Result<()> {
  let mut host = Host::new()?;
  let devices = host.devices();

  for dev in devices {
    println!("{dev}");
  }

  println!("Default Device: {}", host.default_device()?);

  let polling = host.create_listener();

  host.listen()?;

  let timer = Instant::now();

  for _ in 0..100000 {
    let e = polling.poll();
  }

  println!("{:?}", timer.elapsed());

  Ok(())
}
