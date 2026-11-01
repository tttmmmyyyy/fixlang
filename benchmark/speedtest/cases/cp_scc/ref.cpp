// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/scc>
#include <algorithm>
#include <climits>
#include <vector>
#include <sys/resource.h>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

// ac-library's `scc_graph` searches depth first by recursion, as deep as the paths it walks, which
// overflows the default stack of 8 MiB on this graph. The judges ac-library is written for run with
// a far larger stack, and so does this program: the main thread's stack grows up to whatever limit
// is in force when it grows.
static void raise_stack_limit() {
    rlimit limit;
    getrlimit(RLIMIT_STACK, &limit);
    limit.rlim_cur = limit.rlim_max;
    setrlimit(RLIMIT_STACK, &limit);
}

int main() {
    raise_stack_limit();
    speedtest::perf_start();
    int n = 200000, m = 500000;
    auto us = speedtest::rands(m, n, 1), vs = speedtest::rands(m, n, 2);
    speedtest::perf_enable();
    vector<vector<int>> sccs;
    {
        atcoder::scc_graph g(n);
        for (int i = 0; i < m; i++) g.add_edge(us[i], vs[i]);
        sccs = g.scc();
    }
    speedtest::perf_disable();
    size_t largest = 0; for (auto& c : sccs) largest = max(largest, c.size());
    return speedtest::check("40859 159142\n", "%zu %zu\n", sccs.size(), largest);
}
