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
use std::iter;
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

fn main() -> Result<()> {
    let mut stdout = stdout();
    let (columns, rows) = terminal::size()?;
    let mut last_frame = Instant::now();
    let mut time_since_last_disp = Instant::now();
    execute!(stdout, Clear(crossterm::terminal::ClearType::All))?;
    let bg_color = Color::Rgb { r: 0, g: 13, b: 5 };
    for row in 0..=rows {
        for column in 0..(columns + 1) {
            queue!(
                stdout,
                MoveTo(column, row),
                SetBackgroundColor(bg_color),
                Print(" ")
            )?;
        }
    }
    let mut frametime = 0;
    let mut fps = 0;
    let mut lines: Vec<Line> = vec![];
    let mut time_since_last_spawn = Instant::now();
    let mut rng = thread_rng();

    loop {
        for mut line in &mut lines {
            let len = line.length + 1;
            for item in 0..len {
                let grad =
                    Gradient::new(vec![Hsv::new(141.0, 1.0, 0.43), Hsv::new(141.0, 1.0, 0.0)])
                        .take(len as usize)
                        .map(|n| LinSrgb::from(n))
                        .collect::<Vec<_>>();
                let y = (line.y - item as f64).round() - 1.0;
                if y >= 0.0 && y <= rows as f64 - 1.0 {
                    queue!(
                        stdout,
                        SetForegroundColor(match item {
                            0 => Color::White,
                            _ => Color::Rgb {
                                r: (grad[item as usize].red as f32 * 255.0).round() as u8,
                                g: (grad[item as usize].green as f32 * 255.0).round() as u8,
                                b: (grad[item as usize].blue as f32 * 255.0).round() as u8,
                            },
                        }),
                        SetBackgroundColor(bg_color),
                        MoveTo(line.x, y as u16),
                        Print(if item < line.length {
                            &line.items[y as usize]
                        } else {
                            " "
                        }),
                        ResetColor
                    )?;
                }
            }
            line.y += line.speed * (line.last_update.elapsed().as_micros() as f64 / 100_000.0);
            line.last_update = Instant::now();
        }
        queue!(
            stdout,
            SetForegroundColor(Color::Blue),
            SetBackgroundColor(Color::Red),
            MoveTo(0, rows - 2),
            Print(format!("Frametime: {}ns    ", frametime)),
            MoveTo(0, rows - 1),
            Print(format!("{} fps    ", fps)),
            ResetColor
        )?;
        if time_since_last_disp.elapsed().as_millis() >= 150 {
            frametime = last_frame.elapsed().as_nanos();
            fps = 1_000_000_000 / last_frame.elapsed().as_nanos();
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
            time_since_last_spawn = Instant::now();
        }
        stdout.flush()?;
        last_frame = Instant::now();
    }
    //Ok(())
}
