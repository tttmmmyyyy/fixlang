// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/segtree>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

ll op(ll a, ll b) { return max(a, b); }
ll e() { return LLONG_MIN; }
int main() {
    speedtest::perf_start();
    int n = 200000, q = 200000;
    auto init = speedtest::rands(n, 1000000000, 1);
    auto ts = speedtest::rands(q, 2, 2), aa = speedtest::rands(q, n, 3), bb = speedtest::rands(q, n, 4), xs = speedtest::rands(q, 1000000000, 5);
    speedtest::perf_enable();
    atcoder::segtree<ll, op, e> t(init);
    ll acc = 0;
    for (int i = 0; i < q; i++) {
        ll a = aa[i], b = bb[i];
        if (ts[i] == 0) { t.set(a, xs[i]); continue; }
        ll l = min(a, b), r = max(a, b);
        acc += t.prod(l, r + 1);
    }
    speedtest::perf_disable();
    return speedtest::check("100142147228725\n", "%lld\n", acc);
}
