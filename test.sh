#!/bin/bash

set -ux

# hmm it'd be nice if we could, like, lazily evaluate the tests based on the output of this process
cargo build || exit 1

lazy() {
    ./target/debug/lazy "$@"
}

echo "no input"
(:) | lazy true
echo $?

echo "close at open"
# ah, this doesn't work probably because the input is a tty, so closing it has no effect (?)
./target/debug/lazy true <&-

echo "some input"
echo "hi" | lazy true

echo "some input & closed at open"
echo "hi" | (
    sleep 0.1
    lazy true
)

echo "some input & still open"
(
    trap 'echo >&2 PIPEd' PIPE
    echo -n "hi"
    sleep 0.2
    echo
    echo -n there
    echo
    echo >&2 "done"
) | lazy true

# ok so this is funny; since `lazy` exits immediately, this script moves on / ends before all the chilren's are cleaned up
# lol: https://unix.stackexchange.com/questions/351780/wait-for-bash-process-substitution-subshells
# echo "some input & still open 2"
# ./target/debug/lazy true < <(
#     echo -n "hi"
#     sleep 2
#     echo >&2 "in stderr done"
#     echo
# )

# wait





# LOL gotcha
<<<"" lazy true ; echo $?
# => 0
