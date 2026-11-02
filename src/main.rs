//! The `git-snowman` command.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::Parser;
use git_snowman::{Activity, Mood, Snowman, QUERY};

/// Draws a pixel-art snowman from a GitHub account's public activity.
///
/// Fetches the activity with the GitHub CLI (`gh`), which must be
/// installed and signed in, or reads a saved response with --json.
#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    /// The GitHub login to draw. Defaults to the account `gh` is signed in as.
    login: Option<String>,

    /// Where to write the GIF.
    #[arg(short, long, default_value = "snowman.gif")]
    output: PathBuf,

    /// How many screen pixels each art pixel becomes. 4 gives a 256×256 GIF.
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u16).range(1..=16))]
    scale: u16,

    /// Read the GraphQL response from this file instead of fetching it.
    #[arg(long, value_name = "FILE", conflicts_with = "login")]
    json: Option<PathBuf>,

    /// Also save the fetched GraphQL response to this file.
    #[arg(long, value_name = "FILE", conflicts_with = "json")]
    save_json: Option<PathBuf>,

    /// Draw this mood instead of the one the day set: dozing, calm,
    /// waving, blizzard or aurora.
    #[arg(long)]
    mood: Option<Mood>,

    /// Print the GraphQL query and exit.
    #[arg(long)]
    print_query: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if args.print_query {
        println!("{QUERY}");
        return ExitCode::SUCCESS;
    }
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("git-snowman: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let json = match &args.json {
        Some(path) => fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?,
        None => {
            let login = match &args.login {
                Some(l) => l.clone(),
                None => gh(&["api", "user", "--jq", ".login"])?.trim().to_string(),
            };
            let json = gh(&[
                "api",
                "graphql",
                "-f",
                &format!("query={QUERY}"),
                "-f",
                &format!("login={login}"),
            ])?;
            if let Some(path) = &args.save_json {
                fs::write(path, &json)?;
            }
            json
        }
    };

    let activity = Activity::from_graphql_json(&json).map_err(|e| match (e, &args.login) {
        (git_snowman::Error::NoSuchUser(_), Some(login)) => {
            git_snowman::Error::NoSuchUser(login.clone())
        }
        (e, _) => e,
    })?;
    let mut snowman = Snowman::from_activity(&activity);
    if let Some(mood) = args.mood {
        snowman.mood = mood;
    }

    if let Some(dir) = args.output.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    let mut out = BufWriter::new(File::create(&args.output)?);
    git_snowman::render_gif(&snowman, args.scale, &mut out)?;
    out.flush()?;

    println!(
        "{}: {} ({} contributions this year, {} on the day that ended, streak {} / longest {})",
        activity.login,
        snowman,
        activity.total_contributions,
        activity.day_that_ended(),
        activity.current_streak(),
        activity.longest_streak(),
    );
    println!("wrote {}", args.output.display());
    Ok(())
}

/// Runs the GitHub CLI and returns what it printed.
fn gh(args: &[&str]) -> Result<String, String> {
    let out = Command::new("gh").args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "the GitHub CLI (gh) is not installed; get it from https://cli.github.com, or pass --json".to_string()
        } else {
            format!("could not run gh: {e}")
        }
    })?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        return Ok(stdout);
    }
    // GraphQL errors come back on stdout as JSON with a failing status;
    // hand them to the parser, which explains them better than gh does.
    if stdout.trim_start().starts_with('{') {
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    Err(format!(
        "gh {} failed: {}",
        args[..2].join(" "),
        stderr.trim()
    ))
}
