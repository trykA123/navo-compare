<p align="center">
  <img src="assets/brand/sidewise.svg" width="112" height="112" alt="Sidewise icon: two upright panes, the right one set lower">
</p>

<h1 align="center">Sidewise</h1>

<p align="center">
  Compare and merge folders, files and Git refs, side by side.<br>
  A fast, quiet desktop app for Windows.
</p>

---

> **Status: in development.** There is no download yet. The first release, 0.1, will appear
> under [Releases](../../releases) on this page.

## What it is

Sidewise shows what changed between two things: two folders, two files, or two points in a
Git history. You pick the two sides, and it lines them up so the differences are easy to see
and easy to act on.

It is built on the compare engine of [Navo](https://github.com/trykA123/navo), where the same
code runs as a module of the file manager. Sidewise is that engine as its own app: it installs
alone, keeps its own settings, and needs nothing else.

## Planned for 0.1

| | |
| --- | --- |
| **Folder compare** | Two trees lined up row by row, with what differs, what exists on one side only, and what is the same. |
| **File diff** | Side by side or unified, with the changed part of a line marked. |
| **Git refs** | Type `repo@branch` or `repo@tag` as a side; nothing is fetched. |
| **Copy changes** | Move one change or a whole file to the other side, with undo. |
| **Two-way merge** | Take changes from either side and save the result. |
| **Tabs and sessions** | Several compares in one window; recent and named sessions come back with one click. |
| **Git difftool** | Use it from the command line and from Git: `sidewise <left> <right>`. |
| **Updates** | The app updates itself from the releases on this page. |

Later: three-way merge with a base.

## What it will not do

- No telemetry. The only network call is the update check.
- No account, no cloud, no AI features.
- It does not follow links out of the folders you compare.

## Install

Nothing to install yet. When 0.1 is out:

1. Download `Sidewise-Setup.exe` from [Releases](../../releases).
2. Run it and pick a folder.
3. Start Sidewise from the Start menu, or right-click two files or folders in Explorer.

Windows 10 and 11, 64-bit. The installer is not code-signed, so Windows SmartScreen may ask
you to confirm the first run.

## About this repository

This repository holds both the source code and the releases of Sidewise. The crates keep their
`navo-compare-*` names; the app and its binary are called Sidewise.

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
