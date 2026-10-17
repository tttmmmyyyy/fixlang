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
    int q = 1000000; ll m = 1000000007;
    auto xs = speedtest::rands(q, m - 1, 1);
    speedtest::perf_enable();
    ll acc = 0;
    for (int i = 0; i < q; i++) acc ^= atcoder::inv_mod(xs[i] + 1, m);
    speedtest::perf_disable();
    return speedtest::check("690247173\n", "%lld\n", acc);
}
