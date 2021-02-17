use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear},
    Result,
};
use palette::{FromColor, Gradient, Hsv, IntoColor, LinSrgb};

use rand::SeedableRng;
use rand::{distributions::Alphanumeric, prelude::ThreadRng};
use rand::{thread_rng, Rng};
use std::{collections::HashMap, io::Stdout, iter, thread};
use std::{
    io::{stdout, Write},
    time::Instant,
};
struct Line {
    items: Vec<String>,
    length: u16,
    x: u16,
    y: f64,
    speed: f64,
    last_update: Instant,
}
#[derive(Clone, PartialEq)]
struct ScreenItem {
    fg_color: Color,
    bg_color: Color,
    text: String,
}
struct Screen {
    buffer: HashMap<(u16, u16), ScreenItem>,
    terminal: HashMap<(u16, u16), ScreenItem>,
    stdout: Stdout,
    last_known_size: (u16, u16),
    already_flushed: bool,
}
impl Screen {
    pub fn new() -> Self {
        let ret = Self {
            stdout: stdout(),
            terminal: HashMap::new(),
            buffer: HashMap::new(),
            last_known_size: terminal::size().unwrap(),
            already_flushed: false,
        };
        ret
    }
    pub fn flush(&mut self) -> Result<u32> {
        let mut writes = 0;
        let (columns, rows) = terminal::size()?;
        if self.last_known_size != (columns, rows) || !self.already_flushed {
            for y in 0..rows {
                for x in 0..columns {
                    if let Some(item) = self.buffer.get(&(x, y)) {
                        queue!(
                            self.stdout,
                            SetForegroundColor(item.fg_color),
                            SetBackgroundColor(item.bg_color),
                            MoveTo(x, y),
                            Print(&item.text),
                            ResetColor
                        )?;
                        writes += 1;
                    } else {
                        queue!(self.stdout, ResetColor, MoveTo(x, y), Print(" "))?;
                    }
                }
            }
        } else {
            let mut sorted_buffer = self.buffer.iter().collect::<Vec<_>>();
            sorted_buffer.sort_unstable_by(|a, b| a.0 .1.partial_cmp(&b.0 .1).unwrap());
            for ((x, y), item) in sorted_buffer.iter() {
                if x < &columns
                    && y < &rows
                    && match self.terminal.get(&(*x, *y)) {
                        None => true,
                        Some(x) => x != *item,
                    }
                {
                    queue!(
                        self.stdout,
                        SetForegroundColor(item.fg_color),
                        SetBackgroundColor(item.bg_color),
                        MoveTo(*x, *y),
                        Print(&item.text),
                        ResetColor
                    )?;
                    writes += 1;
                }
            }

            for ((x, y), item) in &self.terminal {
                if !self.buffer.contains_key(&(*x, *y)) && x < &columns && y < &rows {
                    queue!(self.stdout, ResetColor, MoveTo(*x, *y), Print(" "))?;
                    writes += 1;
                }
            }
        }
        self.terminal = self.buffer.clone();
        self.last_known_size = (columns, rows);
        queue!(self.stdout, MoveTo(0, rows)).unwrap();
        self.buffer = HashMap::new();
        self.already_flushed = true;
        self.stdout.flush()?;
        Ok(writes)
    }
    pub fn set(&mut self, x: u16, y: u16, item: ScreenItem) {
        let mut y = y;
        let mut mx = 0;
        for c in item.text.chars().into_iter() {
            match c.to_string().as_ref() {
                "\n" => {
                    y += 1;
                    mx = 0;
                }
                _ => {
                    self.buffer.insert(
                        (x + mx, y),
                        ScreenItem {
                            text: c.to_string(),
                            ..item
                        },
                    );
                    mx += 1;
                }
            }
        }
    }
    pub fn clear(&mut self, color: Color) -> Result<()> {
        let (columns, rows) = terminal::size()?;
        for y in 0..=rows {
            for x in 0..=columns {
                self.set(
                    x,
                    y,
                    ScreenItem {
                        bg_color: color,
                        fg_color: Color::Reset,
                        text: " ".into(),
                    },
                );
            }
        }
        Ok(())
    }
}
impl Drop for Screen {
    fn drop(&mut self) {
        let (_, rows) = terminal::size().unwrap();
        queue!(self.stdout, MoveTo(0, rows)).unwrap();
    }
}

fn main() -> Result<()> {
    let mut last_frame = Instant::now();
    let mut time_since_last_disp = Instant::now();
    let bg_color = Color::Rgb { r: 0, g: 13, b: 5 };
    let mut frametime = 0;
    let mut fps = 0;
    let mut lines: Vec<Line> = vec![];
    let mut time_since_last_spawn = Instant::now();
    let mut rng = thread_rng();
    let mut screen = Screen::new();
    let mut writes = 0;

    loop {
        let frame_time_ns = last_frame.elapsed().as_nanos();
        last_frame = Instant::now();
        let (columns, rows) = terminal::size()?;
        screen.clear(bg_color)?;
        for mut line in &mut lines {
            for item in 0..(line.length) {
                let grad = Gradient::new(vec![
                    Hsv::new(141.0, 1.0, 0.43),
                    //Hsv::new(141.0, 1.0, 0.43),
                    Hsv::new(141.0, 0.97, 0.05),
                ])
                .take(line.length as usize)
                .map(|n| LinSrgb::from(n))
                .collect::<Vec<_>>();
                let y = (line.y - item as f64).round() - 1.0;
                if y >= 0.0 && y <= rows as f64 - 1.0 {
                    screen.set(
                        line.x,
                        y as u16,
                        ScreenItem {
                            bg_color,
                            fg_color: match item {
                                0 => Color::White,
                                _ => Color::Rgb {
                                    r: (grad[item as usize].red as f32 * 255.0).round() as u8,
                                    g: (grad[item as usize].green as f32 * 255.0).round() as u8,
                                    b: (grad[item as usize].blue as f32 * 255.0).round() as u8,
                                },
                            },
                            text: if line.items.len() <= y as usize {
                                "a".into()
                            } else {
                                line.items[y as usize].clone()
                            },
                        },
                    );
                }
            }
            line.y += line.speed * (line.last_update.elapsed().as_micros() as f64 / 100_000.0);
            line.last_update = Instant::now();
        }
        screen.set(
            0,
            rows - 5,
            ScreenItem {
                fg_color: Color::Blue,
                bg_color,
                text: format!(
                    "{} fps\n{} lines\nFrametime: {}ns\n{} screen writes per frame\nterminal is {} rows, {} columns",
                    fps,
                    lines.len(),
                    frametime,
                    writes,
                    rows,
                    columns
                ),
            },
        );
        if time_since_last_disp.elapsed().as_millis() >= 150 {
            frametime = frame_time_ns;
            fps = 1_000_000_000 / frame_time_ns;
            time_since_last_disp = Instant::now();
        }
        if time_since_last_spawn.elapsed().as_millis() >= 10 {
            let column = rng.gen_range(0..columns);
            lines.push(Line {
                items: (0..rows)
                    .map(|row| {
                        rand::rngs::StdRng::seed_from_u64(row as u64 * column as u64)
                            .sample(Alphanumeric)
                    })
                    .map(char::from)
                    .map(String::from)
                    .collect(),
                x: column,
                length: 20,
                y: 0.0,
                speed: rng.gen_range(0.2..3.0),
                last_update: Instant::now(),
            });
            lines = lines
                .into_iter()
                .filter(|line| !(line.y - line.length as f64 > rows as f64))
                .collect();
            time_since_last_spawn = Instant::now();
        }
        if time_since_last_disp.elapsed().as_millis() >= 150 {
            writes = screen.flush()?;
        } else {
            screen.flush()?;
        }
    }
    Ok(())
}
