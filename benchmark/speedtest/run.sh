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

# If the log file is empty, write the header line "commit_has, instructions, memory_accesses"
if [ ! -f log.csv ]; then
    echo "commit_hash,instructions,memory_accesses" > log.csv
fi

# If the last line of the log file start with the commit hash, remove it.
if [ -f log.csv ]; then
    if [ "$(tail -n 1 log.csv | cut -d ',' -f 1)" == "$COMMIT_HASH" ]; then
        sed -i '$ d' log.csv
    fi
fi

# Append the commit hash and estimate to the log file
echo "$COMMIT_HASH,$ESTIMATE" >> log.csv

# Run "graph.py"
python3 graph.py