#!/bin/bash
# Peak resident set size (RSS) per parser, per corpus file.
#
# Each standalone bin (src/{oxc,swc,tsv}.rs) parses one file and exits; we run it
# under GNU time, capture the max RSS, and average over $RUNS. tsv has no JSX
# grammar and rejects the `.js` corpus's TS-keyword identifiers, so it runs on
# the real-TypeScript `.ts` file only (the same scoping as the CPU benchmark).
#
# Needs GNU time (not the BSD/macOS built-in): `gtime` (macOS: brew install
# gnu-time) or a GNU `/usr/bin/time` (Debian/Ubuntu: apt install time).
set -euo pipefail

RUNS=10

cargo build --release

if command -v gtime >/dev/null 2>&1; then
  GNU_TIME=gtime
elif /usr/bin/time -f '%M' true >/dev/null 2>&1; then
  GNU_TIME=/usr/bin/time
else
  echo "GNU time not found (need 'gtime' or a GNU '/usr/bin/time')."
  echo "  macOS:         brew install gnu-time"
  echo "  Debian/Ubuntu: apt install time"
  exit 1
fi

# Average peak RSS (MB) for "$1" parsing "$2". GNU time writes %M (max RSS in KB)
# to its stderr; capture that, send the parser's own stdout to /dev/null.
measure() {
  local app="$1" file="$2" total=0 kb
  for _ in $(seq "$RUNS"); do
    kb=$("$GNU_TIME" -f '%M' "./target/release/$app" "$file" 2>&1 1>/dev/null)
    total=$((total + kb))
  done
  awk -v t="$total" -v n="$RUNS" 'BEGIN { printf "%.1f", t / n / 1000 }'
}

for FILE in "./files/cal.com.tsx" "./files/typescript.js" "./files/parser.ts"
do
  echo "$FILE"

  case "$FILE" in
    *.ts) APPS="oxc swc tsv" ;;  # real TypeScript — tsv included
    *)    APPS="oxc swc" ;;       # JSX (.tsx) / JS (.js) — tsv excluded
  esac

  for APP in $APPS
  do
    echo "$APP $(measure "$APP" "$FILE") mb"
  done
done
