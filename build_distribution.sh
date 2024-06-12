# This script builds the release binary and checks for the presence of the username in the binary.
# Reference: https://github.com/rust-lang/rust/issues/75799

RUSTFLAGS="--remap-path-prefix=$HOME=fix-builder-home -C strip=symbols" cargo build --release
name_strings=$(cat target/release/fix | strings | grep $(whoami))

if [ -n "$name_strings" ]; then
  echo "Privacy warning: the binary contains your username. Do not distribute this binary."
  exit 1
fi
