//! Renders the render sheets in `docs/gallery`: every mood, every size and
//! every streak mark, side by side, plus one GIF per mood.
//!
//! ```text
//! cargo run --example gallery --no-default-features
//! ```

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use git_snowman::render::{FRAMES, FRAME_DELAY, HEIGHT, WIDTH};
use git_snowman::{
    draw_frame, encode_gif, render_gif, Colors, Frame, Marks, Mood, Rgb, Size, Snowman,
};

const GAP: usize = 2;
const SCALE: u16 = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = Path::new("docs/gallery");
    fs::create_dir_all(out)?;

    let rusty = Colors {
        scarf: Rgb::hex(0xdea584),
        hat: Rgb::hex(0x3572a5),
        buttons: Rgb::hex(0x1c1c24),
    };
    let base = Snowman {
        size: Size::Large,
        colors: rusty,
        ..Snowman::default()
    };

    let moods: Vec<Snowman> = Mood::ALL
        .into_iter()
        .map(|mood| Snowman {
            mood,
            ..base.clone()
        })
        .collect();
    sheet(&moods, &out.join("moods.gif"))?;

    let sizes: Vec<Snowman> = Size::ALL
        .into_iter()
        .map(|size| Snowman {
            size,
            ..base.clone()
        })
        .collect();
    sheet(&sizes, &out.join("sizes.gif"))?;

    let marks: Vec<Snowman> = [(0, 0), (7, 7), (30, 30), (120, 120)]
        .into_iter()
        .map(|(cur, long)| Snowman {
            size: Size::Grand,
            marks: Marks::from_streaks(cur, long),
            ..base.clone()
        })
        .collect();
    sheet(&marks, &out.join("marks.gif"))?;

    for s in &moods {
        let path = out.join(format!("{}.gif", s.mood));
        render_gif(s, 4, BufWriter::new(File::create(&path)?))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

/// Lays snowmen out in a row, frame by frame, and writes one GIF.
fn sheet(snowmen: &[Snowman], path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let w = snowmen.len() * WIDTH + (snowmen.len() - 1) * GAP;
    let frames: Vec<Frame> = (0..FRAMES)
        .map(|t| {
            let mut f = Frame::new(w, HEIGHT, Rgb::hex(0xffffff));
            for (i, s) in snowmen.iter().enumerate() {
                f.blit(&draw_frame(s, t), (i * (WIDTH + GAP)) as i32, 0);
            }
            f
        })
        .collect();
    encode_gif(
        &frames,
        SCALE,
        FRAME_DELAY,
        BufWriter::new(File::create(path)?),
    )?;
    println!("wrote {}", path.display());
    Ok(())
}
