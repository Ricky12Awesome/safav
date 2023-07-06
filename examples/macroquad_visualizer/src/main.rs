#![feature(never_type, default_free_fn)]

use std::{
  default::default,
  f32::consts::TAU,
  sync::{Arc, Mutex},
};

use egui_macroquad::egui::{CollapsingHeader, ComboBox, Slider, Ui, Widget, Window};
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
  is_circle: bool,
  is_rgb: bool,
  fft_size: usize,
  fft_scale: f32,
  wave_scale: f32,
  line_scale: f32,
  circle_scale: f32,
  line_offset: (f32, f32),
  circle_offset: (f32, f32),
  radius: f32,
  trim: usize,
  current_device: usize,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      is_fft: true,
      is_line: false,
      is_circle: true,
      is_rgb: false,
      fft_size: 16384,
      fft_scale: 100.,
      wave_scale: 100.,
      line_scale: 2.0,
      circle_scale: 0.5,
      line_offset: (0., 0.),
      circle_offset: (0., 0.),
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

fn line(audio: &[f32], (offset_x, offset_y): (f32, f32), scale: f32, is_rgb: bool) {
  let len = audio.len();
  let width = offset_x + (screen_width() / len as f32);
  let center_h = offset_y + (screen_height() / 2f32);

  for i in 0..len {
    let value = audio[i] * scale;

    let x_inner = width * i as f32;
    let y_inner = center_h - (value / 2f32);
    let x_outer = width * i as f32;
    let y_outer = center_h + (value / 2f32);

    draw_line(
      x_inner,
      y_inner,
      x_outer,
      y_outer,
      width,
      color(is_rgb, i as f32, len as f32),
    );
  }
}

fn circle(audio: &[f32], (offset_x, offset_y): (f32, f32), scale: f32, radius: f32, is_rgb: bool) {
  let len = audio.len();
  let center_w = offset_x + (screen_width() / 2f32);
  let center_h = offset_y + (screen_height() / 2f32);

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
    let audio = &audio.fft;
    let start = settings.trim.clamp(0, audio.len());
    let end = (audio.len() - settings.trim).clamp(start, audio.len());
    let trim = &audio[start..end];

    (trim, settings.fft_scale)
  } else {
    (audio.wave.as_slice(), settings.wave_scale)
  };

  if settings.is_line {
    line(
      audio,
      settings.line_offset,
      scale * settings.line_scale,
      settings.is_rgb,
    );
  }

  if settings.is_circle {
    circle(
      audio,
      settings.circle_offset,
      scale * settings.circle_scale,
      settings.radius,
      settings.is_rgb,
    );
  }
}

fn ui(ui: &mut Ui, settings: &mut Settings, host: &Host) {
  ui.checkbox(&mut settings.is_rgb, "RGB");

  let x_cap = screen_width() / 2.;
  let y_cap = screen_height() / 2.;

  CollapsingHeader::new("Circle")
    .default_open(true)
    .show(ui, |ui| {
      ui.checkbox(&mut settings.is_circle, "Enabled");

      Slider::new(&mut settings.circle_scale, 0.01..=5.0)
        .text("Scale")
        .step_by(0.05)
        .ui(ui);

      Slider::new(&mut settings.circle_offset.0, -x_cap..=x_cap)
        .text("Offset X")
        .step_by(5.0)
        .ui(ui);

      Slider::new(&mut settings.circle_offset.1, -y_cap..=y_cap)
        .text("Offset Y")
        .step_by(5.0)
        .ui(ui);

      Slider::new(&mut settings.radius, 1.0..=x_cap.min(y_cap))
        .text("Radius")
        .step_by(5.0)
        .ui(ui);
    });

  CollapsingHeader::new("Line")
    .default_open(true)
    .show(ui, |ui| {
      ui.checkbox(&mut settings.is_line, "Enabled");

      Slider::new(&mut settings.line_scale, 0.01..=5.0)
        .text("Scale")
        .step_by(0.05)
        .ui(ui);

      Slider::new(&mut settings.line_offset.0, -x_cap..=x_cap)
        .text("Offset X")
        .step_by(5.0)
        .ui(ui);

      Slider::new(&mut settings.line_offset.1, -y_cap..=y_cap)
        .text("Offset Y")
        .step_by(5.0)
        .ui(ui);
    });

  CollapsingHeader::new("FFT")
    .default_open(true)
    .show(ui, |ui| {
      ui.checkbox(&mut settings.is_fft, "Enabled (overrides waveform)");

      Slider::new(&mut settings.fft_size, 32..=16384)
        .text("FFT Size")
        .logarithmic(true)
        .step_by(16.)
        .ui(ui);

      Slider::new(&mut settings.fft_scale, 1.0..=3000.0)
        .text("FFT Scale")
        .step_by(5.)
        .ui(ui);

      Slider::new(&mut settings.trim, 0..=settings.fft_size / 4)
        .text("Trim")
        .logarithmic(true)
        .step_by(1.0)
        .ui(ui);
    });

  CollapsingHeader::new("Waveform")
    .default_open(true)
    .show(ui, |ui| {
      Slider::new(&mut settings.wave_scale, 1.0..=3500.0)
        .text("Wave Scale")
        .step_by(5.0)
        .ui(ui);
    });

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

fn update(listener: &AudioListener<CustomData>, settings: &Settings) {
  let audio = listener.poll();

  if audio.fft_size != settings.fft_size {
    let mut audio = listener.poll_mut();
    audio.fft_size = settings.fft_size;
  }
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

    update(&listener, &settings);
    visualize(&listener, &settings);

    egui_macroquad::draw();

    next_frame().await;
  }
}

#[derive(Clone, Debug)]
struct CustomData {
  // FFT isn't "thread safe" so it needs to be in a mutex,
  // also needs to be in Arc since it doesn't implement Clone
  planner: Arc<Mutex<FFT>>,
  wave: Vec<f32>,
  fft: Vec<f32>,
  fft_size: usize,
}

impl CustomData {
  fn set_size(&mut self, size: usize) {
    self.fft_size = size;
  }
}

impl Default for CustomData {
  fn default() -> Self {
    Self {
      planner: Default::default(),
      fft_size: 16384,
      fft: Vec::with_capacity(16384),
      wave: Vec::with_capacity(16384),
    }
  }
}

impl AudioData for CustomData {
  fn update(&mut self, data: &[f32]) {
    self.wave.update(data);

    let mut fft = self.planner.lock().unwrap();
    let data = fft.process(data, self.fft_size);

    self.fft.resize(self.fft_size, 0.);
    self.fft.copy_from_slice(data);
  }
}
