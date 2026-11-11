# Navo Compare · standalone app and module

Status: running (Stage 0 done; Stage 2 wave 1 ready in `plans/2026-11-11-shell.md`; Stage 1 briefs wait for V41b–V42 of P42) · Repos: `navo-compare` (this one, the app) and `navo` (the shared compare code) · Branch: `nc-<atom>` per atom · Base: `navo` `origin/main@14a57fd6`; this repo `main` · Started: 2026-11-11

Not in scope: no fork of the compare code, no new compare engine work (P42 keeps Parts V, M and Y; the merge screen is designed here, NC37), no network or cloud sides, no AI features.

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
- **NC9 — One instance, many tabs, `--wait`.** One process per user and profile (the `navo-app::instance` pipe pattern with its own name). A second `navo-compare …` hands its arguments to the running one, which opens a tab. With `--wait` the calling process stays alive until that tab closes and returns its exit code. `--new-window` opens a window instead of a tab. A console shell does not wait for a GUI-subsystem exe (Beyond Compare ships `BComp.com` beside `BCompare.exe` for this reason), so the package also holds a small console launcher, `ncompare.exe`, that forwards its arguments with `--wait`. The Windows VM proves both from `cmd`, PowerShell and Git.
- **NC10 — Command line.**
  - `navo-compare <left> <right>`: files, folders or `repo@rev` specs, in any mix that makes sense.
  - `--wait`, `--new-window`, `--profile <dir>`, `--left-title <t>`, `--right-title <t>`, `--read-only`.
  - Exit codes: 0 closed or saved, 1 closed with unsaved changes discarded, 2 error. Headless reports (Compare 2) follow `diff(1)`: 0 same, 1 different, 2 error.
  - Three-way (`<base> <ours> <theirs> -o <out>`) arrives with Part M and replaces M9's `navo merge`; `navo compare` in Navo becomes a thin forward to the same parser.
  - Tool contracts the parser must accept (NC-R0): Git difftool `$LOCAL $REMOTE` (Git ignores the exit code unless `--trust-exit-code`); Git mergetool `$BASE $LOCAL $REMOTE $MERGED` (without `trustExitCode` Git checks whether MERGED changed); TortoiseGit `%base %mine %theirs %merged`, exit 0 marks resolved; Subversion base, theirs, mine, merged with 0 success and 1 failure; Perforce base, theirs, yours, result. Mercurial and Visual Studio are unverified and wait for Part M.
  - A "Use as Git difftool" button shows the exact `git config` lines with Copy, and applies them only after a confirm dialog (it changes the user's Git config).
- **NC11 — Keys and commands.** Command ids and contexts stay as they are. The compare bindings move to one fragment that both keymaps include, so a key is defined once. The standalone adds shell keys only: Ctrl+O choose sides, Ctrl+T new tab, Ctrl+W close tab, Ctrl+Tab next tab, F5 refresh, Ctrl+K palette, Ctrl+, settings, F1 key list. All rebindable (D49).
- **NC12 — Look.** Same theme tokens, fade-only motion and D58 calm rule as Navo. The shell (start screen, tab strip, folder tree, settings) comes from one designer round, NC-U0; the orchestrator picks. It gets its own app icon and window title.
- **NC13 — Release.** Own version line, starting at 0.1.0, and own tags and GitHub releases in this repo. Its `release.yml` builds `navo-compare.exe` and packs it as Velopack id `NavoCompare` with its own Setup and portable zip. Navo's installer does not install it. The Windows VM gates each release. Toolchain and lints match `navo` (Rust 1.99.0, edition 2024).
- **NC14 — Git stays read-only (D3).** No staging, no commits from Navo Compare.
- **NC15 — Boundary guard.** `cargo xtask check` fails when a file under `compare_screen`, `compare_diff` or `compare_list` names `NavoWindow` or `Browser`. The seam cannot rot between stages.
- **NC16 — Every pair opens (owner, 2026-11-11: "any type of file").** A view ladder picks the best view a pair has, and the last step always works:
  1. a kind with its own view: text, image, and later structured (JSON, YAML, XML, TOML), table (CSV, XLSX), embedded (Intel HEX, S-Record, linker map, A2L, DBC), document text (PDF, DOCX);
  2. an archive opens as a folder;
  3. anything else, and any file whose own view fails or passes its cap (D20), opens as Bytes (D18).
  - Alt+V switches between Best view, Text and Bytes on every pair. No pair answers "cannot compare". A kind is detected from content first, then from the name.
  - Each new kind is one module behind one trait (`PairView`: detect, load, rows), so adding a kind never touches the screens.
- **NC17 — Speed is a release gate (owner: "extremely fast").** The budgets below are targets. A bench job measures each on every release build in the Windows VM and fails the release when one is missed by more than 10 %; the numbers go in the release notes.
  - Window on screen under 300 ms; first folder rows under 100 ms; 200,000 files by size and time under 2 s; the same folders again under 1 s (hash cache); file diff up to 1 MB under 50 ms; two 1 GB logs aligned on the first screen under 1 s (D16); two 1 GB binaries in hex under 200 ms (D18); scrolling at 60 frames a second on any file.
  - Rules that keep it true: nothing loads before the first frame; all reading, hashing and diffing on workers (D24); rows built for the viewport only; a step that runs long falls back and says so (D20); the content check reads visible rows first and never blocks the tree (D19).
  - The folder numbers are new and unmeasured; the first bench run may move a target, by a Log line, before 0.1 ships.
- **NC18 — Selecting folders (owner, 2026-11-11).** Either side of a compare can be a folder, and a folder is always compared with everything inside it, at any depth.
  - In the tree, Space ticks rows and Shift+↓ ticks a range; a ticked folder takes its whole subtree. "Compare only these" narrows the compare to the ticked rows; Stream, Copy and Ignore act on them too.
  - Ctrl+L and Ctrl+R mark any folder or file as the left or right side, also two that do not share a name or a parent; "Compare these two" opens them as a new tab.
  - The start screen and the Explorer menu take folders the same way as files; several dropped folders fill left, then right.
- **NC19 — Shell direction: A, with two parts of B (orchestrator; the owner delegated the pick, 2026-11-11).** The app keeps Navo's current look: same tokens, same font, red and green for removed and added.
  - From A: tabs on top; the start screen with two side fields and Recent and Named under them; the aligned tree with the status in the middle; the one-line header.
  - From B: the preview strip under the tree (Space shows it; hidden until asked, D58) and the list of changed files in the file diff, as the left drawer (Ctrl+B).
  - From the research: change counts per row and the "Hide unchanged" header chip.
  - The shared screens (Selection, Every kind of file, Speed, Two texts) are built as the prototype shows them.
  - Lost: B's fixed sidebar (it takes about 215 px from a side-by-side diff, and NC8 already chose tabs); C and D (another font and hue; D changes what the colours mean, so the module inside Navo and the app would look different).
- **NC20 — Threat model.** Trusted: the user, the installed files, the profile folder. Untrusted: every file, folder and archive that is compared; every Git repository; command-line arguments and instance messages; the update feed until its package check passes. Out of scope: an attacker who already runs code as the same user, or as an administrator. Every rule in NC21–NC26 names a test, and Stage 2 is not done without them.
- **NC21 — Git runs only through `navo-git`.** At the base, `navo-git/src/trust.rs` turns fsmonitor off, neutralises filters, external diff and textconv programs, and reports a folder Git does not trust. The app never starts `git` any other way.
  - A repository that fails the trust check is compared as plain folders, with the reason in the header.
  - A `repo@rev` rev is refused when it is empty, starts with `-`, holds a control character or is longer than 256 bytes.
  - "Use as Git difftool" writes only the global config, after the confirm dialog (NC10).
  - Test: a fixture repository whose config names a program that writes a marker file; after a full compare the marker does not exist.
- **NC22 — Limits for every view (extends NC16 and D20).** Each `PairView` declares caps: input bytes, decoded bytes, nesting depth, entry count and time. A view never allocates from a size written in the file before it checks the cap.
  - A pair that passes a cap steps down the ladder to Bytes with a notice; it never aborts the app.
  - Archive entries are read in memory or as streams, never extracted to disk. An entry name with `..`, an absolute path or a drive letter is listed as text and never used as a path.
  - Each parser gets a fuzz target before its kind ships: a 60 s run in the gate, a longer run before a release.
- **NC23 — Instance channel.** The app uses Navo's instance transport unchanged. At the base it already limits the pipe to the current user (`instance/windows_security.rs`), rejects remote clients and takes the first-instance flag (`instance/windows_pipe.rs`). This repo forbids `unsafe`, so the transport comes from `navo` as a crate (atom N9), not as a copy.
  - A message carries at most 256 arguments and 64 KiB; a larger one is refused with exit code 2.
  - Arguments from the channel go through the same parser as the command line. No argument can run a program, write a file or change a setting; a message can only open tabs.
- **NC24 — Updates and signing.** Facts: this repository is private, so an installed app cannot read its releases without a secret; Navo's `release.yml` does not sign the exe.
  - No token or key is ever built into the app.
  - Pending, owner decides before the release atom runs: (1) open the repository, (2) publish releases to a public release-only repository, or (3) no self-update while private. Without an answer, 0.1 ships as (3): the installer and a download link, no update check.
  - Authenticode signing is pending with the same deadline (a certificate costs money). Until then the release notes carry the SHA-256 of every file, and the README says that SmartScreen warns.
- **NC25 — Links.** A folder compare does not follow symlinks, junctions or other reparse points. A link is one row of kind Link, compared by its target text.
  - A per-session rule "Follow links" follows them, with a set of visited file identities that stops loops.
  - Copy never writes through a link: the resolved destination must stay under the side's root, else the copy is refused with the reason.
  - Tests: a junction that points to its own parent; a junction that points outside the root.
- **NC26 — Private data.** No telemetry. The only network call the app makes is the update check (NC24); Git is never asked to fetch.
  - Crash reports stay in the profile's `crashes/` folder; sending one is a manual step.
  - Logs hold paths, never file content; five files of 2 MiB.
  - "Two texts" never reaches the disk: not in sessions, not in logs, not in crash reports.
  - "Remember recent compares" is a setting, with "Clear now". The journal's held files follow the `navo-retention` limits and show their size in Settings, with "Clear now".
- **NC27 — Format versions.** Every file the app writes carries a `version`: `settings.toml`, `profile.toml`, each `sessions/*.toml`.
  - Same version: load. Older: migrate in memory, write on the next save, keep one `*.bak-v<N>`. Newer: load what is known, never write the file, show a notice.
  - `CacheDb` keeps its own migrations (`cache_db/migrations.rs`). A newer cache is renamed aside and rebuilt. A newer journal turns Undo off and keeps the held files.
  - Each file has a downgrade test.
- **NC28 — Memory budgets, gated with NC17.** Targets, unmeasured: the empty window under 150 MB; a 200,000-file folder compare under 400 MB; each extra idle tab under 20 MB; a 1 GB pair inside the D17 step. A tab in the background for 60 s drops its row layouts, decoded images and view caches and keeps its model. The first bench run may move a target, by a Log line.
- **NC29 — One scheduler for all tabs.** One worker pool per process (D24). Order: the visible tab's viewport, then the rest of the visible tab, then background tabs.
  - Closing a tab or changing a side cancels its jobs; a job checks for cancel at every file and every 1 MiB.
  - One reader at a time on a rotational or network volume, several on an SSD; a setting overrides the guess.
  - A network read that times out makes the row "Unavailable"; F5 tries again.
- **NC30 — Hash cache rules.** At the base the key is provider, path, size, modified time and file identity (`ContentKey`), with a 16 MiB budget. Added: a file modified less than 2 s before it was hashed is not cached (the racy-timestamp rule Git uses); a path with no file identity can be forced to re-read with the session rule "Always re-read".
- **NC31 — Results that went stale.** Every compared pair keeps the stamp of both sides (size, modified time, file identity). Before a diff opens, a copy runs or a save writes, the stamps are checked again.
  - A mismatch marks the row "Changed on disk" and compares that row again; a copy is refused (D9).
  - Compare 1 re-checks the visible rows on F5 and when the window gets focus. Live watching is Compare 2 (C-live). A refresh never moves the cursor.
- **NC32 — Pin canary and lock drift.** A manual CI job builds and tests this repo against the head of `navo` main and only reports. `cargo xtask lock-drift` fails when a crate that both lockfiles hold has two versions. `[patch.crates-io] gpui-pre-windows` is repeated here at the pinned rev, because a patch does not pass through a dependency.
- **NC33 — Bench method (refines NC17).** Data comes from a seeded generator, `cargo xtask bench-data`: set S (2,000 files), M (200,000 files), L (two 1 GiB logs, two 1 GiB binaries). Runs happen in the Windows VM with the EDR profile off: five runs, the median, cold and warm cache named apart. A missed budget runs once more; two misses fail the release.
- **NC34 — Saving.** Every write goes through `navo-fs`: a temp file in the same folder, an atomic replace, the stamp check (D9), the old version in the journal. A save keeps the file's encoding, BOM and line endings, also mixed ones. A read-only file, or `--read-only`, has no write commands. Test: a UTF-16 LE file with BOM and CRLF survives an edit and a save byte for byte outside the edit.
- **NC35 — Accessibility.** Every action has a key; focus is always visible; a status is a word and a shape, never colour alone; each text and background pair passes the contrast tool; the app follows the Windows text scale and has a high-contrast theme; reduced motion removes the fades. Screen-reader support follows Navo's GPUI work: it gates 1.0, not 0.1.
- **NC36 — Platform and uninstall.** Windows 10 22H2 and later, x64, for every 0.x release; Linux builds are host checks only. Everything the app registers is per user, and the installer never needs an administrator. Uninstall removes the exe and the Explorer menu keys, offers to remove the profile, and removes Git config lines only when they still name this exe, after showing them.
- **NC37 — Merge is in scope, and simple (owner, 2026-11-11: "merge like Beyond Compare, but easy to understand").** Merge moves from Compare 2 to the second release, 0.2; 0.1 ships the two-way part.
  - Two-way (0.1): each change has "Use left" and "Use right" on the change itself; "Take all from left" and "Take all from right" in the header; Undo on every step. No toolbar of arrows.
  - Three-way (0.2): the engine and session are P42 Part M (D13: own diff3, the replay oracle). The screen is designed here.
  - Rules for the screen: one result pane, the only one that is edited; changes that only one side made are taken alone and stay hidden until asked; each conflict is one card with four actions (Use mine, Use theirs, Use both, Edit) and the sides named by branch or file, not "left" and "right"; one counter "N conflicts left"; Save stays off until it is zero; keys 1, 2, 3 choose and F8 goes to the next conflict; a file with conflict markers opens straight in this screen.
  - A design round (NC-U1) gives four directions for this screen; the owner picks.
  - Three-way arguments (NC10) stay refused with a clear message until 0.2.
- **NC38 — Corrections after the senior review (2026-11-11).** They amend the entries named; where they differ, this entry wins.
  - **NC21.** At the pin the guard covers only background status calls. `git cat-file --filters` (`navo-git/src/cli/filters.rs`) runs a repository's own smudge filter, `git diff --raw` (`cli/changes.rs`) runs its fsmonitor program, and external diff and textconv are detected, not neutralised. NC21 holds only after atom N10, which passes the guard to every Git call and allows only the standard git-lfs filter. The marker test belongs to N10.
  - **NC9 and NC23.** Navo's transport sends one typed request, expects an answer in 2 s and serves one client at a time; it cannot carry arguments, wait, or return an exit code. Atom N9 therefore changes it: the payload is an argument list and a cwd; a wait reply stays open and returns the exit code; several clients are served at once. `ncompare.exe` starts the app with null standard handles, so Git's pipe closes when the launcher exits. The pipe limits that exist stay.
  - **NC10 exit codes.** For a three-way merge, 0 means saved with no conflict left; everything else is 1. This matches P42 M9 and keeps TortoiseGit and Git from marking an unsaved file as resolved.
  - **NC37.** NC-U1 replaces P42's M0, and P42's M5 choice set becomes NC37's four actions plus the bulk whitespace resolve; a Log line goes into P42 when Stage 1 starts. Edit has key 4. In 0.1 a file with conflict markers opens in the two-way view with a notice. The feature row "Compare 2 · Merge screen" is void.
  - **NC17, NC28, NC33.** The gate uses warm runs only; a cold run is taken after a VM restart and is reported, not gated. "Window on screen" is the first presented frame after process start; "first folder rows" is measured on set S; memory is peak private bytes; frames per second are reported, not gated (the VM has no GPU). The gate runs with the antivirus profile off; one run with it on is reported.
  - **NC25, NC31, NC34.** The link and stamp checks run on the open handle (final path) right before the rename, not on the path (atom N12 in `navo-fs`). A replace keeps the target's ACL and alternate streams, and the app refreshes its own stamps after a save so its own write is not "Changed on disk". A tab opened for a tool (`--wait`) is never saved to Recent and its writes skip the journal's held files once the tab closes.
  - **Start-up.** The app restricts its DLL search to its own folder and System32 before it loads anything, as `navo-setup` does. `[profile.release]` and `[profile.dev]` are repeated here from `navo`, because profiles do not pass through a dependency.
  - **Explorer menu.** A static menu entry starts one process per selected item. The mechanism (pair two launches that arrive close together, or a shell extension from a `navo` crate) is decided by the orchestrator before atom S1.
- **NC39 — Updates and signing ship in 0.1 (owner, 2026-11-11).** Supersedes the default of NC24; the rule "no token in the app" stays.
  - The update feed must be readable without a secret. Pending, owner: a public release-only repository (the orchestrator's proposal; the source stays private) or opening this repository. Atom NC-R1 checks that Velopack supports it.
  - Every exe and the installer are Authenticode-signed in the release workflow. The certificate is bought by the owner; NC-R1 lists what an individual in Romania can buy and use from CI.
- **NC43 — No code signing (owner, 2026-11-11: "no certificates").** Supersedes the signing bullet of NC39; updates stay in 0.1. The exe and the installer ship unsigned, as Navo's do. The release notes carry the SHA-256 of every file, and the README says that Windows SmartScreen warns on the first run. Signing comes back only by a new owner decision.
- **NC40 — Installer: Navo's wizard, with another brand (owner, 2026-11-11).** `navo-setup` at the pin is a wizard (welcome, licence, folder, shortcuts, finish) that runs Velopack's Setup silently into the chosen folder, with payload checks and an upgrade path. Atom N11 makes its name, icon, pack id and default folder parameters; this repo builds its Setup from that crate. No copy of the wizard code.
- **NC41 — The product name is open (owner, 2026-11-11).** Every name the user sees (window title, exe, profile folder, pack id, pipe name, menu text) comes from one `brand` module. Crate and repository names are internal and stay. Until the owner picks, the brand is "Navo Compare". NC-R1 checks the candidates for collisions.
- **NC42 — Icons (owner, 2026-11-11).** The app icon comes from a design round, NC-U2, after the name is picked: four directions, an SVG master, an `.ico` with 16–256 px, in the family of `navo`'s `assets/brand`. Icons inside the app come from `navo-ui`'s set.

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
| Compare 1 | Hide unchanged lines in the file diff (also a header chip); F1 key overlay; change counts per row and in the status line | small |
| Compare 1 | Tick rows and "Compare only these"; mark any two folders or files and compare them (NC18) | medium |
| Compare 1 | View ladder with Bytes as the floor (NC16); speed bench gate (NC17) | medium |
| Compare 2 | Document text compare for PDF and DOCX (on `navo-pdf`) | medium |
| Compare 1 (0.1) | Two-way merge: Use left, Use right on each change; take all from one side; Undo (NC37) | small; the copy and pick code exists |
| Compare 1 (0.2) | Three-way merge with automatic base (P42 Part M); `mergetool` contract; the simple merge screen (NC37) | P42 engine; screen designed here |
| Compare 2 | Moved blocks (P42 V44+); live working-copy compare (C-live); open `.patch`/`.diff` (C-patch) | P42, research picks |
| Compare 2 | Reports: HTML and unified patch export of a file diff, a Stream or a folder compare; headless `--report` | new |
| Compare 2 | Rules beyond whitespace: per-session regex "ignore lines matching" and "replace before compare" (timestamps, generated ids) | new; the engine's `normalise` is the hook |
| Compare 2 | Unimportant differences: a rule match shows in a quieter tone and one toggle hides it (Beyond Compare's model); a line filter bar in the file diff shows only lines that match | small on top of the rules |
| Compare 2 | Archives as folders on either side (the `navo-fs` archive provider) | wiring only |
| Compare 2 | "What this branch changed": compare a ref with its merge-base (three-dot), one command | small; `GitSource::merge_base` exists |
| Compare 2 | Folder sync (P42 Part Y) | P42, blocked on D65 |
| Compare 3 | Structured compare for JSON, YAML, XML, TOML: by key, order-insensitive, on `navo-structured` | medium |
| Compare 3 | Table compare for CSV and sheets (C-table): key column, per-cell marks | medium |
| Compare 3 | Syntax-aware text diff (C-syntax, D20 caps) | large |
| Compare 3 | Embedded and automotive files on `navo-automotive`: Intel HEX and S-Record by address, linker map by symbol and section size, A2L and DBC by object | medium each; no competitor has it |
| Compare 3 | File history: step one file through its Git revisions with the diff open | medium |
| Later | Three-way folder compare; folder snapshots (compare today with a saved listing); owner and permission compare; a file shelf for more than two files | on demand |

No-gos: FTP, SFTP and cloud sides; Git writes; AI suggestions; registry and media-tag compare.

## Research (NC-R0, 2026-11-11)

Partial: the researcher's budget ran out. Checked from vendor pages and changelogs: Beyond Compare 5.2.8, WinMerge 2.16.60, Araxis 2026.1, Kaleidoscope 7, Sublime Merge, Meld 3.24.1. Not checked: start screens and session naming of each tool, VS Code, JetBrains, Zed, difftastic, delta, user surveys. NC-U0's researcher step covers the screens.

- **Where the others hurt.** Slow folder compares on USB and network drives and disk thrash (Beyond Compare forum); DPI and dark-theme defects fixed release after release (Beyond Compare 5.1–5.2, WinMerge 2.16.58); long-path and Unicode crashes; Meld has no portable Windows build; Kaleidoscope is macOS-only and subscription-only. Navo Compare's answers exist already: one background content pass with a hash cache (D19), theme tokens only, long paths in `navo-fs`.
- **Instances and waiting.** Beyond Compare, WinMerge, Araxis, Kaleidoscope and Sublime Merge all run one instance with a waiting console command beside it. NC9 follows them.
- **Exit codes.** WinMerge and `diff(1)` use 0 same, 1 different, 2 error; NC10 keeps that for headless runs.
- **Prices today.** Beyond Compare $35 Standard, $70 Pro, one-time. Araxis $129 and $269, one-time with paid renewals. Sublime Merge $99 one-time. Kaleidoscope from $8 a month. Meld and WinMerge are free. Pricing of Navo Compare is the owner's decision, not part of this plan.
- **Left out on purpose.** A script language (Beyond Compare `@script`), plugin pipelines, SFTP sides, media tags, hunk staging and an AI conflict helper stay no-gos or later; headless `--report` with exit codes covers the scripted use.

## Risks and rollback

- **R1 — Two sessions in the same files.** P42's V41b–V42 edit `compare_screen` and `navo-app::compare::diff` now. Stage 1 atoms start only from a base that contains V42; N0 writes their briefs against that base. Stage 0 touches no shared file.
- **R2 — Behaviour drift in Navo while the seam is cut.** Every Stage 1 atom is a refactor with no behaviour change (tidy apart): the existing `window_view::tests::compare_*` suites must pass unchanged, and the VM tester re-runs the compare pass after N8.
- **R3 — Two journals, one file.** Navo and Navo Compare can both hold old versions of the same file. Each undo restores only its own app's write, and `WriteFile`'s stamp check (D9) already refuses a target changed by the other app.
- **R4 — `--wait` that never returns.** The waiting process exits with 2 when the pipe to the main process closes without an answer; a test covers it.
- **R5 — Installer and update feed for a second app.** Its own Velopack id and its own release feed, so an update of one app never replaces the other. The VM install test runs for it.
- **R6 — The pin drifts.** A change in `navo` can break this repo unseen. The pin moves only in its own commit, with this repo's gate; `navo`'s boundary guard (NC15) and its compare tests keep the contracts stable.
- **R7 — A hostile file exhausts memory or crashes the app.** NC22: caps, the step down to Bytes, fuzz targets.
- **R8 — A hostile repository starts a program.** NC21: one trusted path to Git, with the marker test.
- **R9 — A secret in the app, or an update nobody can check.** NC24: no built-in token; hashes in the release notes until signing exists.
- **R10 — A copy writes outside the compared folder through a link.** NC25.
- **R11 — An older app opens a newer profile and loses data.** NC27: never write a newer file.
- **R12 — Merge grows into a second Beyond Compare.** NC37 fixes the rules of the screen; anything beyond them is a new decision.
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
| NC-R0 | competitor research | researcher (Haiku) | xhigh | — | report with sources | done, partial (see Research) | |
| NC-U0 | shell design round: start screen, header, folder tree, tabs, clipboard compare; its researcher step gathers the start screens the first research missed | designer (Sonnet) | medium | NC-R0 | 4 directions behind one picker, both themes, 390 px and desktop | done; NC19 picks A with two parts of B | |
| NC-R1 | research: Velopack feed with a private source, name collisions (its signing part is void, NC43) | researcher (Haiku) | xhigh | — | report with sources | running | |
| NC-U2 | app icon round (NC42) | designer (Sonnet) | medium | owner's name pick | 4 directions, SVG and `.ico` | blocked | |
| NC-U1 | merge screen design round (NC37): two-way actions and the three-way screen | designer (Sonnet) | medium | NC19 | 4 directions behind one picker, both themes; owner picks | todo | |
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
| N9 | `navo-instance` crate from `navo-app/src/instance`: argument-list payload, wait reply with exit code, several clients at once, a second app name (NC38) | senior | medium | N0 | `cargo xtask check -p navo-app` and the new crate; VM instance tests | blocked | |
| N10 | Git guard on every call: filtered reads and `read_changes`; only the git-lfs filter allowed; the marker test (NC38) | senior | medium | N0 | `cargo nextest run -p navo-git trust`; marker fixture | blocked | |
| N11 | `navo-setup` takes its brand as parameters (NC40) | api-builder | high | N0, Navo's installer atoms W15b–W15d | `cargo xtask check -p navo-setup`; VM install of both brands | blocked | |
| N12 | handle-based link and stamp check before the rename; replace keeps ACL and streams (NC38) | senior | medium | N0 | `cargo nextest run -p navo-fs local`; VM junction-swap test | blocked | |
| NR1 | review Stage 1; VM compare pass | reviewer-senior (Opus) + tester | medium | N8 | verdict; gate; Windows VM CI | blocked | |
| **Stage 2 — standalone 0.1 (own plan)** | | | | | | | |
| S2 | `FolderTree`, `CompareWindow`, start screen, sessions, profile, CLI and instance, clipboard compare, Explorer menu, release packaging | — | — | wave 1: none; the rest: NR1 | `plans/2026-11-11-shell.md` | wave 1 ready | |
| S2m | three-way merge screen (NC37), release 0.2 | — | — | NC-U1, P42 M1–M5 | own plan (`…-merge.md`) | todo | |
| **Stage 3 — own crates (own plan)** | | | | | | | |
| S3 | `navo-ui-base`, `navo-compare-ui`, `navo-compare-service` | — | — | S2 shipped | own plan | todo | |

## Done when

- Stage 1: no file under `compare_screen`, `compare_diff` or `compare_list` names `NavoWindow` or `Browser` (the guard proves it); Navo's compare tests pass unchanged; the VM compare pass finds no change in behaviour.
- Stage 2: on a Windows VM without Navo, `Navo-Compare-Setup.exe` installs; `navo-compare <dirA> <dirB>` opens the folder tree, the Stream and a file diff; copy one change and Ctrl+Z work; `git difftool -d` and `git difftool` open it and return when the tab closes; it updates itself from the feed when NC24 allows one.
- Stage 2, safety: the hostile-repository fixture leaves no marker (NC21); a junction loop ends and a junction that leaves the root is never written through (NC25); a newer session or settings file is never overwritten (NC27); with the network off the app starts and compares with no error and no wait (NC26); the bench prints time and memory for sets S, M and L (NC17, NC28, NC33).
- Stage 3: `cargo tree -p navo-compare-app` lists no `navo-ui`; Navo still embeds compare; both gates and VM CI pass.

## Log

- 2026-11-11 — Owner: ship the standalone now, as a separate exe, with its own data, also usable as a Navo module; UX and design are the orchestrator's choice (D114). Plan drafted at `origin/main@14a57fd6`. NC-R0 started. Nothing built: V41b–V42 are landing in the same files (R1).
- 2026-11-11 — Owner: a new repository for the app; do not commit this work into `navo` now. Repo `navo-compare` created (private), plan moved here, NC2 rewritten for two repositories, NC13 for an own version line. `navo` is untouched.
- 2026-11-11 — NC-R0 back, partial (tool budget ran out; UX of start screens and editors' diff views unchecked). Added the Research section, the console launcher to NC9, tool contracts to NC10, and three feature rows.
- 2026-11-11 — Owner: more ideas in the design; compare must be extremely fast and work on any kind of file; folders can be selected and compared with their contents. Added NC16 (view ladder), NC17 (speed gate, targets unmeasured), NC18 (selection and marks). Prototype `design/prototypes/shell-directions.html` gained the screens Selection, Every kind of file, Speed and Two texts. The visual research (NC-U0) came back; its two new ideas (change counts, a Hide unchanged chip) are in the prototype. The owner has not picked a direction yet.
- 2026-11-11 — Owner delegated the direction pick: NC19 (A, with the preview strip and the changed-files drawer from B). Owner asked for the architecture, system-design and security gaps in the plan: NC20–NC36, R7–R11, the Stage 2 safety lines in Done when, atom N9. Checked at the base before writing: Git trust handling, the pipe limits and the cache key and migrations already exist in `navo`, so NC21, NC23, NC27 and NC30 reuse them and add only what is missing. Owner asked for merge, simple to read: NC37, R12, NC-U1, S2m. Prototype checked at 1440 px and 390 px in both themes; the picker now wraps at phone width. Stage 2 plan written: `plans/2026-11-11-shell.md`. No agent started: the owner asked to be told first.
- 2026-11-11 — Owner said yes to wave 1 and asked for an Opus review first. Senior review (reviewer-senior, Opus medium): FIX FIRST, eleven findings, all taken: NC38, atoms N9 (rewritten), N10, N12, and the wave 1 and wave order fixes in the shell plan. Two claims of mine were wrong at the pin (Git guard coverage, the instance transport); NC38 corrects them. Owner then asked for auto-update and signing in 0.1 (NC39), Navo's installer as the model (NC40), an open product name (NC41) and icons (NC42). NC-R1 started. Pending with the owner: the feed repository, the certificate, the name.
- 2026-11-11 — Owner: no certificates. NC43 removes signing; auto-update stays. S1 no longer waits for a certificate.
