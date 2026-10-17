// The counterpart of `main.fix`: the same input, given to a plain C++ implementation. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 20000, m = 100000;
    auto us = speedtest::rands(m, n, 1), vs = speedtest::rands(m, n, 2), cs = speedtest::rands(m, 1000000, 3);
    auto ps = speedtest::rands(n, 1000000, 4);
    speedtest::perf_enable();
    vector<vector<pair<int, ll>>> adj(n);
    for (int i = 0; i < m; i++) adj[us[i]].push_back({(int)vs[i], cs[i] + ps[us[i]] - ps[vs[i]]});
    vector<ll> d(n, LLONG_MAX); d[0] = 0;
    while (true) {
        bool update = false;
        for (int i = 0; i < n; i++) {
            if (d[i] == LLONG_MAX) continue;
            for (auto [to, c] : adj[i]) {
                ll nd = d[i] + c;
                if (nd < d[to]) { d[to] = nd; update = true; }
            }
        }
        if (!update) break;
    }
    speedtest::perf_disable();
    ll cnt = 0, sum = 0;
    for (ll x : d) if (x != LLONG_MAX) { cnt++; sum += x; }
    return speedtest::check("19876 39486003338\n", "%lld %lld\n", cnt, sum);
}
