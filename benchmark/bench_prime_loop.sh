echo "C++:"
g++ -O2 ./benchmark/prime_loop.cpp && time ./a.out

echo ""
echo "Fix:"
cargo run -- build ./examples/prime_loop.fix && time ./examples/prime_loop