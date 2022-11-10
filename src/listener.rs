use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub use cpal::InputCallbackInfo;

use crate::{Error, Result};

pub trait Listener: Sync + Send + 'static {
  fn listen(&mut self, data: &[f32], info: &InputCallbackInfo);
}

#[derive(Default)]
pub struct Listeners {
  listeners: Arc<RwLock<HashMap<&'static str, Box<dyn Listener>>>>,
}

impl Listeners {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&self, id: &'static str, listener: impl Listener) -> Result<()> {
    let mut listeners = self.listeners.write().unwrap();

    if listeners.contains_key(id) {
      return Err(Error::ListenerAlreadyExists(String::from(id)));
    }

    listeners.insert(id, Box::new(listener));

    Ok(())
  }

  pub fn remove(&self, id: &'static str) {
    let mut listeners = self.listeners.write().unwrap();

    listeners.remove(id);
  }

  pub(crate) fn data_callback(&self) -> impl FnMut(&[f32], &InputCallbackInfo) + Send + 'static {
    let handle = self.listeners.clone();

    move |data, info| {
      let mut listeners = handle.write().unwrap();

      for listener in listeners.values_mut() {
        listener.listen(data, info)
      }
    }
  }
}
