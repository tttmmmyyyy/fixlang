// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/math>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int q = 200000;
    auto ns = speedtest::rands(q, 1000000000, 1), ms = speedtest::rands(q, 1000000000, 2), aa = speedtest::rands(q, 1000000000, 3), bb = speedtest::rands(q, 1000000000, 4);
    speedtest::perf_enable();
    ll acc = 0;
    for (int i = 0; i < q; i++) {
        ll m = ms[i] + 1;
        acc ^= atcoder::floor_sum(ns[i], m, aa[i] % m, bb[i] % m);
    }
    speedtest::perf_disable();
    return speedtest::check("498075712894463097\n", "%lld\n", acc);
}
