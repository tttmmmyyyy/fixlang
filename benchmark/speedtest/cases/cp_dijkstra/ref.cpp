// The counterpart of `main.fix`: the same input, given to a plain C++ implementation on `std::priority_queue`, over
// the graph layout cp-library uses. Only the part that calls the library is counted, and the program checks the
// answer and prints nothing.

#include <queue>
#include <climits>
#include <vector>
#include "graph.hpp"
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 200000, m = 500000;
    auto us = speedtest::rands(m, n, 1), vs = speedtest::rands(m, n, 2), ws = speedtest::rands(m, 1000000000, 3);
    speedtest::perf_enable();
    vector<pair<int32_t, int32_t>> ends;
    vector<ll> costs;
    for (int i = 0; i < m; i++) { ends.push_back({(int32_t)us[i], (int32_t)vs[i]}); costs.push_back(ws[i]); }
    speedtest::Graph<ll> g(n, ends, costs);
    vector<ll> d(n, LLONG_MAX);
    priority_queue<pair<ll, int>, vector<pair<ll, int>>, greater<pair<ll, int>>> pq;
    d[0] = 0; pq.push({0, 0});
    while (!pq.empty()) {
        auto [dist, v] = pq.top(); pq.pop();
        if (d[v] < dist) continue;
        for (int p = g.start[v]; p < g.start[v + 1]; p++) {
            int to = g.to[p];
            ll nd = dist + g.data[p];
            if (nd < d[to]) { d[to] = nd; pq.push({nd, to}); }
        }
    }
    speedtest::perf_disable();
    ll cnt = 0, sum = 0;
    for (ll x : d) if (x != LLONG_MAX) { cnt++; sum += x; }
    return speedtest::check("178344 940675191237638\n", "%lld %lld\n", cnt, sum);
}
