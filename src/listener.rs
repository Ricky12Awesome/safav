use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub use cpal::InputCallbackInfo;

use crate::{Error, Result};

pub struct DataCallback {
  #[allow(clippy::type_complexity)]
  callback: Box<dyn FnMut(&[f32], &InputCallbackInfo) + Send + Sync + 'static>,
}

impl DataCallback {
  pub fn new(callback: impl FnMut(&[f32], &InputCallbackInfo) + Send + Sync + 'static) -> Self {
    Self {
      callback: Box::new(callback),
    }
  }
}

pub trait Listener {
  fn callback(&self) -> DataCallback;
}

#[derive(Default)]
pub struct Listeners {
  listeners: Arc<Mutex<HashMap<&'static str, DataCallback>>>,
}

impl Listeners {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn insert(&mut self, id: &'static str, handle: DataCallback) -> Result<()> {
    let mut listeners = self.listeners.lock().unwrap();

    if listeners.contains_key(id) {
      return Err(Error::ListenerAlreadyExists(String::from(id)));
    }

    listeners.insert(id, handle);

    Ok(())
  }

  pub fn remove(&mut self, id: &'static str) {
    let mut listeners = self.listeners.lock().unwrap();

    listeners.remove(id);
  }

  pub(crate) fn data_callback(&self) -> impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static {
    let handle = self.listeners.clone();

    move |data, info| {
      let mut listeners = handle.lock().unwrap();

      for listener in listeners.values_mut() {
        (listener.callback)(data, info);
      }
    }
  }
}

pub struct PollingListener {
  handle: Arc<RwLock<Vec<f32>>>,
  buf: RefCell<Vec<f32>>,
}

impl PollingListener {
  /// Creates a new [PollingListener] using a custom capacity
  ///
  /// capacity is used in 2 buffers of `Vec<f32>`,
  pub fn new(capacity: usize) -> Self {
    Self {
      handle: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
      buf: RefCell::new(Vec::with_capacity(capacity)),
    }
  }

  pub fn poll(&self) -> Ref<Vec<f32>> {
    let handle = self.handle.try_read();

    if let Ok(handle) = handle {
      let mut buf = self.buf.borrow_mut();

      buf.resize(handle.len(), 0.);
      buf.copy_from_slice(&handle);
    }

    self.buf.borrow()
  }
}

impl Listener for PollingListener {
  fn callback(&self) -> DataCallback {
    let handle = self.handle.clone();

    DataCallback::new(move |data: &[f32], _: &_| {
      let mut handle = handle.write().unwrap();

      handle.resize(data.len(), 0.);
      handle.copy_from_slice(data);
    })
  }
}

impl Default for PollingListener {
  /// Creates a new [PollingListener] using a capacity of 1024
  fn default() -> Self {
    Self::new(1024)
  }
}
