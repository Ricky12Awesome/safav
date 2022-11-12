use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
  listeners: Arc<RwLock<HashMap<&'static str, DataCallback>>>,
}

impl Listeners {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&mut self, id: &'static str, handle: DataCallback) -> Result<()> {
    let mut listeners = self.listeners.write().unwrap();

    if listeners.contains_key(id) {
      return Err(Error::ListenerAlreadyExists(String::from(id)));
    }

    listeners.insert(id, handle);

    Ok(())
  }

  pub fn remove(&mut self, id: &'static str) {
    let mut listeners = self.listeners.write().unwrap();

    listeners.remove(id);
  }

  pub(crate) fn data_callback(&self) -> impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static {
    let handle = self.listeners.clone();

    move |data, info| {
      let mut listeners = handle.write().unwrap();

      for listener in listeners.values_mut() {
        (listener.callback)(data, info);
      }
    }
  }
}
