# shellcheck shell=bash

testlib_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd)"
testlib_script_dir="$(readlink --canonicalize "$testlib_script_dir")"

# shellcheck source-path=SCRIPTDIR
. "$testlib_script_dir"/colors.sh

test_name="$(readlink --canonicalize "$0")"
test_name="${test_name/$testlib_script_dir\//}"

fail_count=0

print_results()
{
    if (( $? != 0 ))
    then
        echo -e "${red}[ FAIL ]$reset_color $test_name: script failed"
        fail_count=$((fail_count + 1))
    elif (( fail_count == 0 ))
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

    # ShellCheck suggests to put the $(trap …) between quotes to
    # prevent word splitting, but the intent here is to split. The
    # output of trap -p has the commands quoted so it fits nicely with
    # eval.
    #
    # shellcheck disable=SC2046
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
    expected="$(echo "$1" | jq --sort-keys --compact-output .)"

    local actual
    actual="$(jq --sort-keys --compact-output . "$2")"

    if [[ "$expected" = "$actual" ]]
    then
        echo -e "${green}[ PASS ]$reset_color json_eq '$1' = '$2'."
    else
        fail_count=$((fail_count + 1))
        echo -e "${red}[ FAIL ]$reset_color json_eq '$2'."
        echo "Expected: $expected"
        echo "  Actual: $actual"
    fi
}
