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
    int n = 1000000;
    auto cs = speedtest::rands(n, 26, 1);
    string s(n, ' '); for (int i = 0; i < n; i++) s[i] = 'a' + cs[i];
    speedtest::perf_enable();
    auto sa = atcoder::suffix_array(s);
    speedtest::perf_disable();
    ll acc = 0; for (ll i = 0; i < n; i++) acc += sa[i] * i;
    return speedtest::check("249973518341177452\n", "%lld\n", acc);
}
