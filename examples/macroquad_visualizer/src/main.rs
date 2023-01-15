#![feature(never_type, default_free_fn)]

use std::{default::default, f32::consts::TAU};

use egui_macroquad::egui::{ComboBox, Slider, Ui, Widget, Window};
use macroquad::prelude::*;

use safav::{Host, Listener, FFT};

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
      fft_size: 16384,
      fft_scale: 100.,
      wave_scale: 100.,
      radius: 350.,
      trim: 0,
      current_device: 0,
    }
  }
}

fn visualize(listener: &Listener, settings: &Settings, fft: &mut FFT) {
  let audio = listener.poll().clone();

  let (audio, scale) = if settings.is_fft {
    let audio = fft.process(&audio, settings.fft_size);
    let start = settings.trim.clamp(0, audio.len());
    let end = (audio.len() - settings.trim).clamp(start, audio.len());
    let trim = &audio[start..end];

    (Vec::from(trim), settings.fft_scale)
  } else {
    (audio, settings.wave_scale)
  };

  let len = audio.len();

  let center_w = screen_width() / 2f32;
  let center_h = screen_height() / 2f32;

  for i in 0..len {
    let value = audio[i] * scale;
    let theta = (TAU / len as f32) * i as f32;
    let radius = settings.radius;

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
      Color::from_rgba(230, 230, 230, 255),
    );
  }
}

fn ui(ui: &mut Ui, settings: &mut Settings, host: &Host) {
  ui.checkbox(&mut settings.is_fft, "FFT");

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

  Slider::new(&mut settings.radius, 1.0..=500.0)
    .text("Radius")
    .step_by(5.0)
    .ui(ui);

  ComboBox::from_label("Devices")
    .selected_text("Devices")
    .width(200.0)
    .show_ui(ui, |ui| {
      let devices = host.devices();
      let mut changed = settings.current_device;

      for (index, device) in devices.iter().enumerate() {
        ui.selectable_value(&mut changed, index, device.name());
      }

      if changed != settings.current_device {
        host.change_device_by_index(changed).unwrap();
      }
    });
}

async fn _main() -> safav::Result<!> {
  let mut settings = Settings::default();
  let mut host = Host::new()?;
  let mut fft = FFT::default();
  let listener = host.create_listener();

  host.listen()?;

  loop {
    clear_background(Color::from_rgba(32, 32, 32, 255));

    egui_macroquad::ui(|ctx| {
      Window::new("Settings")
        .resizable(true)
        .show(ctx, |ui| self::ui(ui, &mut settings, &host));
    });

    visualize(&listener, &settings, &mut fft);

    egui_macroquad::draw();

    next_frame().await;
  }
}
