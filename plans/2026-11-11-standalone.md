# Navo Compare · standalone app and module

Status: drafting (Stage 0 ready; Stage 1 briefs wait for V41b–V42 of P42) · Repos: `navo-compare` (this one, the app) and `navo` (the shared compare code) · Branch: `nc-<atom>` per atom · Base: `navo` `origin/main@14a57fd6`; this repo `main` · Started: 2026-11-11

Not in scope: no fork of the compare code, no new compare engine work (P42 keeps Parts V, M and Y), no network or cloud sides, no AI features.

## Goal

The user installs and runs **Navo Compare** (`navo-compare.exe`) without Navo: pick two folders, files or Git refs, see the folder compare, the Stream and the file diff, copy changes with undo, and use it as `git difftool`. The same code stays a module inside Navo (D13), drawn in the panes and the full-window screen as today. Navo Compare keeps its own settings, cache and journal.

Appetite: three stages, each shippable alone. Stage 1 (seams) 3 days of agent work, Stage 2 (standalone 0.1) 4 days, Stage 3 (own crates) 3 days. Scope is cut to fit; a stage that runs over stops and needs a new decision.

## What exists today (checked at 14a57fd6)

| Part | Size | Tied to Navo? |
| --- | --- | --- |
| `navo-compare` (engine) | 29,600 lines | No. Depends on `navo-core` and diff crates only. |
| `navo-git` | — | No. Needs `navo-compare` and `navo-shell` (find Git, kill job). |
| `navo-store` tables `content_hash`, `viewed` | `cache_db/{content_hash,viewed}.rs` | Shared `CacheDb` file with `git_badges`; no compare-only store. |
| `navo-app::compare` (adapters, `PaneCompare`, `DiffBackend`, copy, Stream session) | 15,200 lines | Lightly. Talks to the UI through `DiffHooks` and `CompareHooks`; uses `crate::jobs` (compare job factory) and `crate::settings::stored_value` once each; takes `navo_ui::settings::Values`. |
| `navo-ui::compare_diff` (contract, rows, map, cards, Stream model, sheet) | part of 15,000 | Lightly. Uses `tokens`, `primitives`, `prelude`, `settings`; three small links: `file_list::kind`, `compare_list::tone`, `browser::look_of`. |
| `navo-ui::window_view::compare_screen` (file diff, Stream, large, hex, image, copy, search) | rest of 15,000 | **Yes.** About 25 `impl NavoWindow` blocks. State is already gathered in `NavoWindow.compare` (`CompareView`, `compare_view.rs:26`). Outside it the screen reads `self.browser` (`take_file_diff`, `compare_mode`, `compare_reveal`), `self.values`, and viewer parts (`MenuEntry`, `viewer_menu`, `ViewSimple/Rich/Custom`, `ListUp/Down/Go`, `ByteScroll`), plus `context_menu`, `confirm_dialog`, `file_list::UndoLast`. |
| Folder compare (`browser/compare`, `compare_list`) | 2,100 lines | **Yes.** `CompareMode` lives on `Browser` (`impl Browser` in `browser/compare/{mod,navigate}.rs`); rows draw inside the two panes. `compare_list` (bar, list, row, tone) is pane-free. |
| Commands, keys, labels | `commands/compare.rs`, `commands/keymap.toml`, `compare.ftl` | In Navo's one keymap. Contexts `Compare`, `CompareView`, `CompareStream`. |
| `[compare]` settings | `settings/compare_fields.rs` | In Navo's settings file and sheet. |
| Start-up wiring | `navo-app/src/gui.rs:206,319,327` (`OpenHooks.diff`, `OpenHooks.compare`) | Navo only. No `navo compare` CLI yet (M9 of P42). |

## Decisions

Final. Workers never reopen them; a worker that finds one wrong stops and reports. `D…` ids are global; `NC…` ids are local to this plan.

- **D114 — Navo Compare ships now, as its own exe (owner, 2026-11-11).** Supersedes the 2026-11-10 timing ("after modularity and v1.0"). Own exe, installer and update feed; own settings and data; the same crates stay a Navo module. UX and design choices for the standalone shell are the orchestrator's (owner: "your choice").
- **NC1 — One code base, two hosts.** The compare screens never name `NavoWindow` or `Browser`. They are gpui entities that talk to their window through one trait, `CompareHost`. `NavoWindow` implements it (module mode); the new `CompareWindow` implements it (standalone). Alternative lost: `navo --compare` in the same exe (fast, but the code stays welded to `NavoWindow`, and the owner chose a separate exe).
- **NC2 — Two repositories (owner, 2026-11-11).**
  - `navo-compare` (new, private until the owner opens it): the app. Crate `navo-compare-app` with `[[bin]] navo-compare`, the window shell, start screen, sessions, CLI, installer, release workflow, design files, this plan and the Navo Compare roadmap.
  - `navo` keeps the shared compare code (engine, Git, service, screens) as its module, one source for both apps. This repo depends on those crates by Git, pinned to one `rev` for all of them; a pin bump is its own commit with the gate.
  - Only the seam work lands in `navo` (Stage 1), in small atoms, after V42. Nothing else of this plan is committed there.
  - Alternative lost: move every compare crate here and let Navo depend on this repo. The compare crates need `navo-core`, `navo-fs`, `navo-store` and the UI base from `navo`, so the two repos would depend on each other and Cargo would build two copies of the shared types. Stage 3 reopens it once a base crate exists.
- **NC2b — Order: seams in `navo`, then the exe here, then the crate moves.**
  - Stage 1 cuts the seams inside `navo-ui` and `navo-app`, with no file moved across crates, and makes the compare service a library: `navo-app` has no `lib.rs` today, so `navo-app::compare` and the compare job session cannot be used from another crate. N0 chooses between a `navo-compare-service` crate (preferred) and a lib target on `navo-app`.
  - Stage 2 builds the app in this repo on top of `navo-ui` and the service as Git dependencies. It ships.
  - Stage 3 moves code into its own crates so the exe drops the file manager.
  - Why this order: the seams are needed either way; moving 30,000 lines across crates collides with every open branch; exe size is not a constraint (D44), so shipping does not wait for the moves.
- **NC3 — Target crates (Stage 3).** Dependencies point down.
  - `navo-compare` (engine) and `navo-git`: unchanged.
  - `navo-ui-base` (new): `tokens`, `primitives`, `prelude`, `format`, `context_menu`, `confirm_dialog`, and the viewer parts both apps share (View menu builder, drawers, hex rows). `navo-ui` re-exports them, so other code compiles unchanged.
  - `navo-compare-ui` (new): the contracts (`DiffHooks`, `CompareHooks`, plain UI types), `compare_diff`, `compare_list`, `CompareScreen`, `FolderSession`, the compare commands, key fragment, labels and settings fields. Depends on `navo-ui-base`; never on `navo-ui`.
  - `navo-compare-service` (new): today's `navo-app::compare` plus the compare job factory. Depends on the engine, `navo-git`, `navo-fs`, `navo-store`, `navo-retention` and the contracts.
  - `navo-compare-app` (bin): window shell, start screen, sessions, CLI, single instance, updater, crash reports.
  - Navo: `navo-ui` embeds `navo-compare-ui`; `navo-app` uses `navo-compare-service`.
- **NC4 — `CompareHost` contract.** `options() -> CompareOptions`, `close(reveal: Option<path>)`, `undo_last()`, `open_settings(section)`, `notice(text)`. `CompareOptions` is a plain struct made from the `[compare]` and look settings; it replaces `&settings::Values` in `DiffHooks::open_pair`, `open_stream` and `CompareHooks`, so the service no longer needs Navo's settings type.
- **NC5 — `FolderSession`.** The folder compare state (`CompareMode`, its navigation, filter, cursor and rows stamp) becomes a `Browser`-free model. Navo's panes and the standalone folder view both drive it. Navo's behaviour does not change.
- **NC6 — Standalone folder view: one aligned tree.** One list, left and right columns with a status gutter between them; folders expand in place (→ / ←), so the user sees depth without leaving the row. Built from `compare_list` rows. Navo keeps the two-pane Enter/Backspace model (D10). Reason: the pane model exists to fit Navo's panes; a compare app has no panes to honour, and every current compare tool users know shows a tree.
- **NC7 — Own data (owner: "own").** Standalone profile folder `NavoCompare`, a sibling of Navo's: `settings.toml` (`[compare]`, look, keys), its own `CacheDb` (`content-hash`, `compare-viewed`), its own job journal and held files, `sessions/`, logs. Nothing is shared at run time. First start offers once to copy the `[compare]` section from a Navo profile it finds; it never writes there. Module mode keeps using Navo's files. `--profile <dir>` gives a portable folder.
- **NC8 — Sessions and tabs.** A session is two side specs, the rules, the filter and the view stack (folder → Stream → file diff). Open sessions are tabs in one window. Sessions auto-save to "Recent" (20 kept); Ctrl+S on the start row names one, and named ones stay until deleted. Stored as one TOML file each under `sessions/`.
- **NC9 — One instance, many tabs, `--wait`.** One process per user and profile (the `navo-app::instance` pipe pattern with its own name). A second `navo-compare …` hands its arguments to the running one, which opens a tab. With `--wait` the calling process stays alive until that tab closes and returns its exit code. `--new-window` opens a window instead of a tab.
- **NC10 — Command line.**
  - `navo-compare <left> <right>`: files, folders or `repo@rev` specs, in any mix that makes sense.
  - `--wait`, `--new-window`, `--profile <dir>`, `--left-title <t>`, `--right-title <t>`, `--read-only`.
  - Exit codes: 0 closed or saved, 1 closed with unsaved changes discarded, 2 error. Headless reports (Compare 2) follow `diff(1)`: 0 same, 1 different, 2 error.
  - Three-way (`<base> <ours> <theirs> -o <out>`) arrives with Part M and replaces M9's `navo merge`; `navo compare` in Navo becomes a thin forward to the same parser.
  - A "Use as Git difftool" button shows the exact `git config` lines with Copy, and applies them only after a confirm dialog (it changes the user's Git config).
- **NC11 — Keys and commands.** Command ids and contexts stay as they are. The compare bindings move to one fragment that both keymaps include, so a key is defined once. The standalone adds shell keys only: Ctrl+O choose sides, Ctrl+T new tab, Ctrl+W close tab, Ctrl+Tab next tab, F5 refresh, Ctrl+K palette, Ctrl+, settings, F1 key list. All rebindable (D49).
- **NC12 — Look.** Same theme tokens, fade-only motion and D58 calm rule as Navo. The shell (start screen, tab strip, folder tree, settings) comes from one designer round, NC-U0; the orchestrator picks. It gets its own app icon and window title.
- **NC13 — Release.** Own version line, starting at 0.1.0, and own tags and GitHub releases in this repo. Its `release.yml` builds `navo-compare.exe` and packs it as Velopack id `NavoCompare` with its own Setup and portable zip. Navo's installer does not install it. The Windows VM gates each release. Toolchain and lints match `navo` (Rust 1.99.0, edition 2024).
- **NC14 — Git stays read-only (D3).** No staging, no commits from Navo Compare.
- **NC15 — Boundary guard.** `cargo xtask check` fails when a file under `compare_screen`, `compare_diff` or `compare_list` names `NavoWindow` or `Browser`. The seam cannot rot between stages.

## UX choices for the standalone shell (input to NC-U0)

- **Start screen.** Two wide side fields in the middle, each taking a typed path, a pasted path, a dropped file or folder, or `repo@rev`. A swap button between them. Enter compares. Under them: Recent and Named sessions, filtered as you type. No toolbar.
- **Clipboard compare.** "Compare two texts": two empty editors side by side; paste, and the diff runs live. No files needed.
- **Header instead of toolbars.** One line: the two sides as pills (click to change, swap between them), a breadcrumb for the stack (`folder › Stream › file`), active rules as chips (whitespace, line endings, case) that toggle on click, the filter as a segmented control with counts.
- **Folder tree.** Status colour and a short word per row, size and time on both sides, a check-progress cell. Space previews the pair in a bottom strip (first changes only); Enter opens the diff.
- **File diff.** "Hide unchanged lines" in the View menu, with N lines of context, as the Stream already does. The change map stays.
- **Everything by keys.** Ctrl+K lists every command; F1 shows the keys of the current screen as an overlay.
- **Status line.** Totals, check progress, the memory step (D17), and the last copy with Undo.
- **Explorer.** "Compare with Navo Compare": first pick remembers the left side, the second starts the compare; two selected items compare at once.

## Features to add (roadmap section "Navo Compare")

Order by value for the cost. Items marked P42 are already planned there and are only scheduled here.

| Stage | Item | Notes |
| --- | --- | --- |
| Compare 1 (first standalone) | Standalone exe, start screen, folder tree, tabs, sessions, CLI with `--wait`, difftool set-up, drag and drop, clipboard compare, Explorer menu, own installer and updates | This plan, Stages 1–2 |
| Compare 1 | Hide unchanged lines in the file diff; F1 key overlay | small |
| Compare 2 | Three-way merge with automatic base (P42 Part M); `mergetool` contract | P42 |
| Compare 2 | Moved blocks (P42 V44+); live working-copy compare (C-live); open `.patch`/`.diff` (C-patch) | P42, research picks |
| Compare 2 | Reports: HTML and unified patch export of a file diff, a Stream or a folder compare; headless `--report` | new |
| Compare 2 | Rules beyond whitespace: per-session regex "ignore lines matching" and "replace before compare" (timestamps, generated ids) | new; the engine's `normalise` is the hook |
| Compare 2 | Archives as folders on either side (the `navo-fs` archive provider) | wiring only |
| Compare 2 | "What this branch changed": compare a ref with its merge-base (three-dot), one command | small; `GitSource::merge_base` exists |
| Compare 2 | Folder sync (P42 Part Y) | P42, blocked on D65 |
| Compare 3 | Structured compare for JSON, YAML, XML, TOML: by key, order-insensitive, on `navo-structured` | medium |
| Compare 3 | Table compare for CSV and sheets (C-table): key column, per-cell marks | medium |
| Compare 3 | Syntax-aware text diff (C-syntax, D20 caps) | large |
| Compare 3 | Embedded and automotive files on `navo-automotive`: Intel HEX and S-Record by address, linker map by symbol and section size, A2L and DBC by object | medium each; no competitor has it |
| Compare 3 | File history: step one file through its Git revisions with the diff open | medium |
| Later | Three-way folder compare; folder snapshots (compare today with a saved listing) | on demand |

No-gos: FTP, SFTP and cloud sides; Git writes; AI suggestions; registry and media-tag compare.

## Risks and rollback

- **R1 — Two sessions in the same files.** P42's V41b–V42 edit `compare_screen` and `navo-app::compare::diff` now. Stage 1 atoms start only from a base that contains V42; N0 writes their briefs against that base. Stage 0 touches no shared file.
- **R2 — Behaviour drift in Navo while the seam is cut.** Every Stage 1 atom is a refactor with no behaviour change (tidy apart): the existing `window_view::tests::compare_*` suites must pass unchanged, and the VM tester re-runs the compare pass after N8.
- **R3 — Two journals, one file.** Navo and Navo Compare can both hold old versions of the same file. Each undo restores only its own app's write, and `WriteFile`'s stamp check (D9) already refuses a target changed by the other app.
- **R4 — `--wait` that never returns.** The waiting process exits with 2 when the pipe to the main process closes without an answer; a test covers it.
- **R5 — Installer and update feed for a second app.** Its own Velopack id and its own release feed, so an update of one app never replaces the other. The VM install test runs for it.
- **R6 — The pin drifts.** A change in `navo` can break this repo unseen. The pin moves only in its own commit, with this repo's gate; `navo`'s boundary guard (NC15) and its compare tests keep the contracts stable.
- **Rollback.** Stage 1: revert the squash commit of an atom. Stage 2: the new crate and its release step are additive; removing the release step unships it.

## Components

| Component | Layer | Contract (in → out) | New / change | Test |
| --- | --- | --- | --- | --- |
| `CompareOptions` | ui contract | `&settings::Values → CompareOptions` (plain, `Clone`) | new | `compare_diff/hooks/tests.rs` |
| `DiffHooks`, `CompareHooks` | ui contract | take `&CompareOptions` instead of `&Values` | change | hooks tests; `navo-app` compare tests |
| `FolderSession` | ui (pure) | rows in → cursor, filter, enter/up, next difference, rows stamp | new, from `browser/compare/{mode,navigate}.rs` | `browser::compare` tests, moved |
| `CompareHost` | ui contract | NC4 | new | fake host in tests |
| `CompareScreen` entity | ui | `Entity<CompareScreen>`: file diff, Stream, large, hex, image, copy, sheet, search | change: out of `impl NavoWindow` | `window_view::tests::compare_*` unchanged |
| boundary guard | xtask | NC15 | new | `xtask` test with a bad fixture |
| compare key fragment | ui | one key file included by both keymaps | change | `commands` tests |
| `FolderTree` view | ui | `FolderSession` → one aligned tree list | new (Stage 2) | view tests, gallery |
| `CompareWindow` | app shell | tabs, start screen, header, status line; implements `CompareHost` | new (Stage 2) | window tests |
| sessions store | app | `Session ↔ sessions/<id>.toml`; recent and named lists | new (Stage 2) | store tests |
| standalone profile | app | NC7 folders, settings load, `CacheDb`, journal | new (Stage 2) | start tests |
| CLI and instance | app | NC9, NC10 | new (Stage 2) | CLI tests; VM `--wait` case |
| release packaging | CI | NC13 | change (Stage 2) | VM install test |
| `navo-ui-base`, `navo-compare-ui`, `navo-compare-service` | crates | NC3 | new (Stage 3) | workspace gate; `cargo tree` shows no `navo-ui` under `navo-compare-app` |

## Atoms

Stage 0 runs now. Stage 1 briefs are written by N0 at the base that contains V42, each cut to three files and about 150 changed lines; the rows below fix the order and the outcome only. Stage 0 and Stage 2 run in this repo; Stage 1 runs in `navo`; Stage 3 touches both. Stages 2 and 3 get their own plan files here (`…-shell.md`, `…-crates.md`) when Stage 1 is reviewed.

| Atom | Component | Role | Effort | After | Check | Status | Cost |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Stage 0 — this repo, no shared files** | | | | | | | |
| NC-G0 | repo scaffold: workspace, toolchain, lints, `.gitignore`, README, a `--version` stub (gate script and CI come with Stage 2) | orchestrator | — | — | `cargo build`, `cargo fmt --check`, `cargo clippy`; private repo on GitHub | done | |
| NC-R0 | competitor research | researcher (Haiku) | xhigh | — | report with sources | running | |
| NC-U0 | shell design round: start screen, header, folder tree, tabs, clipboard compare | designer (Sonnet) | medium | NC-R0 | 4 directions behind one picker, both themes, 390 px and desktop | todo | |
| **Stage 1 — seams, in `navo` (after V42 is on main)** | | | | | | | |
| N0 | Stage 1 briefs | architect (Opus) | high | V42 merged | briefs N1–N8 in this file; plan check by `reviewer` | blocked | |
| N1b | compare service as a library (NC2b) | api-builder (Sonnet) | high | N0 | `cargo xtask check -p navo-app` and the new crate | blocked | |
| N1 | `CompareOptions` in both hook traits | api-builder (Sonnet) | medium | N0 | `cargo xtask check -p navo-ui -p navo-app` | blocked | |
| N2 | `FolderSession` out of `Browser` | ui-builder (Sonnet) | high | N0 | `cargo nextest run -p navo-ui browser::compare` | blocked | |
| N3 | `CompareHost` and `NavoWindow`'s implementation | ui-builder (Sonnet) | medium | N1 | `cargo nextest run -p navo-ui window_view::tests::compare_screen` | blocked | |
| N4 | `CompareScreen` entity: host, entry, file body | senior (Opus) | medium | N3 | same | blocked | |
| N5 | entity: Stream, outline, search | ui-builder (Sonnet) | high | N4 | `…compare_stream` | blocked | |
| N6 | entity: large, hex, image | ui-builder (Sonnet) | high | N4 | `…compare_large compare_hex compare_image` | blocked | |
| N7 | entity: copy, picks, sheet, menus, drawers | ui-builder (Sonnet) | high | N4 | `…compare_copy compare_sheet` | blocked | |
| N8 | compare key fragment; boundary guard | patcher (Sonnet) | medium | N5, N6, N7 | `cargo nextest run -p navo-ui commands`; `cargo xtask check` red on the fixture | blocked | |
| NR1 | review Stage 1; VM compare pass | reviewer-senior (Opus) + tester | medium | N8 | verdict; gate; Windows VM CI | blocked | |
| **Stage 2 — standalone 0.1 (own plan)** | | | | | | | |
| S2 | `FolderTree`, `CompareWindow`, start screen, sessions, profile, CLI and instance, clipboard compare, Explorer menu, release packaging | — | — | NR1, NC-U0 | own plan | todo | |
| **Stage 3 — own crates (own plan)** | | | | | | | |
| S3 | `navo-ui-base`, `navo-compare-ui`, `navo-compare-service` | — | — | S2 shipped | own plan | todo | |

## Done when

- Stage 1: no file under `compare_screen`, `compare_diff` or `compare_list` names `NavoWindow` or `Browser` (the guard proves it); Navo's compare tests pass unchanged; the VM compare pass finds no change in behaviour.
- Stage 2: on a Windows VM without Navo, `Navo-Compare-Setup.exe` installs; `navo-compare <dirA> <dirB>` opens the folder tree, the Stream and a file diff; copy one change and Ctrl+Z work; `git difftool -d` and `git difftool` open it and return when the tab closes; it updates itself from the feed.
- Stage 3: `cargo tree -p navo-compare-app` lists no `navo-ui`; Navo still embeds compare; both gates and VM CI pass.

## Log

- 2026-11-11 — Owner: ship the standalone now, as a separate exe, with its own data, also usable as a Navo module; UX and design are the orchestrator's choice (D114). Plan drafted at `origin/main@14a57fd6`. NC-R0 started. Nothing built: V41b–V42 are landing in the same files (R1).
- 2026-11-11 — Owner: a new repository for the app; do not commit this work into `navo` now. Repo `navo-compare` created (private), plan moved here, NC2 rewritten for two repositories, NC13 for an own version line. `navo` is untouched.
