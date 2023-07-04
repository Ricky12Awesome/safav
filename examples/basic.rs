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

  let listener = host.create_listener::<CustomData>();

  host.listen()?;

  let timer = Instant::now();
  let seconds = 5;
  let duration = Duration::from_secs(seconds);
  let mut counter = 0;

  while timer.elapsed() <= duration {
    let data = listener.poll();

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

#[derive(Debug)]
struct CustomData {
  fft: Vec<Mutex<FFT>>,
  data: Vec<f32>,
}

impl Default for CustomData {
  fn default() -> Self {
    Self {
      fft: vec![Default::default()],
      data: Vec::with_capacity(16384),
    }
  }
}

impl AudioData for CustomData {
  fn update(&mut self, data: &[f32]) {
    for i in 0..128 {
      let mut replace = false;
      let mut add = false;

      match self.fft[i].try_lock() {
        Ok(mut fft) => {
          let data = fft.process(data, 16384);

          self.data.resize(16384, 0.);
          self.data.copy_from_slice(data);
          break;
        }
        Err(TryLockError::Poisoned(_)) => replace = true,
        Err(TryLockError::WouldBlock) => add = true,
      }

      if replace {
        self.fft[i] = Default::default();
      }

      if add && i < 128 {
        self.fft.push(Default::default())
      }
    }
  }
}
