echo "C++:"
g++ -O2 ./benchmark/prime_loop.cpp && time ./a.out

echo ""
echo "Fix:"
cargo run -- build -f ./benchmark/prime_loop.fix && time ./a.out