#![feature(never_type)]

use std::{
  alloc::System,
  f32::consts::TAU,
  sync::{Arc, Mutex},
};

use alloc_tracker::{total_bytes_used, GlobalAllocTracker};
use imgui_macroquad::imgui::{Condition, SliderFlags, TreeNodeFlags, Ui};
use macroquad::{color::hsl_to_rgb, prelude::*};

use safav::{AudioData, AudioListener, Host, FFT};

#[global_allocator]
static ALLOCATOR: GlobalAllocTracker<System> = GlobalAllocTracker::new(System);

fn conf() -> Conf {
  Conf {
    window_title: String::from("Macroquad Visualizer Example"),
    window_width: 1000,
    window_height: 1000,
    ..Default::default()
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
    // let radius = radius;

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

fn ui(ui: &Ui, settings: &mut Settings, host: &Host) {
  ui.text(format!("Memory Usage: {} MB", total_bytes_used() / 1000000));
  ui.checkbox("RGB", &mut settings.is_rgb);

  let x_cap = screen_width() / 2.;
  let y_cap = screen_height() / 2.;

  if ui.collapsing_header("Circle", TreeNodeFlags::DEFAULT_OPEN) {
    ui.checkbox("Circle Enabled", &mut settings.is_circle);

    ui.slider("Circle Scale", 0.01, 5.0, &mut settings.circle_scale);
    ui.slider(
      "Circle Offset X",
      -x_cap,
      x_cap,
      &mut settings.circle_offset.0,
    );
    ui.slider(
      "Circle Offset Y",
      -y_cap,
      y_cap,
      &mut settings.circle_offset.1,
    );
    ui.slider("Circle Radius", 1., x_cap.min(y_cap), &mut settings.radius);
  }

  if ui.collapsing_header("Line", TreeNodeFlags::DEFAULT_OPEN) {
    ui.checkbox("Line Enabled", &mut settings.is_line);

    ui.slider("Line Scale", 0.01, 5.0, &mut settings.line_scale);
    ui.slider("Line Offset X", -x_cap, x_cap, &mut settings.line_offset.0);
    ui.slider("Line Offset Y", -y_cap, y_cap, &mut settings.line_offset.1);
  }

  if ui.collapsing_header("FFT", TreeNodeFlags::DEFAULT_OPEN) {
    ui.checkbox("Enabled (overrides waveform)", &mut settings.is_fft);

    ui.slider_config("FFT Size", 32, 16384)
      .flags(SliderFlags::ALWAYS_CLAMP)
      .build(&mut settings.fft_size);

    ui.slider("FFT Scale", 1., 3000., &mut settings.fft_scale);
    ui.slider("FFT Trim", 0, settings.fft_size / 4, &mut settings.trim);
    ui.slider("FFT Offset X", -x_cap, x_cap, &mut settings.line_offset.0);
    ui.slider("FFT Offset Y", -y_cap, y_cap, &mut settings.line_offset.1);
  }

  if ui.collapsing_header("Waveform", TreeNodeFlags::DEFAULT_OPEN) {
    ui.slider("Wave Scale", 1., 3500., &mut settings.wave_scale);
  }

  let devices = host.devices();

  let combo = ui.begin_combo("Devices", devices[settings.current_device].name());

  if let Some(combo) = combo {
    let mut selected = settings.current_device;

    for (index, device) in devices.iter().enumerate() {
      if index == selected {
        ui.set_item_default_focus();
      }

      if ui
        .selectable_config(device.name())
        .selected(index == selected)
        .build()
      {
        selected = index;
      }
    }

    if selected != settings.current_device {
      settings.current_device = selected;
      host.change_device_by_index(selected).unwrap();
    }

    combo.end();
  }
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
  let imgui = imgui_macroquad::get_imgui_context();
  let listener = host.create_listener();

  host.listen()?;

  loop {
    clear_background(Color::from_rgba(32, 32, 32, 255));

    imgui.setup_event_handler();

    imgui.ui(|ui| {
      ui.window("Settings")
        .size([300., 600.], Condition::FirstUseEver)
        .position([20., 20.], Condition::FirstUseEver)
        .build(|| self::ui(ui, &mut settings, &host));
    });

    update(&listener, &settings);
    visualize(&listener, &settings);

    imgui.draw();

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

#[allow(unused)]
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
