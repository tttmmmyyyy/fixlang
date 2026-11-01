// The counterpart of `main.fix`: the same input, given to ac-library, which has no bipartite matching, so it
// solves it as a maximum flow with `mf_graph` and reads the matching off the flow, as cp-library does. Only the
// part that calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/maxflow>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int l = 50000, m = 200000;
    auto us = speedtest::rands(m, l, 1), vs = speedtest::rands(m, l, 2);
    int s = 2 * l, t = 2 * l + 1;
    speedtest::perf_enable();
    // Whether each of the `m` edges is in the matching, one byte per edge as `Array Bool` holds it.
    vector<char> matching(m);
    {
        atcoder::mf_graph<ll> g(2 * l + 2);
        for (int i = 0; i < l; i++) { g.add_edge(s, i, 1); g.add_edge(l + i, t, 1); }
        vector<int> ids(m);
        for (int i = 0; i < m; i++) ids[i] = g.add_edge(us[i], l + vs[i], 1);
        g.flow(s, t);
        for (int i = 0; i < m; i++) matching[i] = g.get_edge(ids[i]).flow > 0;
    }
    speedtest::perf_disable();
    ll cnt = 0; for (char b : matching) cnt += b;
    return speedtest::check("48869\n", "%lld\n", cnt);
}
