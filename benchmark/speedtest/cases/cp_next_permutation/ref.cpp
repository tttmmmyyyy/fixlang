// The counterpart of `main.fix`: the same input, given to `std::next_permutation`. Only the part that
// calls the library is counted, and the program checks the answer and prints nothing.

#include <algorithm>
#include <climits>
#include <vector>
#include "speedtest.hpp"
using namespace std;
using ll = long long;

int main() {
    speedtest::perf_start();
    int n = 11;
    vector<ll> arr(n); for (int i = 0; i < n; i++) arr[i] = i;
    speedtest::perf_enable();
    ll acc = 0;
    do { acc += arr[0] * arr[n - 1]; } while (next_permutation(arr.begin(), arr.end()));
    speedtest::perf_disable();
    return speedtest::check("958003200\n", "%lld\n", acc);
}
