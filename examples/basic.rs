use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use safav::{DataCallback, Host, Listener};

fn main() -> safav::Result<()> {
  let mut host = Host::new()?;
  let devices = host.devices();

  for dev in devices {
    println!("{dev}");
  }

  let test = Test::default();

  host.listeners().push("test", test.callback())?;

  host.listen()?;

  sleep(Duration::from_secs(20));

  Ok(())
}

#[derive(Default)]
struct Test {
  buf: Arc<RwLock<Vec<f32>>>
}

impl Listener for Test {
  fn callback(&self) -> DataCallback {
    let buf = self.buf.clone();

    DataCallback::new(move |data, _| {
      let mut buf = buf.write().unwrap();

      buf.resize(data.len(), 0.0);
      buf.copy_from_slice(data);

      println!("{}", data.iter().sum::<f32>());
    })
  }
}
