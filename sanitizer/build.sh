# cargo build --release -- --emit=llvm-bc
# cargo build --release -- --emit=llvm-ir
# rm -r target
# cargo rustc --release -- --emit=llvm-bc && cp ./target/release/deps/fixsanitizer*.bc ./fixsanitizer.bc
# cargo rustc --release -- --emit=llvm-ir && cp ./target/release/deps/fixsanitizer*.ll ./fixsanitizer.ll

# MacOS:
# cargo rustc --release -- --emit=llvm-ir && cp ./target/release/deps/libfixsanitizer*.dylib ./libfixsanitizer.so

# Windows:
cargo rustc --release -- --emit=llvm-ir && cp ./target/release/deps/libfixsanitizer*.so ./libfixsanitizer.so