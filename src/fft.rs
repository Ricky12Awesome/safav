use dasp::window::{Hanning, Window};
use rustfft::{
  num_complex::{Complex32, ComplexFloat},
  num_traits::Zero,
  FftPlanner,
};

pub fn fft(buf: &[f32], size: usize) -> Vec<f32> {
  let mut planner = FftPlanner::<f32>::new();
  let max = (size as f32).sqrt();
  let fft = planner.plan_fft_forward(size);

  let mut buffer = vec![Complex32::zero(); size];

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

  fft.process(&mut buffer);

  buffer
    .iter()
    .map(|c| c.re / max)
    .collect()
}
