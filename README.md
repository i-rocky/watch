# watch

GNU `watch` for Windows that feels like Linux/macOS.

## Features
- Mirrors GNU watch flags and behavior where possible.
- Alternate screen TUI with live updates, no scrollback spam.
- Diff highlighting (`-d`) and cumulative mode (`--differences=permanent`).
- Screenshot capture (`-s` while running, saves to `--shotsdir`).

## Install
### Scoop
```powershell
scoop bucket add rocky https://github.com/i-rocky/scoop-bucket
scoop install watch
```

### Cargo
```sh
cargo install --git https://github.com/i-rocky/watch.git
```

## Usage
```sh
watch -n 2 "dir"
watch -d "date /t"
watch --differences=permanent "powershell -c Get-Date"
watch -t -w -x "echo hello"
```

Common flags:
- `-n, --interval <secs>`: refresh interval (default 2.0s)
- `-d, --differences[=permanent]`: highlight changes
- `-t, --no-title`: hide header
- `-w, --no-wrap`: truncate long lines (alias: `--no-linewrap`)
- `-x, --exec`: execute without a shell
- `-f, --follow`: append output instead of clearing

Keys while running:
- `q` / `Ctrl+C`: quit
- space: trigger immediate refresh
- `s`: save screenshot (requires `--shotsdir`)

## Development
```sh
cargo test
```

## Release
Tag a version to trigger the GitHub release workflow:
```sh
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds the Windows zip and publishes SHA256 checksums for Scoop.
