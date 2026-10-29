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
    using mint = atcoder::modint998244353;
    auto a_ll = speedtest::rands(n, 998244353, 1), b_ll = speedtest::rands(n, 998244353, 2);
    // The input and the output are held as `mint`, as `main.fix` holds them as `ZP`, so that converting
    // between them and `ll` is left out of the count on both sides.
    vector<mint> a(a_ll.begin(), a_ll.end()), b(b_ll.begin(), b_ll.end());
    speedtest::perf_enable();
    auto c = atcoder::convolution(a, b);
    speedtest::perf_disable();
    ll acc = 0; for (mint x : c) acc += x.val();
    return speedtest::check("199293543703014\n", "%lld\n", acc);
}
