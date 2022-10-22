use std::cell::{Ref, RefCell};
use std::sync::{Arc, RwLock};

use cpal::traits::StreamTrait;

use crate::{Error, NamedDevice, Result, Stream};

/// Polling-based stream, can run in a background thread
/// and in other threads you poll data
pub struct PollingStream {
  stream: Option<Stream>,
  handle: Arc<RwLock<Vec<f32>>>,
  buf: RefCell<Vec<f32>>,
}

impl PollingStream {
  /// Creates a new [PollingStream] using a custom capacity
  ///
  /// capacity is used in 2 buffers of `Vec<f32>`,
  pub fn new(capacity: usize) -> Self {
    Self {
      stream: None,
      handle: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
      buf: RefCell::new(Vec::with_capacity(capacity)),
    }
  }

  pub fn change_to(&mut self, device: &NamedDevice) -> Result<()> {
    let handle = self.handle.clone();
    let stream = device.build_stream(
      move |data: &[f32], _: &_| {
        let mut handle = handle.write().unwrap();

        data.clone_into(&mut handle)
      },
      |err| eprintln!("{err}"),
    )?;

    stream.play()?;
    self.stream = Some(stream);

    Ok(())
  }

  pub fn pause(&self) -> Result<()> {
    if let Some(stream) = &self.stream {
      stream.pause()?;
    }

    Ok(())
  }

  pub fn poll(&self) -> Ref<Vec<f32>> {
    let handle = self.handle.try_read();

    if let Ok(handle) = handle {
      handle.clone_into(&mut self.buf.borrow_mut());
    }

    self.buf.borrow()
  }
}

impl Default for PollingStream {
  /// Creates a new [PollingStream] using a capacity of 1024
  fn default() -> Self {
    Self::new(1024)
  }
}

impl TryFrom<&NamedDevice> for PollingStream {
  type Error = Error;

  /// Creates a new [PollingStream] that automatically starts using [NamedDevice]
  ///
  /// **Example**
  /// ```rs
  /// let stream = PollingStream::try_from(&device)?;
  ///
  /// // ...
  /// let data = stream.poll();
  ///
  /// ```
  /// is the same as
  /// ```rs
  /// let mut stream = PollingStream::default();
  /// stream.change_to(device)?;
  ///
  /// // ...
  /// let data = stream.poll();
  ///
  fn try_from(device: &NamedDevice) -> std::result::Result<Self, Self::Error> {
    let mut stream = PollingStream::default();
    stream.change_to(device)?;
    Ok(stream)
  }
}