#!/usr/bin/env bash
# Imports the 50 curated Material Icon Theme 5.40.0 SVGs into ONE binary asset pack plus its MIT
# licence and SHA-256 manifest, so the app embeds them offline with include_bytes!.
# No arguments rebuilds both files; --verify only reads them and needs no source checkout.
set -euo pipefail
export LC_ALL=C
shopt -s nullglob

readonly MAGIC='SWICN1'
readonly COUNT=50
readonly MAX_SVG=65536
readonly DEFAULT_SRC='../navo-proto-sidewise/design/prototypes/sidewise-chosen/icons'
src='' pack='' license=''
names=()

usage() { printf '%s\n' "usage: import-material-icons.sh [--verify] [icons dir, default $DEFAULT_SRC]" >&2; }
die() { printf 'import-material-icons: %s\n' "$1" >&2; exit 1; }

# Writes <value> as 2 or 4 little-endian bytes to stdout.
le16() { printf '%b' "$(printf '\\x%02x\\x%02x' "$(($1 & 255))" "$(($1 >> 8 & 255))")"; }
le32() {
  printf '%b' "$(printf '\\x%02x\\x%02x\\x%02x\\x%02x' "$(($1 & 255))" "$(($1 >> 8 & 255))" "$(($1 >> 16 & 255))" "$(($1 >> 24 & 255))")"
}

# Collects the icon stems, then sorts by stem: a glob would order folder-config-open.svg first.
collect() {
  local svg size name
  local -a files=("$src"/*.svg)
  ((${#files[@]} == COUNT)) || die "$src holds ${#files[@]} SVGs, want exactly $COUNT"
  [[ -f "$src/LICENSE" ]] || die "$src/LICENSE is missing"
  for svg in "${files[@]}"; do
    size=$(wc -c <"$svg")
    ((size > 0 && size < MAX_SVG)) || die "$svg is $size bytes, want 1..$((MAX_SVG - 1))"
    [[ "$(head -c 4 "$svg")" == '<svg' ]] || die "$svg does not start with <svg"
    name="${svg##*/}"
    name="${name%.svg}"
    [[ "$name" =~ ^[a-z0-9][a-z0-9_-]*$ ]] || die "$svg: stem '$name' is not lower-case ASCII"
    names+=("$name")
  done
  mapfile -t names < <(printf '%s\n' "${names[@]}" | sort)
}

# Writes magic, u16 count, then [u16 name len][name][u32 svg len][svg] per icon.
write_pack() {
  local name
  {
    printf '%s' "$MAGIC"
    le16 "$COUNT"
    for name in "${names[@]}"; do
      le16 "${#name}"
      printf '%s' "$name"
      le32 "$(wc -c <"$src/$name.svg")"
      cat "$src/$name.svg"
    done
  } >"$pack"
}

# Writes the upstream MIT licence, then the SHA-256 of every packed SVG in pack order.
write_license() {
  local name sha
  {
    printf '%s\n' "Material Icon Theme 5.40.0 (npm material-icon-theme), MIT: the $COUNT SVGs packed" \
      'unmodified into material-icons.pack.' \
      'Upstream: https://github.com/material-extensions/vscode-material-icon-theme' \
      'Curated copy: navo-proto-sidewise/design/prototypes/sidewise-chosen/icons, see its README.md.'
    printf '\n'
    cat "$src/LICENSE"
    printf '\n%s\n' 'SHA-256 of each packed SVG, in pack order (icon stem, without .svg):'
    for name in "${names[@]}"; do
      sha="$(sha256sum <"$src/$name.svg")"
      printf '%s  %s\n' "${sha%% *}" "$name"
    done
  } >"$license"
}

# Checks the pack count and every payload against the licence manifest, writing nothing.
verify() {
  [[ -f "$pack" ]] || die "missing $pack; run without --verify to build it"
  [[ -f "$license" ]] || die "missing $license; run without --verify to build it"
  [[ "$(head -c 6 "$pack")" == "$MAGIC" ]] || die "$pack does not start with $MAGIC"
  local -a bytes
  read -r -a bytes <<<"$(od -An -v -tu1 "$pack" | tr '\n' ' ')"
  local count=$((bytes[6] + (bytes[7] << 8)))
  ((count == COUNT)) || die "pack holds $count icons, want exactly $COUNT"
  local -A want=()
  local sha name
  while read -r sha name; do want["$name"]="$sha"; done < <(grep -E '^[0-9a-f]{64}  ' "$license")
  ((${#want[@]} == COUNT)) || die "licence manifest lists ${#want[@]} icons, want exactly $COUNT"
  local at=8 i name_len len actual
  for ((i = 0; i < count; i++)); do
    ((at + 2 <= ${#bytes[@]})) || die "pack ends before the name length of icon $i"
    name_len=$((bytes[at] + (bytes[at + 1] << 8)))
    at=$((at + 2))
    ((at + name_len + 4 <= ${#bytes[@]})) || die "pack ends inside the header of icon $i"
    name="$(dd if="$pack" bs=1 skip="$at" count="$name_len" status=none)"
    at=$((at + name_len))
    len=$((bytes[at] + (bytes[at + 1] << 8) + (bytes[at + 2] << 16) + (bytes[at + 3] << 24)))
    at=$((at + 4))
    ((at + len <= ${#bytes[@]})) || die "pack ends inside the SVG of icon '$name'"
    actual="$(dd if="$pack" bs=1 skip="$at" count="$len" status=none | sha256sum)"
    at=$((at + len))
    [[ -n "${want[$name]:-}" ]] || die "pack icon '$name' has no licence manifest entry"
    [[ "${actual%% *}" == "${want[$name]}" ]] || die "icon '$name': SHA-256 differs from the manifest"
    unset "want[$name]"
  done
  ((${#want[@]} == 0)) || die "manifest icons missing from the pack: ${!want[*]}"
  ((at == ${#bytes[@]})) || die "pack has $((${#bytes[@]} - at)) trailing bytes"
  printf 'import-material-icons: verified %s icons in %s\n' "$count" "$pack"
}

main() {
  if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then usage; exit 0; fi
  local mode=build
  if [[ "${1:-}" == "--verify" ]]; then mode=verify; shift; fi
  if [[ $# -gt 1 || "${1:-}" == -* ]]; then usage; exit 2; fi
  local script_dir
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  cd -- "$script_dir/.."
  pack='assets/icons/material-icons.pack'
  license='assets/icons/material-LICENSE.txt'
  if [[ "$mode" == verify ]]; then verify; exit 0; fi
  src="${1:-$DEFAULT_SRC}"
  [[ -d "$src" ]] || die "icons dir not found: $src"
  command -v sha256sum >/dev/null || die 'sha256sum missing'
  collect
  mkdir -p assets/icons
  write_pack
  write_license
  printf 'import-material-icons: packed %s icons (%s bytes) into %s\n' "$COUNT" "$(wc -c <"$pack")" "$pack"
}

main "$@"
