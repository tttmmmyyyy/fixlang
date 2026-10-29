// The counterpart of `main.fix`: the same input, given to a plain C++ implementation over the graph layout
// cp-library uses. Only the part that calls the library is counted, and the program checks the answer and prints
// nothing.

#include <climits>
#include <vector>
#include "graph.hpp"
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 20000, m = 100000;
    auto us = speedtest::rands(m, n, 1), vs = speedtest::rands(m, n, 2), cs = speedtest::rands(m, 1000000, 3);
    auto ps = speedtest::rands(n, 1000000, 4);
    speedtest::perf_enable();
    vector<ll> d;
    {
        vector<pair<int32_t, int32_t>> ends;
        vector<ll> costs;
        for (int i = 0; i < m; i++) {
            ends.push_back({(int32_t)us[i], (int32_t)vs[i]});
            costs.push_back(cs[i] + ps[us[i]] - ps[vs[i]]);
        }
        speedtest::Graph<ll> g(n, ends, costs);
        d.assign(n, LLONG_MAX); d[0] = 0;
        while (true) {
            bool update = false;
            for (int i = 0; i < n; i++) {
                if (d[i] == LLONG_MAX) continue;
                for (int p = g.start[i]; p < g.start[i + 1]; p++) {
                    int to = g.to[p];
                    ll nd = d[i] + g.data[p];
                    if (nd < d[to]) { d[to] = nd; update = true; }
                }
            }
            if (!update) break;
        }
    }
    speedtest::perf_disable();
    ll cnt = 0, sum = 0;
    for (ll x : d) if (x != LLONG_MAX) { cnt++; sum += x; }
    return speedtest::check("19876 39486003338\n", "%lld %lld\n", cnt, sum);
}
