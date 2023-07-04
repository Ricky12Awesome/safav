#![allow(clippy::type_complexity)]

use std::{
  any::TypeId,
  collections::{hash_map::Entry, HashMap},
  fmt::Debug,
  sync::{Arc, RwLock, RwLockReadGuard},
};

pub use cpal::InputCallbackInfo;
use downcast_rs::{impl_downcast, Downcast};

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

pub trait AudioData: Default + Debug + Send + Sync + Sized + 'static {
  fn update(&mut self, data: &[f32]);
}

impl AudioData for Vec<f32> {
  fn update(&mut self, data: &[f32]) {
    self.resize(data.len(), 0.);
    self.copy_from_slice(data);
  }
}

#[derive(Debug, Clone)]
pub struct Listener {
  handles: Arc<RwLock<HashMap<TypeId, Box<dyn AudioListenerTrait>>>>,
}

pub(crate) trait AudioListenerTrait: Downcast + Send + Sync + Debug + 'static {
  fn update(&mut self, data: &[f32]);
}

impl_downcast!(AudioListenerTrait);

#[derive(Debug)]
pub struct AudioListener<T = Vec<f32>> {
  handle: Arc<RwLock<T>>,
}

impl<T: AudioData> AudioListener<T> {
  fn new() -> Self {
    Self {
      handle: Default::default(),
    }
  }

  pub fn poll(&self) -> RwLockReadGuard<T> {
    self.handle.read().unwrap()
  }
}

impl<T: AudioData> Clone for AudioListener<T> {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.clone(),
    }
  }
}

impl<T: AudioData> AudioListenerTrait for AudioListener<T> {
  fn update(&mut self, data: &[f32]) {
    let mut handle = self.handle.write().unwrap();

    handle.update(data)
  }
}

impl Listener {
  pub(crate) fn new() -> Self {
    Self {
      handles: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub fn create<T: AudioData>(&self) -> AudioListener<T> {
    let id = TypeId::of::<T>();

    let mut handles = self.handles.write().unwrap();

    if let Entry::Vacant(e) = handles.entry(id) {
      e.insert(Box::new(AudioListener::<T>::new()));
    }

    let value = handles[&id].downcast_ref::<AudioListener<T>>().unwrap();

    value.clone()
  }

  pub(crate) fn callback(&self) -> DataCallback {
    let handles = self.handles.clone();

    DataCallback::new(move |data: &[f32], _: &_| {
      let mut handles = handles.write().unwrap();

      for handle in handles.values_mut() {
        handle.update(data);
      }
    })
  }
}
