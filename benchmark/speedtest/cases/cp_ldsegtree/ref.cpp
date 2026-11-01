// The counterpart of `main.fix`: the same input, given to ac-library. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <atcoder/lazysegtree>
#include <atcoder/modint>
#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

using mint = atcoder::modint998244353;
struct S { mint a; int size; };
struct F { mint a, b; };
S op(S l, S r) { return {l.a + r.a, l.size + r.size}; }
S e() { return {0, 0}; }
S mapping(F f, S x) { return {f.a * x.a + f.b * x.size, x.size}; }
F composition(F f, F g) { return {f.a * g.a, f.a * g.b + f.b}; }
F id() { return {1, 0}; }
int main() {
    speedtest::perf_start();
    int n = 200000, q = 200000;
    auto init = speedtest::rands(n, 998244353, 1);
    auto ts = speedtest::rands(q, 2, 2), aa = speedtest::rands(q, n, 3), bb = speedtest::rands(q, n, 4);
    auto cs = speedtest::rands(q, 998244353, 5), ds = speedtest::rands(q, 998244353, 6);
    vector<S> v(n); for (int i = 0; i < n; i++) v[i] = {mint(init[i]), 1};
    speedtest::perf_enable();
    ll acc = 0;
    {
        atcoder::lazy_segtree<S, op, e, F, mapping, composition, id> t(v);
        for (int i = 0; i < q; i++) {
            ll a = aa[i], b = bb[i];
            ll l = min(a, b), r = max(a, b);
            if (ts[i] == 0) { t.apply(l, r + 1, F{mint(cs[i]), mint(ds[i])}); continue; }
            acc += t.prod(l, r + 1).a.val();
        }
    }
    speedtest::perf_disable();
    return speedtest::check("50032478004588\n", "%lld\n", acc);
}
