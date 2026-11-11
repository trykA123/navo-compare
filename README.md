# Sidewise

A standalone compare and merge app for folders, files and Git refs. It is built from the same
compare code as [Navo](https://github.com/trykA123/navo), where compare also runs as a module.

Status: in development; there is no release yet. Releases will appear at
[trykA123/sidewise](https://github.com/trykA123/sidewise).

## Layout

- `crates/navo-compare-app`: the app (the `sidewise` binary).
- `crates/navo-compare-cli`: the command-line parser.
- `crates/navo-compare-profile`: the profile, recent and named sessions.
- `assets/brand`: the app icon.
- `scripts/`: the local gate and the Windows build.

## Build

```
cargo build -p navo-compare-app
```

## Licence

MIT or Apache-2.0, at your choice. See `LICENSE-MIT` and `LICENSE-APACHE`.
