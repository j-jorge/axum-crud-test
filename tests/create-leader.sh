#!/bin/bash

set -euo pipefail

service="https://localhost:3000"

cd "$(dirname "${BASH_SOURCE[0]}")/../"

cargo run > server.out.txt 2> server.err.txt &
server_pid=$!
sleep 2
tmp_dir="$(mktemp --directory)"

clean_up()
{
    if [[ $? -eq 0 ]]
    then
        rm -fr "$tmp_dir"
    fi

    kill -9 "$server_pid"
}

trap clean_up EXIT

do_curl()
{
    curl --fail --cacert ./certificates/localhost.crt "$@"
}

! do_curl "$service"/leads/list
! do_curl "$service"/leads/create

# GET not allowed
! do_curl -H "Authorization: _" "$service"/leads/create

do_curl -X POST -H "Authorization: _" "$service"/leads/create \
     -o "$tmp_dir"/create-1.json
token="$(jq -r . "$tmp_dir"/create-1.json)"
echo "Token is '$token'"

do_curl -H "Authorization: $token" "$service"/leads/list \
     -o "$tmp_dir"/list-1.json

test "$(jq length "$tmp_dir"/list-1.json)" -eq 1

! do_curl -X POST -H "Authorization: _" "$service"/leads/create

do_curl -X POST -H "Authorization: $token" "$service"/leads/create \
     -o "$tmp_dir"/create-2.json

do_curl -H "Authorization: $token" "$service"/leads/list \
     -o "$tmp_dir"/list-2.json

test "$(jq length "$tmp_dir"/list-2.json)" -eq 2
