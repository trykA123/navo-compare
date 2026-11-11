#!/usr/bin/env bash
# The quality gate of this repo: format, lints, tests, cargo-deny and lock drift against the
# pinned navo rev. Run it before every merge to main; the first failing step stops it.
set -euo pipefail

usage() {
  printf '%s\n' "usage: gate.sh   (NAVO_LOCK=<navo's Cargo.lock at the pin> overrides the lookup)" >&2
}

# Prints the path of navo's lockfile at the pin: NAVO_LOCK, else cargo's Git checkout of the rev.
navo_lock() {
  if [[ -n "${NAVO_LOCK:-}" ]]; then
    printf '%s\n' "$NAVO_LOCK"
    return
  fi
  local found rev pin
  # The pin is read from the root manifest, so a pin bump needs no edit here.
  rev="$(grep -o -m 1 'rev = "[0-9a-f]\{40\}"' Cargo.toml)" || { printf '%s\n' "gate: no pinned rev in Cargo.toml" >&2; return 1; }
  pin="${rev:7:7}"
  shopt -s nullglob
  for found in "${CARGO_HOME:-$HOME/.cargo}"/git/checkouts/navo-*/"$pin"/Cargo.lock; do
    printf '%s\n' "$found"
    return
  done
  printf '%s\n' "gate: navo's lockfile at $pin not found; build once or set NAVO_LOCK" >&2
  return 1
}

# Runs one named step and prints its result; a failing step ends the gate with exit code 1.
step() {
  local name="$1"
  shift
  printf '%s\n' "=== $name" >&2
  if ! "$@"; then
    printf '%s\n' "=== FAIL $name" >&2
    exit 1
  fi
}

main() {
  if [[ "${1:-}" == "-h" ]]; then usage; exit 0; fi
  if [[ $# -gt 0 ]]; then usage; exit 2; fi
  command -v cargo-deny >/dev/null || { printf '%s\n' "cargo-deny missing" >&2; exit 1; }
  local script_dir other
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  cd -- "$script_dir/.."
  other="$(navo_lock)"
  step fmt cargo fmt --all --check
  step clippy cargo clippy --workspace --all-targets --locked -- -D warnings
  step test cargo test --workspace --locked
  step deny cargo deny --locked check
  step lock-drift cargo xtask lock-drift --other "$other"
  printf '%s\n' "=== ALL GREEN" >&2
}

main "$@"
