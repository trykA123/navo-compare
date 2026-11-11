#!/usr/bin/env bash
# Builds and checks one commit of this repo inside the Windows VM (atom V0 and the later gate).
# It sends the commit as a Git bundle, runs vm-build.ps1 there and keeps the log on the host.
set -euo pipefail

readonly LOCK_WAIT_S=3600
readonly REMOTE_DIR='C:/winvm'

usage() {
  printf '%s\n' "usage: WINVM=<path to winvm.sh> vm-build.sh <rev> [clippy,test,build,release]" >&2
}

# Sends <rev> to the VM and runs the stages; fails when a stage fails or the VM lock stays busy.
main() {
  if [[ "${1:-}" == "-h" ]]; then usage; exit 0; fi
  if [[ $# -lt 1 || "$1" == -* ]]; then usage; exit 2; fi
  local rev="$1" stages="${2:-clippy,test,build}"
  local winvm="${WINVM:?set WINVM to the path of winvm.sh}"
  local logs="${WINVM_LOGS:-$HOME/.cache/winvm-ci}"
  local script_dir repo sha log
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  repo="$(git -C "$script_dir" rev-parse --show-toplevel)"
  sha="$(git -C "$repo" rev-parse --verify "$rev^{commit}")"
  mkdir -p "$logs"

  # One VM serves every project: the lock is the one Navo's CI run takes.
  exec 9>"$logs/.vm.lock"
  if ! flock -w "$LOCK_WAIT_S" 9; then
    printf '%s\n' "vm-build: the VM stayed busy for ${LOCK_WAIT_S} s" >&2
    exit 75
  fi

  bundle="$(mktemp --suffix=.bundle)"
  trap 'rm -f -- "$bundle"' EXIT
  git -C "$repo" branch -f vm-build "$sha" >/dev/null
  git -C "$repo" bundle create -q "$bundle" vm-build
  git -C "$repo" branch -D vm-build >/dev/null

  "$winvm" scp -q "$bundle" "admin@127.0.0.1:$REMOTE_DIR/sidewise.bundle"
  "$winvm" scp -q "$script_dir/vm-build.ps1" "admin@127.0.0.1:$REMOTE_DIR/sidewise-build.ps1"
  log="$logs/sidewise-${sha:0:8}.log"
  printf '%s\n' "log: $log" >&2
  local status=0
  "$winvm" ssh "powershell -NoProfile -ExecutionPolicy Bypass -File C:\\winvm\\sidewise-build.ps1 -Sha $sha -Stages $stages" \
    > "$log" 2>&1 9>&- || status=$?
  grep -E '^=== ' "$log" || true
  if [[ "$status" -ne 0 ]] || ! grep -q '^=== ALL GREEN' "$log"; then
    printf '%s\n' "vm-build: failed (ssh exit $status); read $log" >&2
    exit 1
  fi
}

main "$@"
