# cargo build --release -- --emit=llvm-bc
# cargo build --release -- --emit=llvm-ir
rm -r target
cargo rustc --release -- --emit=llvm-bc && cp ./target/release/deps/fixruntime*.bc ./fixruntime.bc
cargo rustc --release -- --emit=llvm-ir && cp ./target/release/deps/fixruntime*.ll ./fixruntime.ll
cargo rustc --release -- --emit=llvm-ir && cp ./target/release/deps/libfixruntime*.so ./libfixruntime.so