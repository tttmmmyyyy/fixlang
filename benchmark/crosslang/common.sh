# Shared by the harnesses in this directory: finding the cases that carry counterparts.
#
# A case under `../speedtest/cases/` is comparable when it holds `ref.c` and `ref.rs` --
# the same program on the same input as its `main.fix`. Sourced after the caller has
# changed into this directory.

CASES=../speedtest/cases
SELECTED=()

# Every case with counterparts, in directory order.
comparable_cases() {
    local dir
    for dir in "$CASES"/*/; do
        [ -f "$dir/ref.c" ] && [ -f "$dir/ref.rs" ] && basename "$dir"
    done
    # A lister, not a predicate: the last directory failing the test is not an error.
    return 0
}

# Record the case names the caller was given, and reject one that is not comparable -- a
# typo would otherwise select nothing, and the measurement that follows would report the
# binaries left over from a previous run as if they were this one's.
select_cases() {
    SELECTED=("$@")
    local unknown=() name
    # Matched against a here-string rather than through a pipe: `grep -q` closes the pipe
    # on its first match, and with `pipefail` that failure would come back as "no match".
    local all
    all=$(comparable_cases)
    for name in ${SELECTED[@]+"${SELECTED[@]}"}; do
        grep -qx "$name" <<<"$all" || unknown+=("$name")
    done
    if [ ${#unknown[@]} -ne 0 ]; then
        echo "no case with ref.c and ref.rs named: ${unknown[*]}" >&2
        exit 1
    fi
}

# True for every comparable case when none were named.
is_wanted() {
    [ ${#SELECTED[@]} -eq 0 ] && return 0
    grep -qx "$1" <<<"$(printf '%s\n' "${SELECTED[@]}")"
}
