# shellcheck shell=bash

if printf '%s\n' "$@" | grep --quiet '^\(-h\|--help\)$'
then
    cat <<EOF
Usage: "$@" OPTIONS

Where OPTIONS are:
  --binary PATH
     Path to the program to test. Required.
  -h, --help
     Display this message and exit.
EOF
    exit 0
fi

while [[ $# -ne 0 ]]
do
    arg="$1"
    shift

    case "$arg" in
        --binary)
            if [[ $# -eq 0 ]]
            then
                echo "Missing value for --binary." >&2
                exit 1
            fi

            _server_binary="$1"
            shift
            ;;
        *)
            echo "Unsupported argument '$arg'." >&2
            exit 1
            ;;
    esac
done

if [[ "${_server_binary:-}" = "" ]]
then
    echo "--binary is required."
    exit 1
fi

test_functions_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd)"
test_functions_script_dir="$(readlink --canonicalize "$test_functions_script_dir")"

# shellcheck source-path=SCRIPTDIR
. "$test_functions_script_dir"/testlib.sh

tmp_dir="$(mktemp --directory)"

service="https://localhost:3000"

cd "$test_functions_script_dir/../"

kill_services()
{
    echo "Killing services, server_pid='${server_pid:-}', container='${container_name:-}'."
    if [[ "${server_pid:-}" != "" ]]
    then
        kill -15 "$server_pid"
        kill -0 "$server_pid" || kill -9 "$server_pid"
    fi

    if [[ "${container_name:-}" != "" ]]
    then
        docker stop "$container_name"
    fi
}

push_on_exit kill_services

wait_poll_file()
{
    local try_count="$1"
    local f="$2"
    local regex="$3"

    echo -e "${yellow}[ INFO ]$reset_color Waiting for '$regex' in '$f'."

    while (( try_count >= 1 ))
    do
        if grep --quiet "$regex" "$f"
        then
            return 0
        fi

        try_count=$((try_count - 1))
        sleep 1
    done

    echo -e \
         "${red}[ FAIL ]$reset_color Could not find pattern '$regex' in '$f':" \
         >&2
    cat "$f"

    fail_count=$((fail_count + 1))

    return 1
}

set -x

container_name="test-$(echo -n "$test_name" | tr -c 'a-zA-Z0-9_.\-' '.')"
docker run --rm --name "$container_name" \
         --env POSTGRES_PASSWORD=postgres \
         --publish 5432:5432 \
         postgres:18 \
         > "$tmp_dir"/postgres.out.txt \
         2> "$tmp_dir"/postgres.err.txt \
         &
wait_poll_file 60 "$tmp_dir"/postgres.out.txt "ready for start up"

"$_server_binary" \
      > "$tmp_dir"/server.out.txt \
      2> "$tmp_dir"/server.err.txt \
    &
server_pid=$!
wait_poll_file 60 "$tmp_dir"/server.out.txt 'Starting the web services'

rm_tmp_dir()
{
    if (( fail_count == 0 ))
    then
        rm --force --recursive "$tmp_dir"
    else
        echo "Temporary files are in '$tmp_dir'."

        if [[ "${CI:-}" = "true" ]]
        then
            find "$tmp_dir" -type f \
                | while read -r f
            do
                echo "==== $f ===="
                cat "$f"
            done
        fi
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

