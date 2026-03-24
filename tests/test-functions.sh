test_functions_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd)"
test_functions_script_dir="$(readlink --canonicalize "$test_functions_script_dir")"

# shellcheck source-path=SCRIPTDIR
. "$test_functions_script_dir"/colors.sh

test_name="$(readlink --canonicalize "$0")"
test_name="${test_name/$test_functions_script_dir\//}"

fail_count=0

print_results()
{
    if (( fail_count == 0 ))
    then
        echo -e "${green}[ PASS ]$reset_color $test_name"
    else
        echo -e "${red}[ FAIL ]$reset_color $test_name"
    fi
}

trap print_results EXIT

# Takes the output of trap -p as parameter (trap -- 'command'
# signal) and prints the command
print_trap_command()
{
    printf '%s' "$3"
}

push_on_exit()
{
    local new_commands

    new_commands="$(echo -n "$@" ';'; eval print_trap_command $(trap -p EXIT))"

    trap -- "$new_commands" EXIT
}

expect_true()
{
    set +e
    "$@"
    local e=$?
    set -e

    if (( e != 0 ))
    then
        fail_count=$((fail_count + 1))
        echo -e "${red}[ FAIL ]$reset_color" "$@"
        echo "Command should have exited normally. Exit code is $e."
    else
        echo -e "${green}[ PASS ]$reset_color" "$@"
    fi
}

expect_false()
{
    if "$@"
    then
        fail_count=$((fail_count + 1))
        echo -e "${red}[ FAIL ]$reset_color" "$@"
        echo "Command should have failed."
    else
        echo -e "${green}[ PASS ]$reset_color" "!" "$@"
    fi
}

expect_eq()
{
    local expected
    expected="$1"

    local actual
    actual="$(eval "$2")"

    if [[ "$expected" = "$actual" ]]
    then
        echo -e "${green}[ PASS ]$reset_color '$2' = '$1'."
    else
        fail_count=$((fail_count + 1))
        echo -e "${red}[ FAIL ]$reset_color '$2' = '$1'."
        echo "Expected: $expected"
        echo "  Actual: $actual"
    fi
}

expect_json_eq()
{
    local expected
    expected="$(echo "$1" | jq --sort-keys --compact-output . "$2")"

    local actual
    actual="$(jq --sort-keys --compact-output . "$2")"

    if [[ "$expected" = "$actual" ]]
    then
        echo -e "${green}[ PASS ]$reset_color json_eq '$2'."
    else
        fail_count=$((fail_count + 1))
        echo -e "${red}[ FAIL ]$reset_color json_eq '$2'."
        echo "Expected: $expected"
        echo "  Actual: $actual"
    fi
}

# -- in another file

tmp_dir="$(mktemp --directory)"

service="https://localhost:3000"

cd "$test_functions_script_dir/../"

# TODO: Use different ports
container_name="test-$(echo -n "$test_name" | tr -c 'a-zA-Z0-9_.\-' '.')"
docker run --rm --name "$container_name" \
         --env POSTGRES_PASSWORD=postgres \
         --publish 5432:5432 \
         postgres:18 \
         > "$tmp_dir"/postgres.out.txt \
         2> "$tmp_dir"/postgres.err.txt \
         &
sleep 2

# TODO: pass --binary
./target/debug/axum-crud-test \
      > "$tmp_dir"/server.out.txt \
      2> "$tmp_dir"/server.err.txt \
    &
server_pid=$!
sleep 2

kill_services()
{
    kill -15 "$server_pid"
    kill -0 "$server_pid" || kill -9 "$server_pid"

    docker stop "$container_name"
}

push_on_exit kill_services

rm_tmp_dir()
{
    if (( fail_count == 0 ))
    then
        rm --force --recursive "$tmp_dir"
    else
        echo "Temporary files are in '$tmp_dir'."
    fi
}

push_on_exit rm_tmp_dir

do_curl()
{
    local resource="$service/$1"
    shift

    curl --silent --show-error --fail --cacert \
         "$test_functions_script_dir"/../certificates/localhost.crt \
         "$resource" \
         "$@"
}

_expect_curl_error()
{
    local expected="$1"
    shift

    local tmp
    tmp="$(mktemp --tmpdir="$tmp_dir")"

    expect_false do_curl "$@" 2> "$tmp"

    if grep --quiet 'The requested URL returned error' "$tmp"
    then
        local actual
        actual="$(sed 's/.\+: //' "$tmp")"

        if [[ "$expected" = "$actual" ]]
        then
            echo -e "${green}[ PASS ]$reset_color error code $expected for" "$@"
        else
            fail_count=$((fail_count + 1))
            echo -e "${red}[ FAIL ]$reset_color Wrong error code."
            echo "Expected: $expected"
            echo "  Actual: $actual"
        fi
    else
        fail_count=$((fail_count + 1))
        echo -e "${red}[ FAIL ]$reset_color Wrong error kind."
        echo "Expected: $expected"
        echo "  Actual: $(cat "$tmp")"
    fi
}

expect_get()
{
    expect_true do_curl "$@"
}

expect_post()
{
    expect_true do_curl "$@" -X POST
}

expect_get_error()
{
    _expect_curl_error "$@"
}

expect_post_error()
{
    _expect_curl_error "$@" -X POST
}

