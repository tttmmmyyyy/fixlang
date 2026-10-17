// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/string>
#include <string>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 10000000;
    auto cs = speedtest::rands(n, 2, 1);
    string s(n, ' '); for (int i = 0; i < n; i++) s[i] = 'a' + cs[i];
    speedtest::perf_enable();
    auto z = atcoder::z_algorithm(s);
    speedtest::perf_disable();
    ll acc = 0; for (ll x : z) acc += x;
    return speedtest::check("20006181\n", "%lld\n", acc);
}
