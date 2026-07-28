# Shared by the harnesses in this directory: choosing which programs to work on.
#
# Sourced after the caller has changed into this directory, so `programs.txt` is at hand.

SELECTED=()

# Record the program names the caller was given, and reject one that is not a program --
# a typo would otherwise select nothing, and the measurement that follows would report the
# binaries left over from a previous run as if they were this one's.
select_programs() {
    SELECTED=("$@")
    local unknown=()
    local name
    for name in ${SELECTED[@]+"${SELECTED[@]}"}; do
        grep -qE "^$name[[:space:]]" programs.txt || unknown+=("$name")
    done
    if [ ${#unknown[@]} -ne 0 ]; then
        echo "not in programs.txt: ${unknown[*]}" >&2
        exit 1
    fi
}

# True for every program when none were named.
is_wanted() {
    [ ${#SELECTED[@]} -eq 0 ] && return 0
    printf '%s\n' "${SELECTED[@]}" | grep -qx "$1"
}
