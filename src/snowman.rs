//! The snowman: what an account's activity turns into.

use std::fmt;
use std::str::FromStr;

use crate::{Activity, Error, Rgb};

/// How big the snowman has grown, from contributions over the past year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Size {
    /// Fewer than 50 contributions: two small snowballs, no hat.
    Tiny,
    /// 50 to 199: two snowballs and a beanie.
    Small,
    /// 200 to 499: three snowballs and a top hat.
    Medium,
    /// 500 to 999: taller.
    Large,
    /// 1,000 or more: the tallest snowman.
    Grand,
}

impl Size {
    /// Every size, smallest first.
    pub const ALL: [Size; 5] = [
        Size::Tiny,
        Size::Small,
        Size::Medium,
        Size::Large,
        Size::Grand,
    ];

    /// The size a year's contributions grow.
    pub fn from_contributions(total: u32) -> Self {
        match total {
            0..=49 => Size::Tiny,
            50..=199 => Size::Small,
            200..=499 => Size::Medium,
            500..=999 => Size::Large,
            _ => Size::Grand,
        }
    }

    /// Snowball radii in pixels, bottom ball first.
    pub fn radii(self) -> &'static [i32] {
        match self {
            Size::Tiny => &[5, 4],
            Size::Small => &[7, 5],
            Size::Medium => &[8, 6, 5],
            Size::Large => &[10, 7, 5],
            Size::Grand => &[11, 8, 6],
        }
    }
}

/// How the snowman spends its loop, set by the day that just ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Mood {
    /// No contributions: eyes shut, a few flakes, drifting Zs.
    Dozing,
    /// 1 to 3: light snowfall and the odd blink.
    Calm,
    /// 4 to 9: waves a stick arm.
    Waving,
    /// 10 to 19: a blizzard blows its scarf sideways.
    Blizzard,
    /// 20 or more: the northern lights come out and it hops.
    Aurora,
}

impl Mood {
    /// Every mood, quietest first.
    pub const ALL: [Mood; 5] = [
        Mood::Dozing,
        Mood::Calm,
        Mood::Waving,
        Mood::Blizzard,
        Mood::Aurora,
    ];

    /// The mood a day's contributions set.
    pub fn from_day(count: u32) -> Self {
        match count {
            0 => Mood::Dozing,
            1..=3 => Mood::Calm,
            4..=9 => Mood::Waving,
            10..=19 => Mood::Blizzard,
            _ => Mood::Aurora,
        }
    }

    /// The mood's name in lowercase, as the command line takes it.
    pub fn name(self) -> &'static str {
        match self {
            Mood::Dozing => "dozing",
            Mood::Calm => "calm",
            Mood::Waving => "waving",
            Mood::Blizzard => "blizzard",
            Mood::Aurora => "aurora",
        }
    }
}

impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Mood {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Mood::ALL
            .into_iter()
            .find(|m| m.name().eq_ignore_ascii_case(s.trim()))
            .ok_or_else(|| {
                Error::Parse(format!(
                    "unknown mood {s:?}; expected dozing, calm, waving, blizzard or aurora"
                ))
            })
    }
}

/// What streaks have earned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Marks {
    /// Mittens on the stick hands: a current streak of 7 days or more.
    pub mittens: bool,
    /// Blinking string lights: a current streak of 30 days or more.
    pub lights: bool,
    /// A gold star above the hat: a streak of 100 days or more this year.
    /// Unlike the others, it stays once earned.
    pub star: bool,
}

impl Marks {
    /// The marks for a current streak and the year's longest.
    pub fn from_streaks(current: u32, longest: u32) -> Self {
        Marks {
            mittens: current >= 7,
            lights: current >= 30,
            star: longest.max(current) >= 100,
        }
    }
}

/// Colors taken from the account's languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Colors {
    /// The scarf and mittens: the most-written language.
    pub scarf: Rgb,
    /// The beanie, or the top hat's band: the second.
    pub hat: Rgb,
    /// The buttons: the third.
    pub buttons: Rgb,
}

impl Colors {
    /// Used for any language the account doesn't have.
    pub const DEFAULT: Colors = Colors {
        scarf: Rgb::hex(0xc8323c),
        hat: Rgb::hex(0x2e7d4f),
        buttons: Rgb::hex(0x1c1c24),
    };

    /// Picks colors from languages listed most-written first. Languages
    /// without a GitHub color are skipped.
    pub fn from_languages<'a>(languages: impl IntoIterator<Item = &'a crate::Language>) -> Self {
        let mut picked = languages.into_iter().filter_map(|l| l.color);
        let d = Colors::DEFAULT;
        Colors {
            scarf: picked.next().unwrap_or(d.scarf).against_snow(),
            hat: picked.next().unwrap_or(d.hat).against_snow(),
            buttons: picked.next().unwrap_or(d.buttons).against_snow(),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Colors::DEFAULT
    }
}

/// A snowman, ready to render.
///
/// Build one from an [`Activity`] with [`Snowman::from_activity`], or set
/// the fields yourself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Snowman {
    /// Grown from contributions.
    pub size: Size,
    /// Set by the day that just ended.
    pub mood: Mood,
    /// Earned by streaks.
    pub marks: Marks,
    /// Taken from languages.
    pub colors: Colors,
    /// Places the snowflakes, stars and ground. The same seed draws the
    /// same scene; [`Snowman::from_activity`] derives it from the login.
    pub seed: u64,
}

impl Snowman {
    /// The snowman an account's activity draws.
    pub fn from_activity(activity: &Activity) -> Self {
        Snowman {
            size: Size::from_contributions(activity.total_contributions),
            mood: Mood::from_day(activity.day_that_ended()),
            marks: Marks::from_streaks(activity.current_streak(), activity.longest_streak()),
            colors: Colors::from_languages(&activity.languages),
            seed: seed_for(&activity.login),
        }
    }
}

impl Default for Snowman {
    fn default() -> Self {
        Snowman {
            size: Size::Medium,
            mood: Mood::Calm,
            marks: Marks::default(),
            colors: Colors::DEFAULT,
            seed: 0x5eed,
        }
    }
}

impl fmt::Display for Snowman {
    /// A one-line description, such as
    /// `a large snowman, waving, with mittens and lights`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = format!("{:?}", self.size).to_lowercase();
        write!(f, "a {size} snowman, {}", self.mood)?;
        let mut marks = Vec::new();
        if self.marks.mittens {
            marks.push("mittens");
        }
        if self.marks.lights {
            marks.push("lights");
        }
        if self.marks.star {
            marks.push("a star");
        }
        match marks.as_slice() {
            [] => Ok(()),
            [one] => write!(f, ", with {one}"),
            [init @ .., last] => write!(f, ", with {} and {last}", init.join(", ")),
        }
    }
}

/// FNV-1a over the lowercase login, so the same account always gets the
/// same snowfall.
pub fn seed_for(login: &str) -> u64 {
    login
        .to_lowercase()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds() {
        assert_eq!(Size::from_contributions(0), Size::Tiny);
        assert_eq!(Size::from_contributions(50), Size::Small);
        assert_eq!(Size::from_contributions(999), Size::Large);
        assert_eq!(Size::from_contributions(5000), Size::Grand);
        assert_eq!(Mood::from_day(0), Mood::Dozing);
        assert_eq!(Mood::from_day(4), Mood::Waving);
        assert_eq!(Mood::from_day(20), Mood::Aurora);
        let m = Marks::from_streaks(30, 120);
        assert!(m.mittens && m.lights && m.star);
        assert_eq!(Marks::from_streaks(6, 99), Marks::default());
    }

    #[test]
    fn describes_itself() {
        let mut s = Snowman::default();
        assert_eq!(s.to_string(), "a medium snowman, calm");
        s.marks = Marks::from_streaks(40, 40);
        assert_eq!(
            s.to_string(),
            "a medium snowman, calm, with mittens and lights"
        );
    }

    #[test]
    fn moods_parse() {
        assert_eq!("Aurora".parse::<Mood>().unwrap(), Mood::Aurora);
        assert!("sunny".parse::<Mood>().is_err());
    }
}
