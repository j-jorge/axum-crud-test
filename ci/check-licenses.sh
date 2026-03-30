#!/bin/bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd)"

tmp_file="$(mktemp)"
trap clean_up EXIT

clean_up()
{
    rm --force "$tmp_file"
}

"$script_dir"/find-sources.sh -name "*.rs" -print0 \
    | ( xargs -0 grep --files-without-match '// SPDX' || true ) \
          > "$tmp_file"

if [[ -s "$tmp_file" ]]
then
    echo "Those files have no license:"
    cat "$tmp_file"
    exit 1
fi
