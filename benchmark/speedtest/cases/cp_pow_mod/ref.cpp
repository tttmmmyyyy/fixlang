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
    int q = 1000000;
    auto xs = speedtest::rands(q, 1000000000, 1), es = speedtest::rands(q, 1000000000, 2), ms = speedtest::rands(q, 1000000000, 3);
    speedtest::perf_enable();
    ll acc = 0;
    for (int i = 0; i < q; i++) acc ^= atcoder::pow_mod(xs[i], es[i], (int)(ms[i] + 1));
    speedtest::perf_disable();
    return speedtest::check("95675632\n", "%lld\n", acc);
}
