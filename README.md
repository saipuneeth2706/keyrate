# keyrate

Practice typing right from your terminal.

## Install & run

```bash
npx keyrate
```

On first run it detects your OS/architecture and downloads the matching binary, then runs it. Supported platforms: Linux (x64/arm64), macOS (x64/arm64), Windows (x64/arm64).

> Requires Node.js >= 16 to fetch the binary.

## Or build from source

```bash
cargo run --release
```

## Getting started

On first launch keyrate asks for your name (max 24 characters), then drops you straight into a typing test. Your name and city appear in the header, e.g. `keyrate · sai · Hyderabad`.

## Modes & options

press `Tab` before starting to switch between modes:

- **Words** — type a fixed number of words. Options: `1` 10 · `2` 25 (default) · `3` 50 · `4` 100.
- **Time** — race the clock for a set duration. Options: `1` 15s · `2` 30s (default) · `3` 60s · `4` 120s. New words are added automatically, so you never run out of text.

The timer starts on your first keystroke.

## Live stats

While typing, the status line tracks:

- **time** — elapsed (words mode) or a color-coded countdown bar (time mode, green → yellow → red as time runs out)
- **wpm** — live words per minute
- **acc** — accuracy percentage
- **err** — error count

## Results

After finishing, a popup shows:

- final **WPM** with a speed badge: `warming up` < 25 · `steady` 25–39 · `smooth` 40–59 · `blazing` 60–79 · `hyper` 80–99 · `spectral` 100+
- **accuracy** percentage
- chips for `wpm`, `acc`, `err`, `time`, `cons` (consistency)
- a braille chart of your WPM over the course of the test

## Scores history

press `Ctrl+s` at any time to open your recent results. The screen shows:

- words / time tabs (toggle with `s` or `Tab`)
- summary of **best wpm**, **avg wpm**, and **avg acc**
- a table of the last 10 tests: `#`, `wpm`, `acc`, `err`, `cons`, `cfg`, `date`

## Keybindings

| Key | Action |
| --- | --- |
| `Esc` | Quit (typing / results / scores) |
| `Tab` | Toggle Words / Time mode (before starting) |
| `1`–`4` | Choose word count or time limit (before starting) |
| `bs` | Delete previous character |
| `Alt+bs` / `Ctrl+bs` / `Ctrl+w` | Delete previous word |
| `Ctrl+s` | Open scores history |
| `s` / `Tab` | Toggle Words / Time tab in scores |
| `r` | Results: new test with new words |
| `q` | Results: new test with the same words |

## Data & config

keyrate stores everything in a platform config directory (`~/.config/keyrate/` on Linux):

| File | Contents |
| --- | --- |
| `keyrate.json` | your name from onboarding |
| `scores_words.json` | words-mode test history |
| `scores_time.json` | time-mode test history |
| `location.json` | city / country / region / coordinates, fetched once in the background at launch (via ipwho.is) and shown in the header |

### Reset

```bash
keyrate --rjson
```

Deletes your name, all scores, and the saved location, returning you to onboarding.

## CLI flags

| Flag | Description |
| --- | --- |
| `-v`, `--version` | Print version |
| `-h`, `--help` | Print help |
| `--rjson` | Reset all saved data |

## Tech

Rust with [ratatui](https://github.com/ratatui-org/ratatui) and crossterm. Ships as a native binary wrapped in a small npm package that downloads the right build at first run. MIT licensed.