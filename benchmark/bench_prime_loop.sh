echo "C++:"
g++ ./benchmark/prime_loop.cpp && time ./a.out

echo ""
echo "Fix:"
cargo run -- build ./examples/prime_loop.fix && gcc ./examples/prime_loop.o && time ./a.out