//! Draws a pixel-art snowman from a GitHub account's public activity.
//!
//! Its languages color it, its contributions build it, its streaks
//! decorate it, and the day that just ended decides how it spends its
//! loop.
//!
//! | Activity | Snowman |
//! | --- | --- |
//! | Top three languages | Scarf and mittens, hat, buttons |
//! | Contributions this year | [`Size`]: from two small snowballs to a tall top-hatted snowman |
//! | Current streak of 7 / 30 days | [`Marks`]: mittens, then string lights |
//! | Any streak of 100 days this year | A gold star above the hat |
//! | Contributions on the day that ended | [`Mood`]: dozing, calm, waving, blizzard or aurora |
//!
//! # Example
//!
//! ```
//! use git_snowman::{Activity, Snowman};
//!
//! # fn main() -> Result<(), git_snowman::Error> {
//! // The response to `git_snowman::QUERY`, from `gh api graphql` or any client.
//! let json = r##"{"data":{"user":{
//!     "login": "octocat",
//!     "contributionsCollection": {"contributionCalendar": {
//!         "totalContributions": 640,
//!         "weeks": [{"contributionDays": [
//!             {"date": "2026-11-01", "contributionCount": 12},
//!             {"date": "2026-11-02", "contributionCount": 0}
//!         ]}]
//!     }},
//!     "repositories": {"nodes": [{"languages": {"edges": [
//!         {"size": 9000, "node": {"name": "Rust", "color": "#dea584"}}
//!     ]}}]}
//! }}}"##;
//!
//! let activity = Activity::from_graphql_json(json)?;
//! let snowman = Snowman::from_activity(&activity);
//! assert_eq!(snowman.to_string(), "a large snowman, blizzard");
//!
//! let mut gif = Vec::new();
//! git_snowman::render_gif(&snowman, 4, &mut gif)?;
//! assert!(gif.starts_with(b"GIF89a"));
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod activity;
mod color;
mod error;
pub mod render;
mod snowman;

pub use activity::{Activity, Day, Language, QUERY};
pub use color::Rgb;
pub use error::Error;
pub use render::{draw_frame, encode_gif, render_frames, render_gif, Frame};
pub use snowman::{seed_for, Colors, Marks, Mood, Size, Snowman};
