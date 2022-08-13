# cargo build --release -- --emit=llvm-bc
# cargo build --release -- --emit=llvm-ir
cargo rustc --release -- --crate-type=lib --emit=llvm-bc && cp ./target/release/deps/fixruntime-*.bc ./fixruntime.bc
cargo rustc --release -- --crate-type=lib --emit=llvm-ir && cp ./target/release/deps/fixruntime-*.ll ./fixruntime.ll