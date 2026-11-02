//! Tests through the public API only, against saved GraphQL responses.

use git_snowman::render::{FRAMES, HEIGHT, WIDTH};
use git_snowman::{
    render_frames, render_gif, Activity, Colors, Error, Marks, Mood, Rgb, Size, Snowman,
};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("fixture exists")
}

#[test]
fn busy_account() {
    let a = Activity::from_graphql_json(&fixture("busy.json")).unwrap();
    assert_eq!(a.login, "snow-busy");
    assert_eq!(a.total_contributions, 668);
    assert_eq!(a.days.len(), 365);
    assert_eq!(a.days.last().unwrap().date, "2026-11-02");
    assert_eq!(a.day_that_ended(), 25);
    assert_eq!(a.current_streak(), 142);
    assert_eq!(a.longest_streak(), 142);

    let names: Vec<&str> = a.languages.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(
        names,
        ["Rust", "Nix", "TypeScript", "Python", "Shell", "Dockerfile"]
    );

    let s = Snowman::from_activity(&a);
    assert_eq!(s.size, Size::Large);
    assert_eq!(s.mood, Mood::Aurora);
    assert_eq!(
        s.marks,
        Marks {
            mittens: true,
            lights: true,
            star: true
        }
    );
    // Nix has no GitHub color, so TypeScript and Python move up.
    assert_eq!(
        s.colors,
        Colors {
            scarf: Rgb::hex(0xdea584),
            hat: Rgb::hex(0x3178c6),
            buttons: Rgb::hex(0x3572a5),
        }
    );
    assert_eq!(
        s.to_string(),
        "a large snowman, aurora, with mittens, lights and a star"
    );
}

#[test]
fn quiet_account() {
    let a = Activity::from_graphql_json(&fixture("quiet.json")).unwrap();
    let s = Snowman::from_activity(&a);
    assert_eq!(s.size, Size::Tiny);
    assert_eq!(s.mood, Mood::Dozing);
    assert_eq!(s.marks, Marks::default());
    assert_eq!(s.colors, Colors::DEFAULT);
    assert_eq!(a.longest_streak(), 30);
}

#[test]
fn errors_are_explained() {
    match Activity::from_graphql_json(&fixture("not_found.json")) {
        Err(Error::NoSuchUser(login)) => assert_eq!(login, "no-such-snowman"),
        other => panic!("expected NoSuchUser, got {other:?}"),
    }
    match Activity::from_graphql_json(&fixture("bad_credentials.json")) {
        Err(Error::Api(m)) => assert_eq!(m, "Bad credentials"),
        other => panic!("expected Api, got {other:?}"),
    }
    assert!(matches!(
        Activity::from_graphql_json("not json"),
        Err(Error::Parse(_))
    ));
    assert!(matches!(
        Activity::from_graphql_json("{}"),
        Err(Error::Parse(_))
    ));
}

#[test]
fn the_same_account_draws_the_same_gif() {
    let a = Activity::from_graphql_json(&fixture("busy.json")).unwrap();
    let s = Snowman::from_activity(&a);
    let (mut one, mut two) = (Vec::new(), Vec::new());
    render_gif(&s, 2, &mut one).unwrap();
    render_gif(&s, 2, &mut two).unwrap();
    assert_eq!(one, two);
}

#[test]
fn gif_has_the_promised_shape() {
    let s = Snowman::default();
    let mut gif = Vec::new();
    render_gif(&s, 3, &mut gif).unwrap();

    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut dec = opts.read_info(gif.as_slice()).unwrap();
    assert_eq!(dec.width() as usize, WIDTH * 3);
    assert_eq!(dec.height() as usize, HEIGHT * 3);
    let mut frames = 0;
    while dec.read_next_frame().unwrap().is_some() {
        frames += 1;
    }
    assert_eq!(frames, FRAMES);
}

#[test]
fn moods_animate() {
    for mood in Mood::ALL {
        let s = Snowman {
            mood,
            ..Snowman::default()
        };
        let frames = render_frames(&s);
        let distinct = frames
            .iter()
            .map(|f| &f.pixels)
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert!(distinct > FRAMES / 2, "{mood} barely moves");
    }
}

#[test]
fn language_colors_paint_the_scarf() {
    let red = Snowman {
        colors: Colors {
            scarf: Rgb::hex(0xff0000),
            ..Colors::DEFAULT
        },
        ..Snowman::default()
    };
    let frame = &render_frames(&red)[0];
    assert!(frame.pixels.contains(&Rgb::hex(0xff0000)));
}
