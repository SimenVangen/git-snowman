# Getting started

git-snowman can be used three ways: as a GitHub Action that keeps a snowman on your profile README, as a command, or as a Rust library.

## On your profile README

Your profile README lives in the repository named after your username, such as `octocat/octocat`.

1. In that repository, create `.github/workflows/snowman.yml` with the contents of [`examples/profile-workflow.yml`](../examples/profile-workflow.yml).
2. Add the snowman to `README.md`:

   ```markdown
   ![My snowman](assets/snowman.gif)
   ```

3. Open the **Actions** tab, pick **Snowman**, and choose **Run workflow**. After that it runs on its own every night just after midnight UTC.

The workflow needs `contents: write` to commit the GIF, which the example already asks for. The built-in token is enough to read public activity, so there is nothing to set up.

### Action inputs

| Input | Default | What it does |
| --- | --- | --- |
| `login` | the repository's owner | The account to draw. |
| `output` | `snowman.gif` | Where to write the GIF. |
| `scale` | `4` | Screen pixels per art pixel, 1–16. `4` gives 256×256. |
| `mood` | empty | Force a mood (`dozing`, `calm`, `waving`, `blizzard`, `aurora`). Empty uses the real one. |
| `token` | the workflow token | Token for the GitHub API. |

It sets two outputs, `path` and `summary` (such as `a large snowman, waving, with mittens`), and writes the summary to the job summary.

On Linux and Apple Silicon runners it downloads a prebuilt binary from this repository's releases. On other runners, or when run from a branch, it builds from source, which takes about a minute.

## From the command line

Install a binary from the releases page, or build one:

```sh
cargo install git-snowman
```

The command fetches activity with the [GitHub CLI](https://cli.github.com/), so sign in first with `gh auth login`.

```sh
git-snowman                      # draws the account gh is signed in as
git-snowman octocat -o octo.gif  # draws someone else
git-snowman --mood aurora        # previews a mood
git-snowman --save-json me.json  # keeps the API response
git-snowman --json me.json       # draws from a saved response, offline
git-snowman --print-query        # prints the GraphQL query it sends
```

It prints a line describing the snowman and the numbers behind it.

## As a library

```toml
[dependencies]
git-snowman = { version = "0.1", default-features = false }
```

Turning off default features drops the command and its `clap` dependency.

The library doesn't make network requests. Fetch the response to [`QUERY`](https://docs.rs/git-snowman/latest/git_snowman/constant.QUERY.html) however you like, then:

```rust
use git_snowman::{Activity, Snowman};

let activity = Activity::from_graphql_json(&json)?;
let snowman = Snowman::from_activity(&activity);
git_snowman::render_gif(&snowman, 4, std::fs::File::create("snowman.gif")?)?;
```

Every field of `Snowman` is public, so you can also build one by hand. `render_frames` and `draw_frame` give you the raw frames, and `encode_gif` turns any frames into a looping GIF; [`examples/gallery.rs`](../examples/gallery.rs) uses them to lay snowmen out side by side.

## How activity becomes a snowman

See [render-sheets.md](render-sheets.md) for every mood, size and mark.
