#![allow(clippy::type_complexity)]

use std::{
  sync::{Arc, RwLock, RwLockReadGuard},
};

pub use cpal::InputCallbackInfo;

pub struct DataCallback {
  callback: Box<dyn FnMut(&[f32], &InputCallbackInfo) + Send + Sync + 'static>,
}

impl DataCallback {
  pub fn new(callback: impl FnMut(&[f32], &InputCallbackInfo) + Send + Sync + 'static) -> Self {
    Self {
      callback: Box::new(callback),
    }
  }

  pub fn get(self) -> Box<dyn FnMut(&[f32], &InputCallbackInfo) + Send + Sync + 'static> {
    self.callback
  }
}

#[derive(Debug, Clone)]
pub struct Listener {
  handle: Arc<RwLock<Vec<f32>>>,
}

impl Listener {
  pub(crate) fn new(capacity: usize) -> Self {
    Self {
      handle: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
    }
  }

  pub fn poll(&self) -> RwLockReadGuard<Vec<f32>> {
    self.handle.read().unwrap()
  }

  pub(crate) fn callback(&self) -> DataCallback {
    let handle = self.handle.clone();

    DataCallback::new(move |data: &[f32], _: &_| {
      let mut handle = handle.write().unwrap();

      handle.resize(data.len(), 0.);
      handle.copy_from_slice(data);
    })
  }
}
