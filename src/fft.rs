use dasp::window::{Hanning, Window};
use rustfft::{FftPlanner, num_complex::Complex32, num_traits::Zero};

#[derive(custom_debug::Debug)]
pub struct FFT<const BUF_SIZE: usize = 16384> {
  #[debug(skip)]
  planner: FftPlanner<f32>,
  zeros: [Complex32; BUF_SIZE],
  buffer: [Complex32; BUF_SIZE],
  scratch: [Complex32; BUF_SIZE],
  data: [f32; BUF_SIZE],
}

impl Default for FFT {
  fn default() -> Self {
    Self {
      planner: FftPlanner::new(),
      zeros: [Complex32::zero(); 16384],
      buffer: [Complex32::zero(); 16384],
      scratch: [Complex32::zero(); 16384],
      data: [0.; 16384],
    }
  }
}

impl<const BUF_SIZE: usize> FFT<BUF_SIZE> {
  pub fn new() -> Self {
    Self {
      planner: FftPlanner::new(),
      zeros: [Complex32::zero(); BUF_SIZE],
      buffer: [Complex32::zero(); BUF_SIZE],
      scratch: [Complex32::zero(); BUF_SIZE],
      data: [0.; BUF_SIZE],
    }
  }

  pub fn process(&mut self, buf: &[f32], size: usize) -> &[f32] {
    if size > BUF_SIZE {
      panic!("{size} is higher then max buf size of {BUF_SIZE}")
    }

    self.buffer[..size].copy_from_slice(&self.zeros[..size]);

    if buf.len() > size {
      let chunk_size = (buf.len() as f64 / size as f64).floor() as usize;

      let mut bins = buf
        .chunks(chunk_size)
        .map(|chunk: &[f32]| {
          chunk.iter().copied().map(Hanning::window).sum::<f32>() / chunk.len() as f32
        })
        .map(Complex32::from);

      for i in 0..size {
        self.buffer[i] = bins.next().unwrap_or_default();
      }
    } else {
      for i in 0..buf.len().min(size) {
        let value = Hanning::window(buf[i]);
        self.buffer[i] = Complex32::from(value);
      }
    }

    let max = (size as f32).sqrt();
    let fft = self.planner.plan_fft_forward(size);

    fft.process_with_scratch(&mut self.buffer[..size], &mut self.scratch[..fft.get_inplace_scratch_len()]);

    for i in 0..size {
      self.data[i] = self.buffer[i].re / max;
    }

    &self.data[..size]
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
