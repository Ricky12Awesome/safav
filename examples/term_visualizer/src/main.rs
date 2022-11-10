use std::cmp::Ordering;
use std::io::{stdout, Write};

use colored::Color;
use palette::rgb::Rgb;
use palette::{FromColor, Hsl, RgbHue};

use safav::{LinuxHost, PollingStream};

const ESC: char = '\x1b';

#[derive(Debug, Clone)]
struct Grid<'a> {
  pixels: Vec<Vec<Option<Color>>>,
  color: Color,
  width: usize,
  height: usize,
  pixel: &'a str,
}

impl<'a> Grid<'a> {
  fn new(width: usize, height: usize, pixel: &'a str, color: Color) -> Self {
    Self {
      pixels: vec![vec![Some(color); width]; height],
      color,
      width,
      height,
      pixel,
    }
  }

  fn clear(&mut self) {
    self.pixels = vec![vec![Some(self.color); self.width]; self.height];
  }

  fn update_size(&mut self, width: usize, height: usize) {
    self.width = width;
    self.height = height;
    self.clear();
  }

  fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
    let y = y.min(self.height.saturating_sub(1));
    let x = x.min(self.width.saturating_sub(1));

    self.pixels[y][x] = Some(color);
  }

  fn render(&mut self) -> std::io::Result<()> {
    let mut out = stdout();

    out.write_all(format!("{ESC}[{}u", self.width).as_bytes())?;
    out.write_all(format!("{ESC}[{}t", self.height).as_bytes())?;
    out.write_all(format!("{ESC}[?25l").as_bytes())?;

    for (y, line) in self.pixels.iter_mut().enumerate() {
      let y = y + 1;
      for (x, color) in line.iter_mut().enumerate() {
        let x = x + 1;

        if let Some(c) = color {
          let fg = c.to_fg_str();
          out.write_all(format!("{ESC}[{y};{x}H{ESC}[{fg}m{}", self.pixel).as_bytes())?;

          if matches!(color, Some(c) if *c == self.color) {
            *color = None;
          } else {
            *color = Some(self.color);
          }
        }
      }

      if y < self.height {
        out.write_all(b"\n")?;
      }

      out.write_all(format!("{ESC}[;H").as_bytes())?;
      out.flush()?;
    }

    Ok(())
  }
}

fn main() -> safav::Result<()> {
  let manager = LinuxHost::new()?;
  let device = manager.default_loopback_device().unwrap();
  let mut stream = PollingStream::new(1024);

  stream.change_to(device)?;

  let mut grid = Grid::new(0, 0, "█", Color::Black);
  let mut max = 0.0;

  loop {
    let values = stream.poll();
    let termsize::Size { rows, cols } = termsize::get().unwrap();
    let rows = rows as usize;
    let cols = cols as usize;

    if grid.width != cols || grid.height != rows {
      grid.update_size(cols, rows);
    }

    let size = values.len() / cols;

    if size == 0 {
      continue;
    }

    let new_max = *values
      .iter()
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap();

    if new_max > max {
      max = new_max;
    }

    let values = values
      .chunks(size)
      .map(|section| section.iter().sum::<f32>() / (max * section.len() as f32))
      .map(|val| {
        let hue = RgbHue::from_degrees(120. * val);
        let hsl = Hsl::new(hue, 1.0, 0.5);
        let (r, g, b) = Rgb::from_color(hsl).into_format::<u8>().into_components();

        (val, Color::TrueColor { r, g, b })
      })
      .enumerate();

    for (index, (val, color)) in values {
      let rows = rows as f32;
      let y = ((rows / 2.) + (val * 0.5) * rows).floor() as usize;

      grid.set_pixel(index, y, color);
    }

    grid.render()?;

    // sleep(Duration::from_millis(1000 / 60));
  }

  Ok(())
}
