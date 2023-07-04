#![allow(clippy::type_complexity)]

use std::{
  any::TypeId,
  collections::{hash_map::Entry, HashMap},
  fmt::Debug,
  sync::{Arc, RwLock, RwLockReadGuard, TryLockError},
};

pub use cpal::InputCallbackInfo;
use downcast_rs::{Downcast, impl_downcast};

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

pub trait AudioData: Default + Clone + Debug + Send + Sync + Sized + 'static {
  fn update(&mut self, data: &[f32]);
}

impl AudioData for Vec<f32> {
  fn update(&mut self, data: &[f32]) {
    self.resize(data.len(), 0.);
    self.copy_from_slice(data);
  }
}

pub(crate) trait AudioListenerTrait: Downcast + Send + Sync + Debug + 'static {
  fn update(&mut self, data: &[f32]);

  fn clone_box(&self) -> Box<dyn AudioListenerTrait>;
}

impl_downcast!(AudioListenerTrait);

#[derive(Debug)]
pub struct AudioListener<T = Vec<f32>> {
  handle: Arc<RwLock<T>>,
  backup: Arc<RwLock<T>>,
}

impl<T: AudioData> AudioListener<T> {
  fn new() -> Self {
    Self {
      handle: Default::default(),
      backup: Default::default(),
    }
  }

  pub fn poll(&self) -> RwLockReadGuard<T> {
    match self.handle.try_read() {
      Ok(data) => {
        *self.backup.write().unwrap() = data.clone();
        data
      }
      Err(TryLockError::WouldBlock) => self.backup.read().unwrap(),
      Err(TryLockError::Poisoned(err)) => {
        panic!("{err}")
      }
    }
  }
}

impl<T: AudioData> Clone for AudioListener<T> {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.clone(),
      backup: self.backup.clone(),
    }
  }
}

impl<T: AudioData> AudioListenerTrait for AudioListener<T> {
  fn update(&mut self, data: &[f32]) {
    let mut handle = self.handle.read().unwrap().clone();

    handle.update(data);

    *self.handle.write().unwrap() = handle;
  }

  fn clone_box(&self) -> Box<dyn AudioListenerTrait> {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone)]
pub struct Listener {
  handles: Arc<RwLock<HashMap<TypeId, Arc<RwLock<Box<dyn AudioListenerTrait>>>>>>,
}

impl Listener {
  pub(crate) fn new() -> Self {
    Self {
      handles: Default::default(),
    }
  }

  pub fn create<T: AudioData>(&mut self) -> AudioListener<T> {
    let id = TypeId::of::<T>();

    if let Entry::Vacant(e) = self.handles.write().unwrap().entry(id) {
      e.insert(Arc::new(RwLock::new(Box::new(AudioListener::<T>::new()))));
    }

    let value = self.handles.read().unwrap();
    let value = value[&id].read().unwrap();
    let value = value.downcast_ref::<AudioListener<T>>().unwrap();

    value.clone()
  }

  pub(crate) fn callback(&self) -> DataCallback {
    let handles = self.handles.clone();

    DataCallback::new(move |data: &[f32], _: &_| {
      for handle in handles.read().unwrap().values() {
        let mut handle = handle.read().unwrap().clone_box();
        handle.update(data);
      }
    })
  }
}
