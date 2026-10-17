// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/dsu>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 2000000, q = 2000000;
    auto ts = speedtest::rands(q, 2, 2), aa = speedtest::rands(q, n, 3), bb = speedtest::rands(q, n, 4);
    speedtest::perf_enable();
    atcoder::dsu d(n);
    ll acc = 0;
    for (int i = 0; i < q; i++) {
        if (ts[i] == 0) { d.merge(aa[i], bb[i]); continue; }
        acc += d.same(aa[i], bb[i]) ? 1 : 0;
    }
    speedtest::perf_disable();
    return speedtest::check("5\n", "%lld\n", acc);
}
