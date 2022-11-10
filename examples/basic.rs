use std::thread::sleep;
use std::time::Duration;

use safav::{Host, InputCallbackInfo, Listener};

fn main() -> safav::Result<()> {
  let mut host = Host::new()?;
  let devices = host.devices();

  for dev in devices {
    println!("{dev}");
  }

  host.listeners().push("test", Test)?;

  host.listen()?;

  sleep(Duration::from_secs(20));

  Ok(())
}

struct Test;

impl Listener for Test {
  fn listen(&mut self, _data: &[f32], _: &InputCallbackInfo) {
    // println!("{}", data.iter().sum::<f32>())
  }
}
