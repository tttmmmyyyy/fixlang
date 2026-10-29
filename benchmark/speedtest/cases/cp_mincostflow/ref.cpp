// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/mincostflow>
#include <algorithm>
#include <climits>
#include <tuple>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 200;
    auto cs = speedtest::rands(n * n, 1000000000, 1);
    int s = 2 * n, t = 2 * n + 1;
    speedtest::perf_enable();
    ll f, c;
    {
        atcoder::mcf_graph<ll, ll> g(2 * n + 2);
        for (int i = 0; i < n; i++) { g.add_edge(s, i, 1, 0); g.add_edge(n + i, t, 1, 0); }
        for (int k = 0; k < n * n; k++) g.add_edge(k / n, n + k % n, 1, cs[k]);
        tie(f, c) = g.flow(s, t);
    }
    speedtest::perf_disable();
    return speedtest::check("200 1548180538\n", "%lld %lld\n", f, c);
}
