set -e

SCRIPT_DIR=$(dirname "$0")
pushd $SCRIPT_DIR
trap 'popd' EXIT

# Run the benchmarking script
cargo run -- build
OUTPUT=$(python3 ./cachegrind-benchmarking/cachegrind.py ./a.out)

# Get the last line of the output
ESTIMATE=$(echo "$OUTPUT" | tail -n 1)

# Get the current commit hash.
pushd ../../
COMMIT_HASH=$(git rev-parse HEAD)
popd

# Append the commit hash and estimate to a file "sppedtest.csv"
echo "$COMMIT_HASH, $ESTIMATE" >> speedtest.csv