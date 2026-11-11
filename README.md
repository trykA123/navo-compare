# Navo Compare

A standalone compare and merge app for folders, files and Git refs. It is built from the same
compare code as [Navo](https://github.com/trykA123/navo), where compare also runs as a module.

Status: planning. The plan is `plans/2026-11-11-standalone.md`.

## Layout

- `crates/navo-compare-app`: the app (the `sidewise` binary).
- `plans/`: plans, one file per piece of work.
- `design/`: prototypes for the app shell.

## Build

```
cargo build -p navo-compare-app
```
