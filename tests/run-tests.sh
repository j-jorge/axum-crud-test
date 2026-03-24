#!/bin/bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd)"
cd "$script_dir"

# shellcheck source-path=SCRIPTDIR
. ./colors.sh

fail_count=0

while read -r test_script
do
    echo -e "${green}[ RUN  ]$reset_color $test_script"

    if ! "$test_script"
    then
        fail_count=$((fail_count + 1))
    fi
done < <(find . -mindepth 2 -executable -name "*.sh")

if (( fail_count == 0 ))
then
    echo -e "${green}[ PASS ]$reset_color"
else
    echo -e "${red}[ FAIL ]$reset_color"
fi
