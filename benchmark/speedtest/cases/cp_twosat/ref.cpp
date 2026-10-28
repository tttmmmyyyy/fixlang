// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/twosat>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 100000, m = 200000;
    auto sol = speedtest::rands(n, 2, 1);
    auto ii = speedtest::rands(m, n, 2), jj = speedtest::rands(m, n, 3), ff = speedtest::rands(m, 2, 4), gg = speedtest::rands(m, 2, 5);
    vector<ll> fs(m);
    for (int k = 0; k < m; k++) fs[k] = (sol[ii[k]] != ff[k] && sol[jj[k]] != gg[k]) ? sol[ii[k]] : ff[k];
    speedtest::perf_enable();
    bool ok;
    {
        atcoder::two_sat ts(n);
        for (int k = 0; k < m; k++) ts.add_clause(ii[k], fs[k] == 1, jj[k], gg[k] == 1);
        ok = ts.satisfiable();
    }
    speedtest::perf_disable();
    return speedtest::check("1\n", "%d\n", ok ? 1 : 0);
}
