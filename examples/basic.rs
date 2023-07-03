use std::time::{Duration, Instant};

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
  let seconds = 5;
  let duration = Duration::from_secs(seconds);
  let mut counter = 0;

  while timer.elapsed() <= duration {
    let _data = polling.poll();
    counter += 1;
  }

  println!(
    "Total of {} in {:.0?} [{}/s]",
    counter,
    timer.elapsed(),
    counter / seconds,
  );

  Ok(())
}
