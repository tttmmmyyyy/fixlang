// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/convolution>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 200000;
    auto a = speedtest::rands(n, 1000000, 1), b = speedtest::rands(n, 1000000, 2);
    speedtest::perf_enable();
    auto c = atcoder::convolution_ll(a, b);
    speedtest::perf_disable();
    ll acc = 0; for (ll x : c) acc ^= x;
    return speedtest::check("21474833636446616\n", "%lld\n", acc);
}
