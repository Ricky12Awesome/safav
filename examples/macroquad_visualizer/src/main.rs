#![feature(never_type, default_free_fn)]

use std::{
  default::default,
  f32::consts::TAU,
  sync::{Arc, Mutex, RwLock},
};

use egui_macroquad::egui::{ComboBox, Slider, Ui, Widget, Window};
use macroquad::{color::hsl_to_rgb, prelude::*};

use safav::{AudioData, AudioListener, Host, FFT};

fn conf() -> Conf {
  Conf {
    window_title: String::from("Macroquad Visualizer Example"),
    window_width: 1000,
    window_height: 1000,
    ..default()
  }
}

#[macroquad::main(conf)]
async fn main() {
  _main().await.unwrap();
}

#[derive(Debug)]
struct Settings {
  is_fft: bool,
  is_line: bool,
  is_rgb: bool,
  fft_size: usize,
  fft_scale: f32,
  wave_scale: f32,
  radius: f32,
  trim: usize,
  current_device: usize,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      is_fft: true,
      is_line: false,
      is_rgb: false,
      fft_size: 16384,
      fft_scale: 100.,
      wave_scale: 100.,
      radius: 350.,
      trim: 0,
      current_device: 0,
    }
  }
}

fn color(is_rgb: bool, index: f32, size: f32) -> Color {
  if is_rgb {
    hsl_to_rgb((1f32 / size) * index, 1f32, 0.5f32)
  } else {
    Color::from_rgba(230, 230, 230, 255)
  }
}

fn line(audio: &[f32], scale: f32, is_rgb: bool) {
  let len = audio.len();
  let width = screen_width() / len as f32;
  let center_h = screen_height() / 2f32;

  for i in 0..len {
    let value = audio[i] * scale;

    draw_line(
      width * i as f32,
      center_h - (value / 2f32),
      width * i as f32,
      center_h + (value / 2f32),
      width,
      color(is_rgb, i as f32, len as f32),
    );
  }
}

fn circle(audio: &[f32], scale: f32, radius: f32, is_rgb: bool) {
  let len = audio.len();
  let center_w = screen_width() / 2f32;
  let center_h = screen_height() / 2f32;

  for i in 0..len {
    let value = audio[i] * scale;
    let theta = (TAU / len as f32) * i as f32;
    let radius = radius;

    let x_inner = center_w + (radius - value) * theta.sin();
    let y_inner = center_h - (radius - value) * theta.cos();
    let x_outer = center_w + (radius + value) * theta.sin();
    let y_outer = center_h - (radius + value) * theta.cos();

    draw_line(
      x_inner,
      y_inner,
      x_outer,
      y_outer,
      2.,
      color(is_rgb, i as f32, len as f32),
    );
  }
}

fn visualize(listener: &AudioListener<CustomData>, settings: &Settings) {
  let audio = listener.poll();

  let (audio, scale) = if settings.is_fft {
    audio.set_size(settings.fft_size);

    let audio = audio.fft.clone();
    let start = settings.trim.clamp(0, audio.len());
    let end = (audio.len() - settings.trim).clamp(start, audio.len());
    let trim = &audio[start..end];

    (Vec::from(trim), settings.fft_scale)
  } else {
    (audio.wave.clone(), settings.wave_scale)
  };

  if settings.is_line {
    line(&audio, scale, settings.is_rgb);
  } else {
    circle(&audio, scale, settings.radius, settings.is_rgb);
  }
}

fn ui(ui: &mut Ui, settings: &mut Settings, host: &Host) {
  ui.checkbox(&mut settings.is_fft, "FFT");
  ui.checkbox(&mut settings.is_line, "Line");
  ui.checkbox(&mut settings.is_rgb, "RGB");

  if settings.is_fft {
    Slider::new(&mut settings.fft_size, 32..=16384)
      .text("FFT Size")
      .logarithmic(true)
      .step_by(16.)
      .ui(ui);

    Slider::new(&mut settings.fft_scale, 1.0..=500.0)
      .text("FFT Scale")
      .step_by(5.)
      .ui(ui);

    Slider::new(&mut settings.trim, 0..=settings.fft_size / 4)
      .text("Trim")
      .logarithmic(true)
      .step_by(1.0)
      .ui(ui);
  } else {
    Slider::new(&mut settings.wave_scale, 1.0..=500.0)
      .text("Wave Scale")
      .step_by(5.0)
      .ui(ui);
  }

  if !settings.is_line {
    Slider::new(&mut settings.radius, 1.0..=500.0)
      .text("Radius")
      .step_by(5.0)
      .ui(ui);
  }

  let devices = host.devices();

  ComboBox::from_label("Devices")
    .selected_text(format!("{:.28}", devices[settings.current_device].name()))
    .width(200.0)
    .show_ui(ui, |ui| {
      let mut changed = settings.current_device;

      for (index, device) in devices.iter().enumerate() {
        ui.selectable_value(&mut changed, index, device.name());
      }

      if changed != settings.current_device {
        settings.current_device = changed;
        host.change_device_by_index(changed).unwrap();
      }
    });
}

async fn _main() -> safav::Result<!> {
  let mut settings = Settings::default();
  let mut host = Host::new()?;
  let listener = host.create_listener();

  host.listen()?;

  loop {
    clear_background(Color::from_rgba(32, 32, 32, 255));

    egui_macroquad::ui(|ctx| {
      Window::new("Settings")
        .resizable(true)
        .show(ctx, |ui| self::ui(ui, &mut settings, &host));
    });

    visualize(&listener, &settings);

    egui_macroquad::draw();

    next_frame().await;
  }
}

#[derive(Clone, Debug)]
struct CustomData {
  planner: Arc<Mutex<FFT>>,
  size: Arc<RwLock<usize>>,
  wave: Vec<f32>,
  fft: Vec<f32>,
}

impl CustomData {
  fn set_size(&self, size: usize) {
    *self.size.write().unwrap() = size.clamp(32, 16384);
  }
}

impl Default for CustomData {
  fn default() -> Self {
    Self {
      planner: Default::default(),
      size: Arc::new(RwLock::new(16384)),
      fft: Vec::with_capacity(16384),
      wave: Vec::with_capacity(16384),
    }
  }
}

impl AudioData for CustomData {
  fn update(&mut self, data: &[f32]) {
    let mut fft = self.planner.lock().unwrap();
    let size = *self.size.read().unwrap();

    self.wave.resize(data.len(), 0.);
    self.wave.copy_from_slice(data);

    let data = fft.process(data, size);

    self.fft.resize(size, 0.);
    self.fft.copy_from_slice(data);
  }
}
