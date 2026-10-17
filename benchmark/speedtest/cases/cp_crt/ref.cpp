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
    auto m1s = speedtest::rands(q, 1000000000, 1), m2s = speedtest::rands(q, 1000000000, 2), r1s = speedtest::rands(q, 1000000000, 3), r2s = speedtest::rands(q, 1000000000, 4);
    speedtest::perf_enable();
    ll acc = 0;
    for (int i = 0; i < q; i++) {
        ll m1 = m1s[i] + 1, m2 = m2s[i] + 1;
        auto [y, z] = atcoder::crt({r1s[i] % m1, r2s[i] % m2}, {m1, m2});
        acc ^= y; acc ^= z;
    }
    speedtest::perf_disable();
    return speedtest::check("785281547879847128\n", "%lld\n", acc);
}
