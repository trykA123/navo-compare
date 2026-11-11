#!/usr/bin/env bash
# Pin canary: builds and tests this repo against another commit of navo (default: the head of
# its main branch) inside the Windows VM. It only reports; the pin in the repo is not changed.
set -euo pipefail

readonly NAVO_URL='https://github.com/trykA123/navo'
tmp=''

usage() {
  printf '%s\n' "usage: WINVM=<path to winvm.sh> canary.sh [navo commit, 40 hex digits]" >&2
}

# Removes the temporary worktree; runs on every exit.
cleanup() {
  if [[ -n "$tmp" && -d "$tmp" ]]; then
    git -C "$repo" worktree remove --force "$tmp" >/dev/null 2>&1 || rm -rf -- "${tmp:?}"
  fi
}

# Prints the commit to test: the argument, else the head of navo's main branch on the remote.
target_rev() {
  local rev="${1:-}"
  if [[ -z "$rev" ]]; then
    rev="$(git ls-remote "$NAVO_URL" refs/heads/main | cut -f1)"
  fi
  if [[ ! "$rev" =~ ^[0-9a-f]{40}$ ]]; then
    printf '%s\n' "canary: '$rev' is not a full commit id" >&2
    return 1
  fi
  printf '%s\n' "$rev"
}

# Makes a throwaway commit with the pin replaced by <rev> and prints its id.
# Fails when the manifests hold no pin or cargo cannot resolve the new lockfile.
canary_commit() {
  local pin="$1" rev="$2" manifest
  git -C "$repo" worktree add -q --detach "$tmp" HEAD
  while IFS= read -r manifest; do
    sed -i "s/$pin/$rev/g" "$tmp/$manifest"
  done < <(git -C "$tmp" grep -l "$pin" -- '*Cargo.toml')
  # Without --locked cargo rewrites only the lockfile entries that the new commit changes.
  (cd "$tmp" && cargo fetch >&2)
  git -C "$tmp" commit -q -a -m "canary: navo at ${rev:0:8}"
  git -C "$tmp" rev-parse HEAD
}

# Runs the canary and prints GREEN or RED; exit code 1 on RED, 2 on a usage error.
main() {
  if [[ "${1:-}" == "-h" ]]; then usage; exit 0; fi
  if [[ $# -gt 1 || "${1:-}" == -* ]]; then usage; exit 2; fi
  command -v cargo >/dev/null || { printf '%s\n' "canary: cargo missing" >&2; exit 1; }
  : "${WINVM:?set WINVM to the path of winvm.sh}"
  local script_dir pin rev sha
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  repo="$(git -C "$script_dir" rev-parse --show-toplevel)"
  pin="$(grep -o -m 1 'rev = "[0-9a-f]\{40\}"' "$repo/Cargo.toml" | cut -d'"' -f2)"
  [[ -n "$pin" ]] || { printf '%s\n' "canary: no pinned rev in Cargo.toml" >&2; exit 1; }
  rev="$(target_rev "${1:-}")"
  if [[ "$rev" == "$pin" ]]; then
    printf '%s\n' "canary: navo ${rev:0:8} is the pin; nothing to test"
    exit 0
  fi
  tmp="$(mktemp -d --suffix=.canary)"
  rmdir -- "$tmp"
  trap cleanup EXIT
  sha="$(canary_commit "$pin" "$rev")"
  printf '%s\n' "canary: pin ${pin:0:8} -> navo ${rev:0:8}, commit ${sha:0:8}" >&2
  if "$script_dir/vm-build.sh" "$sha" clippy,test; then
    printf '%s\n' "canary: GREEN against navo ${rev:0:8}"
  else
    printf '%s\n' "canary: RED against navo ${rev:0:8}"
    exit 1
  fi
}

main "$@"
