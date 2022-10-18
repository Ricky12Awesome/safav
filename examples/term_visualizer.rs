use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use colored::Color;
use palette::rgb::Rgb;
use palette::{FromColor, Hsl, RgbHue};

use safav::{DeviceManager, PollingStream};

const ESC: char = '\x1b';

#[derive(Debug, Default, Clone)]
struct Grid<'a> {
  pixels: Vec<Vec<Color>>,
  width: usize,
  height: usize,
  pixel: &'a str,
}

impl<'a> Grid<'a> {
  fn new(width: usize, height: usize, pixel: &'a str, color: Color) -> Self {
    Self {
      pixels: vec![vec![color; width]; height],
      width,
      height,
      pixel,
    }
  }

  fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
    let y = y.min(self.height.saturating_sub(1));
    let x = x.min(self.width.saturating_sub(1));

    self.pixels[y][x] = color;
  }

  fn render(&self) -> std::io::Result<()> {
    let mut out = stdout();

    out.write_all(format!("{ESC}[{}u", self.width).as_bytes())?;
    out.write_all(format!("{ESC}[{}t", self.height).as_bytes())?;

    for (y, line) in self.pixels.iter().enumerate() {
      let y = y + 1;
      for (x, color) in line.iter().enumerate() {
        let x = x + 1;
        let fg = color.to_fg_str();

        out.write_all(format!("{ESC}[{y};{x}H{ESC}[{fg}m{}", self.pixel).as_bytes())?;
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

impl<'a> Display for Grid<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    for y in &self.pixels {
      for color in y {
        let fg = color.to_fg_str();
        write!(f, "{ESC}[{fg}m{}", self.pixel)?;
      }

      writeln!(f)?;
    }

    Ok(())
  }
}

fn main() -> safav::Result<()> {
  let manager = DeviceManager::new()?;
  let device = manager.default_loopback_device().unwrap();
  let mut stream = PollingStream::new(1024);

  stream.change_to(device)?;

  loop {
    let values = stream.poll();
    let termsize::Size { rows, cols } = termsize::get().unwrap();
    let rows = rows as usize;
    let cols = cols as usize;

    let mut grid = Grid::new(cols, rows, "█", Color::Black);

    let size = values.len() / cols;

    if size == 0 {
      continue;
    }

    let max = values
      .iter()
      .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
      .unwrap();

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

    // print!("{ESC}[H{}", grid);
    grid.render()?;
    // stdout().flush().unwrap();

    // sleep(Duration::from_millis(1000 / 60));
  }
}
