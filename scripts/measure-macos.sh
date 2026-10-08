#!/bin/sh
# Measure an already-running instance, avoiding startup transients.
set -eu
[ "$#" = 1 ] || { echo "usage: $0 <pid>" >&2; exit 1; }
pid="$1"
case "$pid" in *[!0-9]*|'') echo "numeric pid required" >&2; exit 1;; esac
cd "$(dirname "$0")/.."
stat -f 'binary_bytes=%z' target/release/tomito
du -sk "dist/Tomito RS.app"
ps -p "$pid" -o pid,rss,%cpu,time,comm
vmmap -summary "$pid" | grep -E 'Physical footprint|Physical footprint \(peak\)'
echo "RSS is KiB. vmmap footprint includes process-owned memory; GPU/system caches differ."
