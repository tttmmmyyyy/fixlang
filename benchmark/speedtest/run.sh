set -e

LOG_FILE=speedtest.log

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

# If the last line of the log file start with the commit hash, remove it.
if [ -f speedtest.csv ]; then
    if [ "$(tail -n 1 speedtest.csv | cut -d ',' -f 1)" == "$COMMIT_HASH" ]; then
        sed -i '$ d' speedtest.csv
    fi
fi

# Append the commit hash and estimate to a file "sppedtest.csv"
echo "$COMMIT_HASH, $ESTIMATE" >> speedtest.csv