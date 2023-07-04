use std::{
  sync::{Arc, Mutex, TryLockError, TryLockResult},
  time::{Duration, Instant},
};

use safav::{AudioData, Host, FFT};

fn main() -> safav::Result<()> {
  let mut host = Host::new()?;
  let devices = host.devices();

  for dev in devices {
    println!("{dev}");
  }

  println!("Default Device: {}", host.default_device()?);

  let thread = host.create_listener::<CustomData>();
  let main = host.create_listener::<Vec<f32>>();

  host.listen()?;

  let timer = Instant::now();
  let seconds = 5;
  let duration = Duration::from_secs(seconds);
  let mut counter = 0;

  while timer.elapsed() <= duration {
    let _data = thread.poll();

    counter += 1;
  }

  println!(
    "[Thread] Total of {} in {:.0?} [{}/s]",
    counter,
    timer.elapsed(),
    counter / seconds,
  );

  let timer = Instant::now();
  let seconds = 5;
  let duration = Duration::from_secs(seconds);
  let mut fft = FFT::default();
  let mut counter = 0;

  while timer.elapsed() <= duration {
    let data = main.poll();
    let _fft = fft.process(&data, 16384);

    counter += 1;
  }

  println!(
    "[Main] Total of {} in {:.0?} [{}/s]",
    counter,
    timer.elapsed(),
    counter / seconds,
  );

  Ok(())
}

#[derive(Clone, Debug)]
struct CustomData {
  fft: Arc<Mutex<FFT>>,
  data: Vec<f32>,
}

impl Default for CustomData {
  fn default() -> Self {
    Self {
      fft: Default::default(),
      data: Vec::with_capacity(16384),
    }
  }
}

impl AudioData for CustomData {
  fn update(&mut self, data: &[f32]) {
    let mut fft = self.fft.lock().unwrap();

    let data = fft.process(data, 16384);

    self.data.resize(16384, 0.);
    self.data.copy_from_slice(data);
  }
}
