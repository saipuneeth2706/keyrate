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

## Usage

- Press `1`–`4` to pick word count (10/25/50/100), then start typing
- `bs` backspace to correct a character, `Alt+bs` / `Ctrl+bs` to clear a word
- `Esc` to quit
