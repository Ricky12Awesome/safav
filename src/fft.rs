use std::{
  any::type_name,
  fmt::{Debug, Formatter},
};

use dasp::window::{Hanning, Window};
use rustfft::{num_complex::Complex32, num_traits::Zero, FftPlanner};

pub struct FFT<const BUF_SIZE: usize = 16384> {
  planner: FftPlanner<f32>,
  buf: [f32; BUF_SIZE],
}

impl <const BUF_SIZE: usize> Debug for FFT<BUF_SIZE> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(type_name::<Self>())
      .field("planner", &"<no debug info>")
      .field("buf", &self.buf)
      .finish()
  }
}

impl Default for FFT {
  fn default() -> Self {
    Self {
      planner: FftPlanner::new(),
      buf: [0.; 16384],
    }
  }
}

impl<const BUF_SIZE: usize> FFT<BUF_SIZE> {
  pub fn new() -> Self {
    Self {
      planner: FftPlanner::new(),
      buf: [0.; BUF_SIZE],
    }
  }

  pub fn process(&mut self, buf: &[f32], size: usize) -> &[f32] {
    let max = (size as f32).sqrt();
    let fft = self.planner.plan_fft_forward(size);

    if size > BUF_SIZE {
      panic!("{size} is higher then max buf size of {BUF_SIZE}")
    }

    let mut buffer = [Complex32::zero(); BUF_SIZE];

    if buf.len() > size {
      let mut bins = buf
        .chunks((buf.len() as f64 / size as f64).floor() as usize)
        .map(|chunk: &[f32]| {
          chunk.iter().copied().map(Hanning::window).sum::<f32>() / chunk.len() as f32
        })
        .map(Complex32::from);

      for i in 0..size {
        buffer[i] = bins.next().unwrap_or_default();
      }
    } else {
      for i in 0..buf.len().min(size) {
        let value = Hanning::window(buf[i]);
        buffer[i] = Complex32::from(value);
      }
    }

    fft.process(&mut buffer[..size]);

    for i in 0..size {
      self.buf[i] = buffer[i].re / max;
    }

    &self.buf[..size]
  }
}

// pub fn fft(buf: &[f32], size: usize) -> Vec<f32> {
//   let mut planner = FftPlanner::<f32>::new();
//   let max = (size as f32).sqrt();
//   let fft = planner.plan_fft_forward(size);
//
//   let mut buffer = vec![Complex32::zero(); size];
//
//   if buf.len() > size {
//     let mut bins = buf
//       .chunks((buf.len() as f64 / size as f64).floor() as usize)
//       .map(|chunk: &[f32]| {
//         chunk.iter().copied().map(Hanning::window).sum::<f32>() / chunk.len() as f32
//       })
//       .map(Complex32::from);
//
//     for i in 0..size {
//       buffer[i] = bins.next().unwrap_or_default();
//     }
//   } else {
//     for i in 0..buf.len().min(size) {
//       let value = Hanning::window(buf[i]);
//       buffer[i] = Complex32::from(value);
//     }
//   }
//
//   fft.process(&mut buffer);
//
//   buffer
//     .iter()
//     .map(|c| c.re / max)
//     .collect()
// }
