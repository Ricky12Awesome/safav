#![allow(clippy::type_complexity)]

use std::{
  any::TypeId,
  collections::{hash_map::Entry, HashMap},
  fmt::Debug,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering}, RwLock, RwLockReadGuard, RwLockWriteGuard,
  },
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
  fn update(&self, data: &[f32]);
}

impl_downcast!(AudioListenerTrait);

#[derive(Debug)]
pub struct AudioListener<T = Vec<f32>> {
  handle: Arc<RwLock<T>>,
  modify: Arc<RwLock<T>>,
  marked: Arc<AtomicBool>,
}

impl<T: AudioData> AudioListener<T> {
  fn new() -> Self {
    Self {
      handle: Default::default(),
      modify: Default::default(),
      marked: Default::default(),
    }
  }

  pub fn poll(&self) -> RwLockReadGuard<T> {
    if self.marked.load(Ordering::SeqCst) {
      *self.handle.write().unwrap() = self.modify.read().unwrap().clone();

      self.marked.store(false, Ordering::SeqCst)
    }

    self.handle.read().unwrap()
  }

  pub fn poll_mut(&self) -> RwLockWriteGuard<T> {
    let mut data = self.modify.write().unwrap();

    *data = self.handle.read().unwrap().clone();
    self.marked.store(true, Ordering::SeqCst);

    data
  }
}

impl<T: AudioData> Clone for AudioListener<T> {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle.clone(),
      modify: self.modify.clone(),
      marked: self.marked.clone(),
    }
  }
}

impl<T: AudioData> AudioListenerTrait for AudioListener<T> {
  fn update(&self, data: &[f32]) {
    let mut handle = self.handle.read().unwrap().clone();

    handle.update(data);

    *self.handle.write().unwrap() = handle;
  }
}

#[derive(Debug, Clone)]
pub struct Listener {
  handles: Arc<RwLock<HashMap<TypeId, Box<dyn AudioListenerTrait>>>>,
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
      e.insert(Box::new(AudioListener::<T>::new()));
    }

    let value = self.handles.read().unwrap();
    let value = &value[&id];
    let value = value.downcast_ref::<AudioListener<T>>().unwrap();

    value.clone()
  }

  pub(crate) fn callback(&self) -> DataCallback {
    let handles = self.handles.clone();

    DataCallback::new(move |data: &[f32], _: &_| {
      for handle in handles.read().unwrap().values() {
        handle.update(data);
      }
    })
  }
}
