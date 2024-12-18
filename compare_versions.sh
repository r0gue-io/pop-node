#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <Cargo1.toml> <Cargo2.toml>"
    exit 1
fi

FILE1="$1"
FILE2="$2"

extract_versions() {
    local file="$1"
    sed -n '/^\[workspace\.dependencies\]/,/^\[/{p}' "$file" | \
    grep -E '^[^#[:space:]][[:alnum:]._-]+.*version\s*=' | \
    sed -E 's/^[[:space:]]*([[:alnum:]._-]+).*version\s*=\s*"(.*?)".*/\1:\2/' | \
    sort
}

diff --suppress-common-lines -u <(extract_versions "$FILE1") <(extract_versions "$FILE2") || true

