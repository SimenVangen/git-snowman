//! Drawing the snowman's scene, frame by frame, and encoding the GIF.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::f32::consts::TAU;
use std::io::Write;

use crate::{Error, Mood, Rgb, Size, Snowman};

/// Scene width in pixels, before scaling.
pub const WIDTH: usize = 64;
/// Scene height in pixels, before scaling.
pub const HEIGHT: usize = 64;
/// Frames in one loop.
pub const FRAMES: usize = 32;
/// Each frame's delay, in hundredths of a second.
pub const FRAME_DELAY: u16 = 10;

const GROUND_Y: i32 = 56;
const CX: i32 = 32;

// The palette that doesn't come from languages.
const SNOW: Rgb = Rgb::hex(0xf4f8ff);
const SNOW_SHADE: Rgb = Rgb::hex(0xc9d6ec);
const SNOW_EDGE: Rgb = Rgb::hex(0x8297bd);
const GROUND: Rgb = Rgb::hex(0xe6edf8);
const GROUND_SHADE: Rgb = Rgb::hex(0xbccbe4);
const COAL: Rgb = Rgb::hex(0x1c1c24);
const CARROT: Rgb = Rgb::hex(0xf28c28);
const CARROT_TIP: Rgb = Rgb::hex(0xc4601c);
const STICK: Rgb = Rgb::hex(0x6b4426);
const HAT: Rgb = Rgb::hex(0x22222c);
const HAT_SHINE: Rgb = Rgb::hex(0x40404e);
const STAR: Rgb = Rgb::hex(0xffd75e);
const STAR_GLOW: Rgb = Rgb::hex(0xb8923a);
const MOON: Rgb = Rgb::hex(0xf3efd2);
const SKY_STAR: Rgb = Rgb::hex(0xdfe6ff);
const SKY_STAR_DIM: Rgb = Rgb::hex(0x6d7aa8);
const WIRE: Rgb = Rgb::hex(0x2c4a35);
const BULBS: [Rgb; 4] = [
    Rgb::hex(0xff4d4d),
    Rgb::hex(0xffd75e),
    Rgb::hex(0x5cdb6a),
    Rgb::hex(0x5cb8ff),
];
const AURORA_CORE: Rgb = Rgb::hex(0x3ee6a0);
const AURORA_FADE: Rgb = Rgb::hex(0x1f7a62);
const AURORA_EDGE: Rgb = Rgb::hex(0x6a4bc4);
const NIGHT: [Rgb; 4] = [
    Rgb::hex(0x0b1026),
    Rgb::hex(0x111a3a),
    Rgb::hex(0x18244e),
    Rgb::hex(0x223063),
];
const STORM: [Rgb; 4] = [
    Rgb::hex(0x2a3046),
    Rgb::hex(0x333a54),
    Rgb::hex(0x3d4562),
    Rgb::hex(0x485172),
];

/// One frame of the scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Pixels, row by row from the top left.
    pub pixels: Vec<Rgb>,
}

impl Frame {
    /// A frame filled with one color.
    pub fn new(width: usize, height: usize, fill: Rgb) -> Self {
        Frame {
            width,
            height,
            pixels: vec![fill; width * height],
        }
    }

    /// The pixel at `(x, y)`, if it is inside the frame.
    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.index(x, y).map(|i| self.pixels[i])
    }

    /// Sets the pixel at `(x, y)`. Points outside the frame are ignored.
    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = c;
        }
    }

    /// Copies `other` into this frame with its top left at `(x, y)`.
    pub fn blit(&mut self, other: &Frame, x: i32, y: i32) {
        for oy in 0..other.height {
            for ox in 0..other.width {
                let c = other.pixels[oy * other.width + ox];
                self.set(x + ox as i32, y + oy as i32, c);
            }
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height)
            .then(|| y as usize * self.width + x as usize)
    }

    fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set(x, y, c);
            }
        }
    }

    fn line(&mut self, (mut x0, mut y0): (i32, i32), (x1, y1): (i32, i32), c: Rgb) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}

/// SplitMix64: small, fast and the same everywhere.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u32) -> i32 {
        (self.next() % n as u64) as i32
    }
}

fn inside(dx: i32, dy: i32, r: i32) -> bool {
    let rf = r as f32 + 0.5;
    ((dx * dx + dy * dy) as f32) <= rf * rf
}

/// Where the snowman's parts sit in a frame.
struct Body {
    /// Snowballs as `(center x, center y, radius)`, bottom first.
    balls: Vec<(i32, i32, i32)>,
}

impl Body {
    fn new(size: Size, lift: i32) -> Self {
        let mut balls = Vec::new();
        let mut prev: Option<(i32, i32)> = None;
        for &r in size.radii() {
            let cy = match prev {
                None => GROUND_Y - r + 1,
                Some((pcy, pr)) => pcy - pr - r + 3,
            };
            balls.push((CX, cy - lift, r));
            prev = Some((cy, r));
        }
        Body { balls }
    }

    fn head(&self) -> (i32, i32, i32) {
        *self.balls.last().expect("a snowman has a head")
    }

    /// The ball under the head: it gets the arms and buttons.
    fn chest(&self) -> (i32, i32, i32) {
        self.balls[self.balls.len() - 2]
    }

    fn bottom(&self) -> (i32, i32, i32) {
        self.balls[0]
    }
}

/// Draws one frame of a snowman's loop. `frame` runs from 0 to
/// [`FRAMES`]` - 1`, and the last frame leads back into the first.
pub fn draw_frame(snowman: &Snowman, frame: usize) -> Frame {
    let t = frame % FRAMES;
    let mood = snowman.mood;
    let mut f = Frame::new(WIDTH, HEIGHT, NIGHT[0]);

    draw_sky(&mut f, snowman, t);
    let flakes = flakes(snowman);
    draw_flakes(&mut f, &flakes, mood, t, false);
    draw_ground(&mut f, snowman.seed);

    let lift = if mood == Mood::Aurora && t % 8 < 3 {
        1
    } else {
        0
    };
    let body = Body::new(snowman.size, lift);
    let (bx, _, br) = body.bottom();
    f.rect(
        bx - br + 2,
        GROUND_Y + 2,
        bx + br + 3,
        GROUND_Y + 2,
        GROUND_SHADE,
    );

    for &(x, y, r) in &body.balls {
        draw_ball(&mut f, x, y, r);
    }
    if snowman.marks.lights {
        draw_lights(&mut f, &body, t);
    }
    draw_buttons(&mut f, snowman, &body);
    draw_arms(&mut f, snowman, &body, t);
    draw_scarf(&mut f, snowman, &body, t);
    draw_face(&mut f, snowman, &body, t);
    let top = draw_hat(&mut f, snowman, &body);
    if snowman.marks.star {
        draw_star(&mut f, top - 3, t);
    }
    if mood == Mood::Dozing {
        draw_zs(&mut f, &body, t);
    }

    draw_flakes(&mut f, &flakes, mood, t, true);
    f
}

/// Draws every frame of a snowman's loop.
pub fn render_frames(snowman: &Snowman) -> Vec<Frame> {
    (0..FRAMES).map(|t| draw_frame(snowman, t)).collect()
}

/// Renders a snowman's loop as a GIF that repeats forever, each pixel
/// scaled up to a `scale` × `scale` block.
pub fn render_gif<W: Write>(snowman: &Snowman, scale: u16, out: W) -> Result<(), Error> {
    encode_gif(&render_frames(snowman), scale, FRAME_DELAY, out)
}

/// Encodes frames as a looping GIF. All frames must be the same size and
/// use 256 colors or fewer between them.
pub fn encode_gif<W: Write>(frames: &[Frame], scale: u16, delay: u16, out: W) -> Result<(), Error> {
    let first = frames
        .first()
        .ok_or_else(|| Error::Encode("there are no frames".into()))?;
    let (w, h) = (first.width, first.height);
    if frames.iter().any(|f| f.width != w || f.height != h) {
        return Err(Error::Encode("frames differ in size".into()));
    }
    let scale = scale.max(1) as usize;
    let (sw, sh) = (w * scale, h * scale);
    let (gw, gh) = (
        u16::try_from(sw).map_err(|_| Error::Encode("the GIF would be too wide".into()))?,
        u16::try_from(sh).map_err(|_| Error::Encode("the GIF would be too tall".into()))?,
    );

    let colors: BTreeSet<Rgb> = frames
        .iter()
        .flat_map(|f| f.pixels.iter().copied())
        .collect();
    if colors.len() > 256 {
        return Err(Error::Encode(format!(
            "the frames use {} colors; a GIF holds 256",
            colors.len()
        )));
    }
    let colors: Vec<Rgb> = colors.into_iter().collect();
    let palette: Vec<u8> = colors.iter().flat_map(|c| [c.0, c.1, c.2]).collect();

    let mut enc = gif::Encoder::new(out, gw, gh, &palette).map_err(enc_err)?;
    enc.set_repeat(gif::Repeat::Infinite).map_err(enc_err)?;

    let mut buf = vec![0u8; sw * sh];
    for frame in frames {
        for y in 0..sh {
            for x in 0..sw {
                let c = frame.pixels[(y / scale) * w + x / scale];
                buf[y * sw + x] = colors.binary_search(&c).expect("color is in palette") as u8;
            }
        }
        let gf = gif::Frame {
            width: gw,
            height: gh,
            delay,
            buffer: Cow::Borrowed(&buf),
            ..gif::Frame::default()
        };
        enc.write_frame(&gf).map_err(enc_err)?;
    }
    Ok(())
}

fn enc_err(e: impl std::fmt::Display) -> Error {
    Error::Encode(e.to_string())
}

fn draw_sky(f: &mut Frame, s: &Snowman, t: usize) {
    let bands = if s.mood == Mood::Blizzard {
        &STORM
    } else {
        &NIGHT
    };
    let band_h = GROUND_Y / 4;
    for y in 0..GROUND_Y {
        let b = ((y / band_h) as usize).min(3);
        for x in 0..WIDTH as i32 {
            // A checker row where two bands meet keeps the gradient soft.
            let edge = b < 3 && y % band_h == band_h - 1 && (x + y) % 2 == 0;
            f.set(x, y, if edge { bands[b + 1] } else { bands[b] });
        }
    }
    if s.mood == Mood::Blizzard {
        return;
    }

    let mut rng = Rng(s.seed ^ 0x57a2);
    for i in 0..12 {
        let (x, y) = (rng.below(WIDTH as u32), rng.below(30));
        let dim = (i * 5 + t) % 16 < 3;
        f.set(x, y, if dim { SKY_STAR_DIM } else { SKY_STAR });
    }

    if s.mood == Mood::Aurora {
        draw_aurora(f, t);
    } else {
        // A crescent moon, top left.
        let (mx, my) = (9, 9);
        for dy in -3..=3 {
            for dx in -3..=3 {
                if inside(dx, dy, 3) && !inside(dx - 2, dy + 1, 3) {
                    f.set(mx + dx, my + dy, MOON);
                }
            }
        }
    }
}

fn draw_aurora(f: &mut Frame, t: usize) {
    let phase = t as f32 / FRAMES as f32 * TAU;
    for x in 0..WIDTH as i32 {
        let xf = x as f32;
        let cy = 11.0 + 3.0 * (xf * 0.11 + phase).sin() + 1.2 * (xf * 0.29 - 2.0 * phase).sin();
        let cy = cy.round() as i32;
        // Rays rise from the bright hem as thin vertical streaks: a slow
        // wave sets how tall they can be, and a faster stripe pattern
        // makes neighbouring columns differ.
        let envelope = 0.5 + 0.5 * (xf * 0.21 + phase).sin();
        let stripe = ((x * 5 + t as i32 / 2) % 4) as f32 / 3.0;
        let ray = (1.0 + 7.0 * envelope * stripe).round() as i32;
        for y in cy - ray..cy {
            let height = cy - y;
            if height <= 2 || (height % 2 == 1 && height < ray) {
                f.set(x, y, AURORA_FADE);
            }
        }
        f.set(x, cy, AURORA_CORE);
        if (x + t as i32) % 3 != 0 {
            f.set(x, cy + 1, AURORA_CORE);
        }
        if x % 2 == 0 {
            f.set(x, cy + 2, AURORA_EDGE);
        }
    }
}

fn draw_ground(f: &mut Frame, seed: u64) {
    f.rect(0, GROUND_Y, WIDTH as i32 - 1, HEIGHT as i32 - 1, GROUND);
    let mut rng = Rng(seed ^ 0x6a0d);
    // Drifts along the horizon.
    let mut x = rng.below(6) - 4;
    while x < WIDTH as i32 {
        let w = 4 + rng.below(8);
        if rng.below(2) == 0 {
            f.rect(x, GROUND_Y - 1, x + w, GROUND_Y - 1, GROUND);
        }
        x += w + 3 + rng.below(6);
    }
    for _ in 0..14 {
        let (x, y) = (
            rng.below(WIDTH as u32),
            GROUND_Y + 2 + rng.below((HEIGHT as i32 - GROUND_Y - 2) as u32),
        );
        f.set(x, y, GROUND_SHADE);
        f.set(x + 1, y, GROUND_SHADE);
    }
}

fn draw_ball(f: &mut Frame, cx: i32, cy: i32, r: i32) {
    for dy in -r - 1..=r + 1 {
        for dx in -r - 1..=r + 1 {
            if !inside(dx, dy, r) {
                continue;
            }
            let edge = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(ex, ey)| !inside(dx + ex, dy + ey, r));
            let c = if edge {
                SNOW_EDGE
            } else if (dx + 2 * dy) as f32 > r as f32 * 0.9 {
                SNOW_SHADE
            } else {
                SNOW
            };
            f.set(cx + dx, cy + dy, c);
        }
    }
}

fn draw_face(f: &mut Frame, s: &Snowman, body: &Body, t: usize) {
    let (hx, hy, r) = body.head();
    let ey = hy - 2;
    let closed = match s.mood {
        Mood::Dozing => true,
        Mood::Calm | Mood::Waving => (20..=21).contains(&t),
        Mood::Blizzard | Mood::Aurora => false,
    };
    for side in [-1, 1] {
        let ex = hx + side * 2;
        if closed {
            f.set(ex, ey, COAL);
            f.set(ex + side, ey, COAL);
        } else {
            f.set(ex, ey, COAL);
            if r >= 5 {
                f.set(ex, ey - 1, COAL);
            }
        }
    }

    let len = if r >= 6 { 3 } else { 2 };
    for i in 0..len {
        f.set(hx + i, hy, if i == len - 1 { CARROT_TIP } else { CARROT });
    }

    let mouth: &[(i32, i32)] = match (s.mood, r >= 5) {
        (Mood::Dozing, _) => &[(0, 2)],
        (_, true) => &[(-2, 2), (-1, 3), (0, 3), (1, 3), (2, 2)],
        (_, false) => &[(-2, 1), (-1, 2), (0, 2), (1, 2), (2, 1)],
    };
    for &(dx, dy) in mouth {
        f.set(hx + dx, hy + dy, COAL);
    }
}

fn draw_scarf(f: &mut Frame, s: &Snowman, body: &Body, t: usize) {
    let (hx, hy, r) = body.head();
    let main = s.colors.scarf;
    let shade = main.scale(0.72);
    let stripe = |x: i32| {
        if (x - hx).rem_euclid(4) < 2 {
            main
        } else {
            shade
        }
    };

    let y0 = hy + r - 1;
    for y in y0..=y0 + 1 {
        let inset = if y == y0 { 0 } else { 1 };
        for x in hx - r + inset..=hx + r - inset {
            f.set(x, y, stripe(x));
        }
    }

    if s.mood == Mood::Blizzard {
        // Blown out to the right, rippling.
        let start = hx + r;
        let len = 4 + r / 2;
        for i in 0..len {
            let wave = if (i as usize + t / 2) % 4 < 2 { 0 } else { 1 };
            f.set(
                start + i,
                y0 + wave,
                if i == len - 1 {
                    shade
                } else {
                    stripe(start + i)
                },
            );
            f.set(start + i, y0 + wave + 1, shade);
        }
    } else {
        // Hanging down the left side.
        let x = hx - r + 2;
        let len = 3 + r / 2;
        for i in 0..len {
            let c = if i % 2 == 0 { main } else { shade };
            f.set(x, y0 + 2 + i, c);
            f.set(x + 1, y0 + 2 + i, c);
        }
        f.set(x, y0 + 2 + len, shade);
        f.set(x + 1, y0 + 3 + len - 1, main);
    }
}

fn draw_arms(f: &mut Frame, s: &Snowman, body: &Body, t: usize) {
    let (bx, by, br) = body.chest();
    let reach = (br - 1).clamp(4, 7);
    let sy = by - br / 3;

    let left_start = (bx - br, sy);
    let left_end = (left_start.0 - reach, sy - reach * 2 / 3);

    let right_start = (bx + br, sy);
    let right_end = if s.mood == Mood::Waving && t % 8 >= 4 {
        (right_start.0 + reach / 2, sy - reach - 1)
    } else if s.mood == Mood::Waving {
        (right_start.0 + reach - 1, sy - reach + 1)
    } else {
        (right_start.0 + reach, sy - reach * 2 / 3)
    };

    for (start, end, side) in [(left_start, left_end, -1), (right_start, right_end, 1)] {
        f.line(start, end, STICK);
        // A twig partway along.
        let mx = start.0 + (end.0 - start.0) / 2;
        let my = start.1 + (end.1 - start.1) / 2;
        f.set(mx, my - 1, STICK);
        if s.marks.mittens {
            let c = s.colors.scarf;
            f.rect(end.0 - 1, end.1 - 1, end.0, end.1, c);
            f.set(
                if side > 0 { end.0 - 2 } else { end.0 + 1 },
                end.1 + 1,
                c.scale(0.72),
            );
        } else {
            f.set(end.0 + side, end.1 - 1, STICK);
            f.set(end.0, end.1 - 1, STICK);
        }
    }
}

fn draw_buttons(f: &mut Frame, s: &Snowman, body: &Body) {
    let (bx, by, br) = body.chest();
    let gap = (br / 2).max(2);
    let count = if body.balls.len() == 2 { 2 } else { 3 };
    for i in 0..count {
        let y = if count == 3 {
            by + (i - 1) * gap
        } else {
            by - gap + i * gap
        };
        f.set(bx, y, s.colors.buttons);
    }
    if body.balls.len() > 2 {
        // Coal down the bottom ball too.
        let (x, y, r) = body.bottom();
        f.set(x, y - r / 3, s.colors.buttons);
    }
}

fn draw_lights(f: &mut Frame, body: &Body, t: usize) {
    let (bx, by, br) = body.bottom();
    let base = by + br / 4;
    let span = br - 1;
    for (i, dx) in (-span..=span).enumerate() {
        let dy = (dx * dx) as f32 / (br as f32 * 2.2);
        let y = base - (span * span) / (br * 2 * 2) + dy.round() as i32;
        if i % 3 == 1 {
            let k = (i / 3 + t / 4) % BULBS.len();
            let lit = (i / 3 + t / 4) % 2 == 0;
            let c = if lit { BULBS[k] } else { BULBS[k].scale(0.55) };
            f.set(bx + dx, y + 1, c);
        } else {
            f.set(bx + dx, y, WIRE);
        }
    }
}

/// Draws the hat and returns the topmost row of the snowman.
fn draw_hat(f: &mut Frame, s: &Snowman, body: &Body) -> i32 {
    let (hx, hy, r) = body.head();
    let head_top = hy - r;
    match s.size {
        Size::Tiny => head_top,
        Size::Small => {
            // A beanie with a pom-pom.
            let c = s.colors.hat;
            let cut = hy - r / 2 + 1;
            for y in head_top..cut {
                for x in hx - r..=hx + r {
                    if inside(x - hx, y - hy, r) {
                        let shade = (x - hx) as f32 > r as f32 * 0.4;
                        f.set(x, y, if shade { c.scale(0.8) } else { c });
                    }
                }
            }
            f.rect(hx - r, cut, hx + r, cut, c.scale(0.72));
            f.rect(hx - 1, head_top - 2, hx, head_top - 1, SNOW);
            f.set(hx, head_top - 1, SNOW_SHADE);
            head_top - 2
        }
        Size::Medium | Size::Large | Size::Grand => {
            let brim = head_top + 1;
            f.rect(hx - r - 1, brim, hx + r + 1, brim, HAT);
            let half = r - 2;
            let h = r;
            f.rect(hx - half, brim - h, hx + half, brim - 1, HAT);
            f.rect(hx - half, brim - 2, hx + half, brim - 1, s.colors.hat);
            f.rect(
                hx - half + 1,
                brim - h + 1,
                hx - half + 1,
                brim - 4,
                HAT_SHINE,
            );
            brim - h
        }
    }
}

fn draw_star(f: &mut Frame, cy: i32, t: usize) {
    let cx = CX;
    let bright = t % 8 < 4;
    f.set(cx, cy, STAR);
    let arm = if bright { 2 } else { 1 };
    for i in 1..=arm {
        let c = if i == 2 { STAR_GLOW } else { STAR };
        f.set(cx + i, cy, c);
        f.set(cx - i, cy, c);
        f.set(cx, cy + i, c);
        f.set(cx, cy - i, c);
    }
    if !bright {
        for (dx, dy) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
            f.set(cx + dx, cy + dy, STAR_GLOW);
        }
    }
}

const Z_GLYPH: [&str; 4] = ["####", "..#.", ".#..", "####"];

fn draw_zs(f: &mut Frame, body: &Body, t: usize) {
    let (hx, hy, r) = body.head();
    for i in 0..3 {
        let phase = (t + i * 11) % FRAMES;
        if phase >= 24 {
            continue;
        }
        let x = hx + r + 1 + phase as i32 / 5;
        let y = hy - r - 2 - phase as i32 / 2;
        for (gy, row) in Z_GLYPH.iter().enumerate() {
            for (gx, ch) in row.chars().enumerate() {
                if ch == '#' {
                    f.set(x + gx as i32, y + gy as i32, SNOW_SHADE);
                }
            }
        }
    }
}

struct Flake {
    x: i32,
    y: i32,
    front: bool,
    sway: f32,
}

fn flakes(s: &Snowman) -> Vec<Flake> {
    let count = match s.mood {
        Mood::Dozing => 10,
        Mood::Calm | Mood::Waving => 22,
        Mood::Blizzard => 80,
        Mood::Aurora => 16,
    };
    let mut rng = Rng(s.seed ^ 0xf1a4e);
    (0..count)
        .map(|_| Flake {
            x: rng.below(WIDTH as u32),
            y: rng.below(HEIGHT as u32),
            front: rng.below(10) < 3,
            sway: rng.below(628) as f32 / 100.0,
        })
        .collect()
}

fn draw_flakes(f: &mut Frame, flakes: &[Flake], mood: Mood, t: usize, front: bool) {
    // Every speed covers a whole number of screens per loop, so the
    // snowfall loops without a jump.
    let step = (HEIGHT / FRAMES) as i32;
    let (vy, vx) = match mood {
        Mood::Blizzard => (2 * step, step),
        _ => (step, 0),
    };
    let phase = t as f32 / FRAMES as f32 * TAU;
    for fl in flakes.iter().filter(|fl| fl.front == front) {
        let sway = if mood == Mood::Blizzard {
            0
        } else {
            (fl.sway + phase * 2.0).sin().round() as i32
        };
        let x = (fl.x + vx * t as i32 + sway).rem_euclid(WIDTH as i32);
        let y = (fl.y + vy * t as i32).rem_euclid(HEIGHT as i32);
        if y < GROUND_Y {
            f.set(x, y, SNOW);
            if mood == Mood::Blizzard && fl.front {
                f.set(x - 1, y - 1, SNOW_SHADE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_combination_fits_and_loops() {
        for size in Size::ALL {
            for mood in Mood::ALL {
                let s = Snowman {
                    size,
                    mood,
                    marks: crate::Marks::from_streaks(200, 200),
                    ..Snowman::default()
                };
                let frames = render_frames(&s);
                assert_eq!(frames.len(), FRAMES);
                let colors: BTreeSet<Rgb> = frames
                    .iter()
                    .flat_map(|f| f.pixels.iter().copied())
                    .collect();
                assert!(colors.len() <= 256);
            }
        }
    }
}
